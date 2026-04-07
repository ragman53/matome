//! Web crawler module
//!
//! Handles HTTP fetching, sitemap.xml parsing, and robots.txt compliance.

use crate::config::{Config, Domain};
use crate::pipeline::RawPage;
use reqwest::Client;
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};
use url::Url;

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Sitemap parse error: {0}")]
    SitemapParse(String),
    #[error("Robots.txt error: {0}")]
    RobotsTxt(String),
}

/// Web crawler
pub struct Crawler {
    client: Client,
    config: Arc<Config>,
}

impl Crawler {
    /// Create a new crawler instance
    pub fn new(config: &Config) -> Result<Self, CrawlerError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.crawl.timeout))
            .user_agent("matome/0.1.0")
            .build()?;

        Ok(Self {
            client,
            config: Arc::new(config.clone()),
        })
    }

    /// Fetch and parse robots.txt, return disallowed paths
    async fn fetch_robots_txt(&self, base_url: &str) -> Result<Vec<String>, CrawlerError> {
        let robots_url = format!("{}/robots.txt", base_url);

        let response = self.client.get(&robots_url).send().await?;

        if !response.status().is_success() {
            return Ok(Vec::new()); // robots.txt not found, allow all
        }

        let body = response.text().await
            .map_err(|e| CrawlerError::RobotsTxt(e.to_string()))?;

        // Simple robots.txt parser - extract Disallow paths
        let mut disallowed = Vec::new();
        for line in body.lines() {
            let line = line.trim();
            if line.starts_with("Disallow:") {
                let path = line.trim_start_matches("Disallow:").trim();
                if !path.is_empty() && !path.starts_with("*") {
                    disallowed.push(path.to_string());
                }
            }
        }

        Ok(disallowed)
    }

    /// Check if URL is allowed by robots.txt
    fn is_allowed_by_robots(&self, url: &str, disallowed: &[String]) -> bool {
        if !self.config.crawl.respect_robots {
            return true;
        }

        !disallowed.iter().any(|path| url.contains(path))
    }

    /// Crawl a domain and return all discovered pages
    pub async fn crawl(&self, domain: &Domain) -> Result<Vec<RawPage>, CrawlerError> {
        info!("Starting crawl for: {}", domain.url);

        // Fetch robots.txt first if configured
        let disallowed = if self.config.crawl.respect_robots {
            self.fetch_robots_txt(&domain.url).await.unwrap_or_default()
        } else {
            Vec::new()
        };

        // Try to get URLs from sitemap.xml first
        let sitemap_urls = self.fetch_sitemap(&domain.url).await?;

        if !sitemap_urls.is_empty() {
            debug!("Found {} URLs in sitemap", sitemap_urls.len());
            let mut pages = Vec::new();

            for url in sitemap_urls {
                if self.is_allowed_by_robots(&url, &disallowed) {
                    if let Some(raw_page) = self.fetch_page(&url).await {
                        pages.push(raw_page);
                    }
                }
            }

            return Ok(pages);
        }

        // Fall back to fetching the root page and extracting links
        let mut pages = Vec::new();
        let mut discovered = std::collections::HashSet::new();
        let mut to_visit = vec![domain.url.clone()];

        while let Some(url) = to_visit.pop() {
            if discovered.contains(&url) {
                continue;
            }
            discovered.insert(url.clone());

            if self.is_allowed_by_robots(&url, &disallowed) {
                if let Some(raw_page) = self.fetch_page(&url).await {
                    // Extract links from page
                    let links = self.extract_links(&raw_page.html, &url);
                    for link in links {
                        if !discovered.contains(&link) && link.starts_with(&domain.url) {
                            to_visit.push(link);
                        }
                    }
                    pages.push(raw_page);
                }
            }
        }

        info!("Crawl complete: {} pages discovered", pages.len());
        Ok(pages)
    }

    /// Fetch a single page
    async fn fetch_page(&self, url: &str) -> Option<RawPage> {
        debug!("Fetching: {}", url);

        match self.client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    response.text().await.ok().map(|html| RawPage {
                        url: url.to_string(),
                        html,
                    })
                } else {
                    warn!("HTTP {} for: {}", response.status(), url);
                    None
                }
            }
            Err(e) => {
                warn!("Failed to fetch {}: {}", url, e);
                None
            }
        }
    }

    /// Fetch and parse sitemap.xml
    async fn fetch_sitemap(&self, base_url: &str) -> Result<Vec<String>, CrawlerError> {
        let sitemap_url = format!("{}/sitemap.xml", base_url);

        let response = match self.client.get(&sitemap_url).send().await {
            Ok(resp) => resp,
            Err(_) => return Ok(Vec::new()), // Sitemap not found, that's OK
        };

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let body = response
            .text()
            .await
            .map_err(|e| CrawlerError::SitemapParse(e.to_string()))?;

        let xml = roxmltree::Document::parse(&body)
            .map_err(|e| CrawlerError::SitemapParse(e.to_string()))?;

        let mut urls = Vec::new();

        // Parse standard sitemap
        for node in xml.descendants().filter(|n| n.has_tag_name("loc")) {
            if let Some(loc) = node.text().map(|t| t.trim().to_string()) {
                if !loc.is_empty() {
                    urls.push(loc);
                }
            }
        }

        // Parse sitemap index
        for node in xml.descendants().filter(|n| n.has_tag_name("sitemap")) {
            for loc in node.descendants().filter(|n| n.has_tag_name("loc")) {
                if let Some(loc_text) = loc.text().map(|t| t.trim().to_string()) {
                    // Recursively fetch sub-sitemaps
                    if let Ok(sub_urls) = self.client.get(&loc_text).send().await {
                        if sub_urls.status().is_success() {
                            if let Ok(sub_body) = sub_urls.text().await {
                                if let Ok(sub_xml) = roxmltree::Document::parse(&sub_body) {
                                    for loc_node in sub_xml
                                        .descendants()
                                        .filter(|n| n.has_tag_name("loc"))
                                    {
                                        if let Some(loc_text) =
                                            loc_node.text().map(|t| t.to_string())
                                        {
                                            urls.push(loc_text);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(urls)
    }

    /// Extract links from HTML page
    fn extract_links(&self, html: &str, base_url: &str) -> Vec<String> {
        let mut links = Vec::new();

        let base = match Url::parse(base_url) {
            Ok(b) => b,
            Err(_) => return links,
        };

        // Simple regex to find href attributes
        let re = regex_lite::Regex::new(r#"href=["']([^"']+)["']"#).unwrap();

        for cap in re.captures_iter(html) {
            if let Some(href) = cap.get(1) {
                let href_str = href.as_str();
                if href_str.starts_with('/') || href_str.starts_with('#') {
                    // Relative URL
                    if let Ok(full_url) = base.join(href_str) {
                        links.push(full_url.to_string());
                    }
                } else if href_str.starts_with("http") {
                    // Absolute URL
                    links.push(href_str.to_string());
                }
            }
        }

        links
    }
}

#[cfg(test)]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links() {
        let crawler = Crawler::new(&Config::default()).unwrap();
        let html = "<html><a href=\"/test\">Test</a></html>";
        let links = crawler.extract_links(html, "https://example.com");
        assert_eq!(links.len(), 1);
    }
}
