//! Data pipeline module
//!
//! Orchestrates the data flow: Crawl -> Extract -> Translate -> Store

mod crawler;
mod extractor;
mod translator;
mod glossary;
mod tree_inference;
mod content_hash;
mod change_detection;

pub use crawler::{Crawler, CrawlerError};
pub use extractor::Extractor;
pub use translator::Translator;
pub use tree_inference::{infer_tree_path, infer_breadcrumbs};
pub use content_hash::compute_content_hash;

use crate::config::{Config, Domain};
use crate::db::Database;
use crate::db::search::SearchEngine;
use crate::pipeline::glossary::Glossary;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Crawl error: {0}")]
    Crawl(#[from] CrawlerError),
    #[error("Extract error: {0}")]
    Extract(String),
    #[error("Translate error: {0}")]
    Translate(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Page data from crawler
#[derive(Debug, Clone)]
pub struct RawPage {
    #[allow(dead_code)] // Future: debugging, logging
    pub url: String,
    pub html: String,
}

/// Extracted page data
#[derive(Debug, Clone)]
pub struct ExtractedPage {
    #[allow(dead_code)] // Future: debugging, logging
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub markdown: String,
}

/// Translated page data
#[derive(Debug, Clone)]
pub struct TranslatedPage {
    #[allow(dead_code)] // Future: debugging, logging
    pub url: String,
    #[allow(dead_code)] // Future: debugging, logging
    pub title: String,
    #[allow(dead_code)] // Future: debugging, logging
    pub description: Option<String>,
    #[allow(dead_code)] // Future: debugging, logging
    pub original_md: String,
    #[allow(dead_code)] // Future: debugging, logging
    pub translated_md: String,
    #[allow(dead_code)] // Future: debugging, logging
    pub domain: String,
    // v0.2.0: hierarchical data (computed but not yet read back)
    #[allow(dead_code)]
    pub tree_path: String,
    #[allow(dead_code)]
    pub breadcrumbs: Vec<String>,
    #[allow(dead_code)]
    pub content_hash: String,
}

/// Pipeline execution report
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct PipelineReport {
    pub pages_crawled: usize,
    pub pages_extracted: usize,
    pub pages_translated: usize,
    pub pages_stored: usize,
    pub errors: Vec<String>,
}

/// Main pipeline orchestrator
pub struct Pipeline {
    config: Arc<Config>,
    crawler: Crawler,
    extractor: Extractor,
    translator: Option<Translator>,
    glossary: Option<Glossary>,
    search_engine: Option<Arc<SearchEngine>>,
    db: Database,
    concurrency: Arc<Semaphore>,
}

/// Context for processing a single page
struct PageProcessContext {
    extractor: Extractor,
    translator: Option<Translator>,
    glossary: Option<Glossary>,
    search_engine: Option<Arc<SearchEngine>>,
    db: Database,
    semaphore: Arc<Semaphore>,
    domain_name: String,
    target_lang: String,
    base_url: String, // v0.2.0: for tree_path inference
}

impl Pipeline {
    /// Create a new pipeline instance
    pub async fn new(config: &Config) -> Result<Self, PipelineError> {
        let config = Arc::new(config.clone());
        let db = Database::new(&std::path::PathBuf::from(&config.core.data_dir))
            .map_err(|e| PipelineError::Storage(e.to_string()))?;

        let crawler = Crawler::new(&config)?;
        let translator = Self::init_translator(&config)?;
        let glossary = Self::load_glossary(&config)?;
        let search_engine = Self::init_search_engine(&config)?;
        let concurrency = Arc::new(Semaphore::new(config.crawl.concurrency));

        Ok(Self { config, crawler, extractor: Extractor::new(), translator, glossary, search_engine, db, concurrency })
    }

    fn init_translator(config: &Arc<Config>) -> Result<Option<Translator>, PipelineError> {
        if config.translate.provider != "none" {
            Ok(Some(Translator::new(config).map_err(|e| PipelineError::Translate(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn load_glossary(config: &Arc<Config>) -> Result<Option<Glossary>, PipelineError> {
        let Some(glossary_file) = &config.translate.glossary_file else { return Ok(None); };
        let path = std::path::PathBuf::from(glossary_file);
        match Glossary::load(&path) {
            Ok(g) if g.has_terms() => { info!("Loaded glossary with {} terms", g.term_count()); Ok(Some(g)) }
            Ok(_) => { info!("Glossary file is empty, skipping"); Ok(None) }
            Err(e) => { warn!("Failed to load glossary: {}, continuing without", e); Ok(None) }
        }
    }

    fn init_search_engine(config: &Arc<Config>) -> Result<Option<Arc<SearchEngine>>, PipelineError> {
        let data_dir = std::path::PathBuf::from(&config.core.data_dir);
        match SearchEngine::new(&data_dir) {
            Ok(se) => { info!("Search engine initialized"); Ok(Some(Arc::new(se))) }
            Err(e) => { warn!("Failed to initialize search engine: {}, continuing without", e); Ok(None) }
        }
    }

    /// Run the full pipeline
    pub async fn run(&mut self, incremental: bool) -> Result<PipelineReport, PipelineError> {
        let mut report = PipelineReport::default();
        for domain in self.config.domains.clone() {
            let domain_report = self.process_domain(&domain, incremental).await;
            report += domain_report;
        }
        Ok(report)
    }

    /// Process a single domain
    async fn process_domain(&self, domain: &Domain, incremental: bool) -> PipelineReport {
        let existing_urls = self.get_existing_urls(domain, incremental);
        let raw_pages = match self.crawler.crawl(domain).await {
            Ok(pages) => pages,
            Err(e) => return PipelineReport { errors: vec![format!("Crawl error for {}: {}", domain.url, e)], ..Default::default() },
        };

        let ctx = PageProcessContext {
            extractor: self.extractor.clone(),
            translator: self.translator.clone(),
            glossary: self.glossary.clone(),
            search_engine: self.search_engine.clone(),
            db: self.db.clone(),
            semaphore: self.concurrency.clone(),
            domain_name: domain.normalized_name(self.config.crawl.treat_subdomains_same),
            target_lang: self.config.translate.target_lang.clone(),
            base_url: domain.url.clone(), // v0.2.0
        };

        let results = Self::process_pages(raw_pages, existing_urls, incremental, ctx).await;
        Self::aggregate_results(results)
    }

    fn get_existing_urls(&self, domain: &Domain, incremental: bool) -> std::collections::HashSet<String> {
        if incremental { self.db.get_urls_by_domain(&domain.normalized_name(self.config.crawl.treat_subdomains_same)).unwrap_or_default() } else { std::collections::HashSet::new() }
    }

    async fn process_pages(raw_pages: Vec<RawPage>, existing_urls: std::collections::HashSet<String>, incremental: bool, ctx: PageProcessContext) -> Vec<Result<(), PipelineError>> {
        let filtered: Vec<RawPage> = if incremental { raw_pages.into_iter().filter(|p| !existing_urls.contains(&p.url)).collect() } else { raw_pages };
        let total = filtered.len();
        
        if total == 0 {
            info!("No new pages to process");
            return Vec::new();
        }
        
        info!("Processing {} pages (extract + translate + store)...", total);
        
        let futures = filtered.into_iter().enumerate().map(|(i, page)| {
            let ctx = PageProcessContext {
                extractor: ctx.extractor.clone(),
                translator: ctx.translator.clone(),
                glossary: ctx.glossary.clone(),
                search_engine: ctx.search_engine.clone(),
                db: ctx.db.clone(),
                semaphore: ctx.semaphore.clone(),
                domain_name: ctx.domain_name.clone(),
                target_lang: ctx.target_lang.clone(),
                base_url: ctx.base_url.clone(), // v0.2.0
            };
            async move {
                let i = i + 1;
                print_process_progress(i, total, &page.url);
                Self::process_single_page(page, &ctx).await
            }
        });
        let results = futures::future::join_all(futures).await;
        println!(); // Newline after progress
        results
    }

    async fn process_single_page(raw_page: RawPage, ctx: &PageProcessContext) -> Result<(), PipelineError> {
        let _permit = ctx.semaphore.acquire().await;
        
        let extracted = ctx.extractor.extract(&raw_page.html, &raw_page.url)
            .map_err(|e| PipelineError::Extract(e.to_string()))?;
        
        // Skip empty or very small pages
        if extracted.markdown.len() < 50 {
            debug!("Skipping empty/small page: {}", raw_page.url);
            return Ok(());
        }
        
        let translated_md = Self::translate_content(&extracted.markdown, &ctx.translator).await;
        
        let final_md = ctx.glossary.as_ref()
            .map(|g| g.apply_for_lang(&translated_md, &ctx.target_lang))
            .unwrap_or(translated_md);

        // v0.2.0: Compute hierarchical data
        let tree_path = infer_tree_path(&raw_page.url, &ctx.base_url);
        let breadcrumbs = infer_breadcrumbs(&tree_path);
        let content_hash = compute_content_hash(&final_md);

        let page = TranslatedPage {
            url: raw_page.url.clone(),
            title: extracted.title.clone(),
            description: extracted.description.clone(),
            original_md: extracted.markdown,
            translated_md: final_md.clone(),
            domain: ctx.domain_name.clone(),
            tree_path,
            breadcrumbs,
            content_hash,
        };

        ctx.db.save_article(&page).map_err(|e| PipelineError::Storage(e.to_string()))?;
        
        if let Some(ref se) = ctx.search_engine {
            if let Err(e) = se.index_document(&raw_page.url, &extracted.title, &final_md, &ctx.domain_name) {
                warn!("Failed to index document {}: {}", raw_page.url, e);
            }
        }
        
        Ok(())
    }

    async fn translate_content(content: &str, translator: &Option<Translator>) -> String {
        let Some(t) = translator else { return content.to_string(); };
        match t.translate(content).await {
            Ok(translated) => translated,
            Err(e) => {
                warn!("Translation failed, using original: {}", e);
                content.to_string()
            }
        }
    }

    fn aggregate_results(results: Vec<Result<(), PipelineError>>) -> PipelineReport {
        let (successes, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
        PipelineReport {
            pages_crawled: successes.len(),
            pages_extracted: successes.len(),
            pages_translated: successes.len(),
            pages_stored: successes.len(),
            errors: errors.into_iter().filter_map(Result::err).map(|e| e.to_string()).collect(),
        }
    }
}


impl std::ops::AddAssign for PipelineReport {
    fn add_assign(&mut self, rhs: Self) {
        self.pages_crawled += rhs.pages_crawled;
        self.pages_extracted += rhs.pages_extracted;
        self.pages_translated += rhs.pages_translated;
        self.pages_stored += rhs.pages_stored;
        self.errors.extend(rhs.errors);
    }
}

/// Print progress for processing pages
fn print_process_progress(current: usize, total: usize, url: &str) {
    let display_url = if url.len() > 60 {
        format!("...{}", &url[url.len()-60..])
    } else {
        url.to_string()
    };
    
    let pct = if total > 0 {
        (current as f64 / total as f64 * 100.0) as u32
    } else {
        0
    };
    
    let bar_width = 30;
    let filled = (current as f64 / total as f64 * bar_width as f64) as usize;
    let bar: String = "=".repeat(filled) + &"-".repeat(bar_width - filled);
    
    print!("\r[{bar}] {pct:3}% ({current}/{total}) {display_url}");
    std::io::Write::flush(&mut std::io::stdout()).ok();
}
