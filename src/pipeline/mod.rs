//! Data pipeline module
//!
//! Orchestrates the data flow: Crawl -> Extract -> Translate -> Store

mod crawler;
mod extractor;
mod translator;
mod glossary;

pub use crawler::{Crawler, CrawlerError};
pub use extractor::Extractor;
pub use translator::Translator;

use crate::config::{Config, Domain};
use crate::db::Database;
use crate::db::search::SearchEngine;
use crate::pipeline::glossary::Glossary;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Semaphore;
use tracing::{info, warn};

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
    pub url: String,
    pub html: String,
}

/// Extracted page data
#[derive(Debug, Clone)]
pub struct ExtractedPage {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub markdown: String,
}

/// Translated page data
#[derive(Debug, Clone)]
pub struct TranslatedPage {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub original_md: String,
    pub translated_md: String,
    pub domain: String,
}

/// Pipeline execution report
#[derive(Debug, Clone, Default)]
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

impl Pipeline {
    /// Create a new pipeline instance
    pub async fn new(config: &Config) -> Result<Self, PipelineError> {
        let config = Arc::new(config.clone());
        let db = Database::new(&std::path::PathBuf::from(&config.core.data_dir))
            .map_err(|e| PipelineError::Storage(e.to_string()))?;

        let crawler = Crawler::new(&config)?;
        let extractor = Extractor::new();

        let translator = if config.translate.provider != "none" {
            Some(Translator::new(&config).map_err(|e| PipelineError::Translate(e.to_string()))?)
        } else {
            None
        };

        // Load glossary if configured
        let glossary = if let Some(glossary_file) = &config.translate.glossary_file {
            let path = std::path::PathBuf::from(glossary_file);
            match Glossary::load(&path) {
                Ok(g) if g.has_terms() => {
                    info!("Loaded glossary with {} terms", g.term_count());
                    Some(g)
                }
                Ok(_) => {
                    info!("Glossary file is empty, skipping");
                    None
                }
                Err(e) => {
                    warn!("Failed to load glossary: {}, continuing without", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize search engine
        let data_dir = std::path::PathBuf::from(&config.core.data_dir);
        let search_engine = match SearchEngine::new(&data_dir) {
            Ok(se) => {
                info!("Search engine initialized");
                Some(Arc::new(se))
            }
            Err(e) => {
                warn!("Failed to initialize search engine: {}, continuing without", e);
                None
            }
        };

        let concurrency = Arc::new(Semaphore::new(config.crawl.concurrency));

        Ok(Self {
            config,
            crawler,
            extractor,
            translator,
            glossary,
            search_engine,
            db,
            concurrency,
        })
    }

    /// Run the full pipeline
    pub async fn run(&mut self, incremental: bool) -> Result<PipelineReport, PipelineError> {
        let mut report = PipelineReport::default();

        // Clone domain info to avoid borrow conflicts
        let domain_infos: Vec<(String, Vec<String>)> = self.config.domains
            .iter()
            .map(|d| (d.url.clone(), d.include.clone()))
            .collect();
        
        // Process each domain
        for (url, include) in domain_infos {
            let domain = Domain { url: url.clone(), include };
            info!("Processing domain: {}", domain.url);

            let domain_report = self.process_domain(&domain, incremental).await;
            domain_report.errors.iter().for_each(|e| warn!("{}", e));
            
            // Accumulate stats
            report.pages_crawled += domain_report.pages_crawled;
            report.pages_extracted += domain_report.pages_extracted;
            report.pages_translated += domain_report.pages_translated;
            report.pages_stored += domain_report.pages_stored;
            report.errors.extend(domain_report.errors);
        }

        Ok(report)
    }

    /// Process a single domain
    async fn process_domain(
        &mut self,
        domain: &Domain,
        incremental: bool,
    ) -> PipelineReport {
        let mut report = PipelineReport::default();

        // Get existing URLs if incremental
        let existing_urls = if incremental {
            self.db.get_urls_by_domain(&domain.name())
                .unwrap_or_default()
        } else {
            std::collections::HashSet::new()
        };

        // Crawl domain
        let raw_pages = match self.crawler.crawl(domain).await {
            Ok(pages) => {
                report.pages_crawled = pages.len();
                pages
            }
            Err(e) => {
                report.errors.push(format!("Crawl error for {}: {}", domain.url, e));
                return report;
            }
        };

        // Process pages concurrently
        let semaphore = self.concurrency.clone();
        let translator = self.translator.clone();
        let glossary = self.glossary.clone();
        let search_engine = self.search_engine.clone();
        let db = &self.db;
        let extractor = self.extractor.clone();
        let domain_name = domain.name();
        let target_lang = self.config.translate.target_lang.clone();

        let futures = raw_pages.into_iter().filter(|page| {
            // Skip existing URLs in incremental mode
            if incremental {
                !existing_urls.contains(&page.url)
            } else {
                true
            }
        }).map(|raw_page| {
            let extractor = extractor.clone();
            let translator = translator.clone();
            let glossary = glossary.clone();
            let search_engine = search_engine.clone();
            let db = db;
            let semaphore = semaphore.clone();
            let domain_name = domain_name.clone();
            let target_lang = target_lang.clone();

            async move {
                let _permit = semaphore.acquire().await;

                // Extract markdown
                let extracted = match extractor.extract(&raw_page.html, &raw_page.url) {
                    Ok(extracted) => extracted,
                    Err(e) => return Err(PipelineError::Extract(e.to_string())),
                };

                // Translate
                let mut translated_md = if let Some(ref translator) = translator {
                    match translator.translate(&extracted.markdown).await {
                        Ok(tmd) => tmd,
                        Err(_e) => {
                            // Fall back to original if translation fails
                            extracted.markdown.clone()
                        }
                    }
                } else {
                    extracted.markdown.clone()
                };

                // Apply glossary replacements if configured
                if let Some(ref gloss) = glossary {
                    translated_md = gloss.apply_for_lang(&translated_md, &target_lang);
                }

                // Store in database
                let page = TranslatedPage {
                    url: raw_page.url.clone(),
                    title: extracted.title.clone(),
                    description: extracted.description.clone(),
                    original_md: extracted.markdown,
                    translated_md: translated_md.clone(),
                    domain: domain_name.clone(),
                };

                if let Err(e) = db.save_article(&page) {
                    return Err(PipelineError::Storage(e.to_string()));
                }

                // Index document for full-text search
                if let Some(ref se) = search_engine {
                    let url = raw_page.url.clone();
                    let title = extracted.title.clone();
                    let content = translated_md.clone();
                    let domain = domain_name.clone();
                    if let Err(e) = se.index_document(&url, &title, &content, &domain) {
                        warn!("Failed to index document: {}", e);
                    }
                }

                Ok::<(), PipelineError>(())
            }
        });

        // Execute futures
        let results = futures::future::join_all(futures).await;

        for result in results {
            match result {
                Ok(()) => {
                    report.pages_stored += 1;
                }
                Err(e) => {
                    report.errors.push(e.to_string());
                }
            }
        }

        report.pages_extracted = report.pages_stored;
        report.pages_translated = report.pages_stored;

        report
    }
}

impl Extractor {
    fn clone(&self) -> Self {
        Extractor::new()
    }
}
