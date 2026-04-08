//! Web crawler module
//!
//! Handles HTTP fetching, sitemap.xml parsing, and robots.txt compliance.

use crate::config::{Config, Domain};
use crate::pipeline::RawPage;
use reqwest::Client;
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
            let total = sitemap_urls.len();
            let allowed_urls: Vec<_> = sitemap_urls.into_iter()
                .filter(|url| self.is_allowed_by_robots(url, &disallowed))
                .collect();
            
            info!("Crawling {} pages from sitemap...", allowed_urls.len());
            let mut pages = Vec::new();
            
            for (i, url) in allowed_urls.iter().enumerate() {
                print_progress(i + 1, total, url);
                if let Some(raw_page) = self.fetch_page(url).await {
                    pages.push(raw_page);
                }
            }
            
            println!(); // Newline after progress
            info!("Crawl complete: {} pages discovered", pages.len());
            return Ok(pages);
        }

        // Fall back to fetching the root page and extracting links
        let mut pages = Vec::new();
        let mut discovered = std::collections::HashSet::new();
        let mut to_visit = vec![domain.url.clone()];
        let mut total_crawled = 0;

        info!("Crawling via link traversal...");
        
        while let Some(url) = to_visit.pop() {
            if discovered.contains(&url) {
                continue;
            }
            discovered.insert(url.clone());

            if self.is_allowed_by_robots(&url, &disallowed) {
                total_crawled += 1;
                print_progress(total_crawled, discovered.len().max(total_crawled), &url);
                
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
            
            // Check max_pages limit
            if self.config.crawl.max_pages > 0 && pages.len() >= self.config.crawl.max_pages {
                println!();
                info!("Reached max_pages limit: {}", self.config.crawl.max_pages);
                break;
            }
        }

        println!(); // Newline after progress
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
            Err(_) => return Ok(Vec::new()),
        };

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let body = response.text().await.map_err(|e| CrawlerError::SitemapParse(e.to_string()))?;
        self.parse_sitemap_with_submaps(&body).await
    }

    async fn parse_sitemap_with_submaps(&self, body: &str) -> Result<Vec<String>, CrawlerError> {
        let xml = roxmltree::Document::parse(body)
            .map_err(|e| CrawlerError::SitemapParse(e.to_string()))?;

        let mut urls = extract_loc_nodes(&xml);
        self.collect_sub_sitemap_urls(&xml, &mut urls).await;
        Ok(urls)
    }

    async fn collect_sub_sitemap_urls(&self, xml: &roxmltree::Document<'_>, urls: &mut Vec<String>) {
        for node in xml.descendants().filter(|n| n.has_tag_name("sitemap")) {
            for loc in node.descendants().filter(|n| n.has_tag_name("loc")) {
                if let Some(loc_text) = loc.text().map(|t| t.trim().to_string()) {
                    if !loc_text.is_empty() {
                        if let Ok(sub_urls) = self.fetch_sub_sitemap(&loc_text).await {
                            urls.extend(sub_urls);
                        }
                    }
                }
            }
        }
    }

    async fn fetch_sub_sitemap(&self, url: &str) -> Result<Vec<String>, CrawlerError> {
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Ok(Vec::new());
        }
        let body = response.text().await.map_err(|e| CrawlerError::SitemapParse(e.to_string()))?;
        Ok(extract_loc_nodes_from_str(&body))
    }

    /// Extract links from HTML page
    fn extract_links(&self, html: &str, base_url: &str) -> Vec<String> {
        let base = match Url::parse(base_url) {
            Ok(b) => b,
            Err(_) => return Vec::new(),
        };

        let re = regex_lite::Regex::new(r#"href=["']([^"']+)["']"#).unwrap();
        re.captures_iter(html)
            .filter_map(|cap| {
                cap.get(1).and_then(|href| {
                    let href_str = href.as_str();
                    if href_str.starts_with('/') || href_str.starts_with('#') {
                        base.join(href_str).ok().map(|u| u.to_string())
                    } else if href_str.starts_with("http") {
                        Some(href_str.to_string())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

/// Print progress bar for crawling
fn print_progress(current: usize, total: usize, url: &str) {
    // Truncate URL for display
    let display_url = if url.len() > 60 {
        format!("...{}", &url[url.len()-60..])
    } else {
        url.to_string()
    };
    
    // Calculate percentage
    let pct = if total > 0 {
        (current as f64 / total as f64 * 100.0) as u32
    } else {
        0
    };
    
    // Progress bar
    let bar_width = 30;
    let filled = (current as f64 / total as f64 * bar_width as f64) as usize;
    let bar: String = "=".repeat(filled) + &"-".repeat(bar_width - filled);
    
    print!("\r[{bar}] {pct:3}% ({current}/{total}) {display_url}");
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

/// Extract all loc nodes from an XML document
fn extract_loc_nodes(xml: &roxmltree::Document) -> Vec<String> {
    xml.descendants()
        .filter(|n| n.has_tag_name("loc"))
        .filter_map(|n| n.text().map(|t| t.trim().to_string()))
        .filter(|loc| !loc.is_empty())
        .collect()
}

/// Extract loc nodes from a string (for sub-sitemaps)
fn extract_loc_nodes_from_str(body: &str) -> Vec<String> {
    roxmltree::Document::parse(body)
        .map(|xml| extract_loc_nodes(&xml))
        .unwrap_or_default()
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
