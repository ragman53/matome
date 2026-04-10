//! Web crawler module - Optimized for speed
//!
//! Handles HTTP fetching, sitemap.xml parsing, and robots.txt compliance.
//! Optimizations:
//! - Parallel fetching with connection pooling
//! - Batch processing with progress reporting
//! - Retry with exponential backoff
//! - Smart sitemap parsing

use crate::config::{Config, Domain};
use crate::pipeline::RawPage;
use futures::future;
use reqwest::Client;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Semaphore;
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

/// Web crawler - Optimized for parallel fetching
pub struct Crawler {
    client: Client,
    config: Arc<Config>,
    semaphore: Arc<Semaphore>,
}

impl Crawler {
    /// Create a new crawler instance with connection pooling
    pub fn new(config: &Config) -> Result<Self, CrawlerError> {
        let concurrency = config.crawl.concurrency;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.crawl.timeout))
            .user_agent("matome/0.2.0 (+https://github.com/ragman53/matome)")
            // Connection pooling - critical for speed
            .pool_max_idle_per_host(concurrency)
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .tcp_nodelay(true)
            .build()?;

        Ok(Self {
            client,
            config: Arc::new(config.clone()),
            semaphore: Arc::new(Semaphore::new(concurrency)),
        })
    }

    /// Create a new crawler with custom concurrency
    #[allow(dead_code)]
    pub fn with_concurrency(config: &Config, concurrency: usize) -> Result<Self, CrawlerError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.crawl.timeout))
            .user_agent("matome/0.2.0 (+https://github.com/ragman53/matome)")
            .pool_max_idle_per_host(concurrency)
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .tcp_nodelay(true)
            .build()?;

        Ok(Self {
            client,
            config: Arc::new(config.clone()),
            semaphore: Arc::new(Semaphore::new(concurrency)),
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

    /// Crawl a domain and return all discovered pages (OPTIMIZED - PARALLEL)
    pub async fn crawl(&self, domain: &Domain) -> Result<Vec<RawPage>, CrawlerError> {
        info!("Starting crawl for: {} (concurrency: {})", domain.url, self.config.crawl.concurrency);

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
            let allowed_urls: Vec<_> = sitemap_urls.into_iter()
                .filter(|url| self.is_allowed_by_robots(url, &disallowed))
                .collect();
            
            info!("Crawling {} pages from sitemap (parallel)...", allowed_urls.len());
            return self.crawl_parallel(allowed_urls).await;
        }

        // Fall back to fetching the root page and extracting links
        info!("Crawling via link traversal (parallel)...");
        self.crawl_parallel_link_traversal(domain, disallowed).await
    }

    /// Parallel crawling of URLs from sitemap
    async fn crawl_parallel(&self, urls: Vec<String>) -> Result<Vec<RawPage>, CrawlerError> {
        let total = urls.len();
        let semaphore = self.semaphore.clone();
        let client = self.client.clone();

        // Progress tracking
        let completed = Arc::new(AtomicUsize::new(0));
        let start_time = std::time::Instant::now();

        // Process in batches for better progress reporting
        let batch_size = self.config.crawl.concurrency * 2;
        let mut all_pages = Vec::with_capacity(total);

        for chunk in urls.chunks(batch_size) {
            let futures: Vec<_> = chunk.iter().map(|url| {
                let url = url.clone();
                let sem = semaphore.clone();
                let client = client.clone();
                let comp = completed.clone();
                
                async move {
                    let _permit = sem.acquire().await.ok();
                    
                    // Retry logic (up to 3 attempts)
                    for attempt in 0..3 {
                        match client.get(&url).send().await {
                            Ok(response) if response.status().is_success() => {
                                if let Ok(html) = response.text().await {
                                    let c = comp.fetch_add(1, Ordering::Relaxed) + 1;
                                    print_progress(c, total, &url, start_time);
                                    return Some(RawPage { url, html });
                                }
                            }
                            Ok(resp) => {
                                warn!("HTTP {} for: {}", resp.status(), url);
                            }
                            Err(_) if attempt < 2 => {
                                // Exponential backoff
                                tokio::time::sleep(std::time::Duration::from_millis(100 * 2_u64.pow(attempt))).await;
                                continue;
                            }
                            Err(e) => {
                                warn!("Failed to fetch {}: {}", url, e);
                            }
                        }
                    }
                    comp.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }).collect();

            // Execute batch in parallel
            let results = future::join_all(futures).await;
            
            for page in results.into_iter().flatten() {
                all_pages.push(page);
            }
        }

        println!(); // Newline after progress
        info!("Crawl complete: {} pages", all_pages.len());
        Ok(all_pages)
    }

    /// Parallel crawling via link traversal
    async fn crawl_parallel_link_traversal(&self, domain: &Domain, disallowed: Vec<String>) -> Result<Vec<RawPage>, CrawlerError> {
        let mut pages = Vec::new();
        let mut discovered: HashSet<String> = HashSet::new();
        let mut to_visit = vec![domain.url.clone()];
        let semaphore = self.semaphore.clone();
        let client = self.client.clone();
        let domain_url = domain.url.clone();
        let max_pages = self.config.crawl.max_pages;

        let completed = Arc::new(AtomicUsize::new(0));
        let _start_time = std::time::Instant::now();
        
        while !to_visit.is_empty() {
            // Check max_pages limit
            if max_pages > 0 && pages.len() >= max_pages {
                info!("Reached max_pages limit: {}", max_pages);
                break;
            }

            // Take a batch of URLs
            let batch: Vec<String> = to_visit.drain(..std::cmp::min(100, to_visit.len())).collect();
            
            // Filter already discovered
            let batch: Vec<String> = batch.into_iter()
                .filter(|url| !discovered.contains(url))
                .collect();

            if batch.is_empty() {
                continue;
            }

            // Mark as discovered
            for url in &batch {
                discovered.insert(url.clone());
            }

            // Fetch batch in parallel
            let futures: Vec<_> = batch.iter().map(|url| {
                let url = url.clone();
                let sem = semaphore.clone();
                let client = client.clone();
                let comp = completed.clone();
                
                async move {
                    let _permit = sem.acquire().await.ok();
                    
                    match client.get(&url).send().await {
                        Ok(response) if response.status().is_success() => {
                            if let Ok(html) = response.text().await {
                                let c = comp.fetch_add(1, Ordering::Relaxed) + 1;
                                print_progress_simple(c, &url);
                                return Some((url, html));
                            }
                        }
                        _ => {}
                    }
                    comp.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }).collect();

            let results = future::join_all(futures).await;

            for result in results.into_iter().flatten() {
                let (url, html) = result;
                
                // Extract links
                let links = self.extract_links(&html, &url);
                for link in links {
                    if !discovered.contains(&link) && link.starts_with(&domain_url)
                        && self.is_allowed_by_robots(&link, &disallowed) {
                            to_visit.push(link);
                        }
                }
                
                pages.push(RawPage { url, html });
            }
        }

        println!();
        info!("Crawl complete: {} pages discovered from {}", pages.len(), discovered.len());
        Ok(pages)
    }

    /// Fetch a single page (used for fallback)
    #[allow(dead_code)]
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
        
        // Fetch sub-sitemaps in parallel
        let sitemap_locs: Vec<String> = xml.descendants()
            .filter(|n| n.has_tag_name("sitemap"))
            .filter_map(|n| n.descendants().filter(|m| m.has_tag_name("loc"))
                .filter_map(|m| m.text().map(|t| t.trim().to_string()))
                .next())
            .collect();

        if !sitemap_locs.is_empty() {
            info!("Fetching {} sub-sitemaps...", sitemap_locs.len());
            
            let futures: Vec<_> = sitemap_locs.iter().map(|loc| {
                let loc = loc.clone();
                let client = self.client.clone();
                async move {
                    match client.get(&loc).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            if let Ok(body) = resp.text().await {
                                return extract_loc_nodes_from_str(&body);
                            }
                        }
                        _ => {}
                    }
                    Vec::new()
                }
            }).collect();

            let results = future::join_all(futures).await;
            for sub_urls in results {
                urls.extend(sub_urls);
            }
        }

        // Deduplicate
        urls.sort();
        urls.dedup();
        Ok(urls)
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

/// Print progress bar with speed indicator
fn print_progress(current: usize, total: usize, url: &str, start_time: std::time::Instant) {
    let elapsed = start_time.elapsed().as_secs_f64();
    let rate = if elapsed > 0.0 { current as f64 / elapsed } else { 0.0 };
    
    // Truncate URL for display
    let display_url = if url.len() > 50 {
        format!("...{}", &url[url.len()-50..])
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
    let bar_width = 25;
    let filled = if total > 0 {
        (current as f64 / total as f64 * bar_width as f64) as usize
    } else {
        current % bar_width
    };
    let bar: String = "=".repeat(filled) + &"-".repeat(bar_width - filled);
    
    print!("\r[{bar}] {pct:3}% | {current}/{total} | {rate:.1}/s | {display_url}");
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

/// Simple progress without total (for link traversal)
fn print_progress_simple(current: usize, url: &str) {
    // Truncate URL for display
    let display_url = if url.len() > 60 {
        format!("...{}", &url[url.len()-60..])
    } else {
        url.to_string()
    };
    
    print!("\r[{current:5}] {display_url}");
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
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_extract_links() {
        let config = Config::default();
        let crawler = Crawler::new(&config).unwrap();
        let html = "<html><a href=\"/test\">Test</a></html>";
        let links = crawler.extract_links(html, "https://example.com");
        assert_eq!(links.len(), 1);
    }
}
