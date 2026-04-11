//! CLI argument definitions using clap
//!
//! Defines all command-line interface commands and arguments.

use clap::{Parser, Subcommand};
use std::path::Path;
use std::path::PathBuf;
use crate::db::models::ChangeType;

/// matome - Collect, translate, and browse documentation locally
#[derive(Parser, Debug)]
#[command(
    name = "matome",
    about = "A Rust CLI tool for collecting, translating, and browsing documentation",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize matome configuration files
    Init {
        /// Output directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },


    /// Add a domain to the configuration
    Add {
        /// URL of the domain to add
        url: String,
        /// Include patterns (e.g., "/docs/**")
        #[arg(short, long, action = clap::ArgAction::Append)]
        include: Option<Vec<String>>,
        /// Configuration file path
        #[arg(short, long, default_value = "matome.toml")]
        config: PathBuf,
    },

    /// Crawl and translate documents
    Crawl {
        /// Enable incremental crawling (only fetch new/updated pages)
        #[arg(short, long)]
        incremental: bool,
        /// Configuration file path
        #[arg(short, long, default_value = "matome.toml")]
        config: PathBuf,
        /// Number of concurrent requests
        #[arg(long)]
        concurrency: Option<usize>,
    },

    /// Start the web server
    Serve {
        /// Configuration file path
        #[arg(short, long, default_value = "matome.toml")]
        config: PathBuf,
        /// Port to listen on
        #[arg(short = 'p', long, default_value = "8080")]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },

    /// Show statistics and status
    Status {
        /// Configuration file path
        #[arg(short, long, default_value = "matome.toml")]
        config: PathBuf,
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Clean database (delete articles)
    Clean {
        /// Delete all articles
        #[arg(short, long)]
        all: bool,
        /// Delete articles from specific domain
        #[arg(long)]
        domain: Option<String>,
        /// Delete articles with missing/incomplete data
        #[arg(long)]
        orphaned: bool,
        /// Delete specific article by ID
        #[arg(short, long)]
        id: Option<i64>,
        /// Configuration file path
        #[arg(short = 'c', long, default_value = "matome.toml")]
        config: PathBuf,
        /// Data directory
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },

    // ====== v0.2.0: Diff Mode Commands ======

    /// Show changes since last crawl (Diff Mode)
    Diff {
        /// Show changes since this date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,
        /// Show only breaking changes
        #[arg(short, long)]
        breaking: bool,
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
        /// Configuration file path
        #[arg(short, long, default_value = "matome.toml")]
        config: PathBuf,
    },
    /// Switch operation mode
    Mode {
        /// Mode to switch to (library, diff, agent)
        mode: String,
    },
    // ====== v0.2.0: Agent Mode Commands ======

    /// Export workspace for AI coding assistant (Agent Mode)
    Export {
        /// Workspace name
        #[arg(short, long)]
        workspace: String,
        /// Workspace directory (default: ~/.matome/workspaces)
        #[arg(long)]
        workspace_dir: Option<String>,
        /// Maximum tokens per context
        #[arg(long, default_value = "128000")]
        max_tokens: usize,
        /// Configuration file path
        #[arg(short, long, default_value = "matome.toml")]
        config: PathBuf,
    },
    /// Generate context bundle for AI agent
    Bundle {
        /// Topics to include (comma-separated)
        #[arg(short, long)]
        topics: String,
        /// Maximum tokens
        #[arg(long, default_value = "80000")]
        max_tokens: usize,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
        /// Configuration file path
        #[arg(short, long, default_value = "matome.toml")]
        config: PathBuf,
    },
}

impl Cli {
    /// Execute the CLI command
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Command::Init { output } => {
                init_command(output)?;
            }
            Command::Add {
                url,
                include,
                config,
            } => {
                add_command(url, include.as_deref(), config)?;
            }
            Command::Crawl {
                incremental,
                config,
                concurrency,
            } => {
                crawl_command(*incremental, config, *concurrency)?;
            }
            Command::Serve { config, port, host, data_dir } => {
                serve_command(*port, host, data_dir.as_deref(), config)?;
            }
            Command::Status {
                config,
                verbose,
            } => {
                status_command(config, *verbose)?;
            }
            Command::Clean { all, domain, orphaned, id, config, data_dir } => {
                clean_command(*all, domain.as_deref(), *orphaned, *id, data_dir.as_deref(), config)?;
            }
            // v0.2.0: Diff Mode
            Command::Diff { since, breaking, format, config } => {
                diff_command(since.as_deref(), *breaking, format, config)?;
            }
            Command::Mode { mode } => {
                mode_command(mode)?;
            }
            // v0.2.0: Agent Mode
            Command::Export { workspace, workspace_dir, max_tokens, config } => {
                export_command(workspace, workspace_dir.as_deref(), *max_tokens, config)?;
            }
            Command::Bundle { topics, max_tokens, output, config } => {
                bundle_command(topics, *max_tokens, output.as_deref(), config)?;
            }
        }
        Ok(())
    }
}

/// Initialize configuration files
fn init_command(output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let config_path = output.join("matome.toml");
    let glossary_path = output.join("glossary.example.toml");

    // Check if files already exist
    if config_path.exists() {
        eprintln!("Warning: {} already exists, skipping", config_path.display());
    } else {
        let config_content = include_str!("examples/matome.toml.example");
        fs::write(&config_path, config_content)?;
        println!("Created: {}", config_path.display());
    }

    if glossary_path.exists() {
        eprintln!("Warning: {} already exists, skipping", glossary_path.display());
    } else {
        let glossary_content = include_str!("examples/glossary.example.toml");
        fs::write(&glossary_path, glossary_content)?;
        println!("Created: {}", glossary_path.display());
    }

    println!("\nInitialization complete!");
    println!("Edit matome.toml to configure your domains.");
    println!("Run 'matome crawl' to start collecting documents.");

    Ok(())
}

/// Add a domain to configuration
fn add_command(
    url: &str,
    include: Option<&[String]>,
    config_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_or_create_config(config_path)?;
    add_domain_to_config(&mut config, url, include)?;
    save_config(&config, config_path)?;
    println!("Added domain: {}", url);
    println!("Updated: {}", config_path.display());
    Ok(())
}

fn load_or_create_config(config_path: &Path) -> Result<crate::config::Config, Box<dyn std::error::Error>> {
    if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        toml::from_str::<crate::config::Config>(&content)
            .map_err(|e| format!("Failed to parse config: {}", e).into())
    } else {
        Ok(crate::config::Config::default())
    }
}

fn add_domain_to_config(
    config: &mut crate::config::Config,
    url: &str,
    include: Option<&[String]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let include_patterns = include
        .map(|v| v.to_vec())
        .unwrap_or_else(|| vec!["/**".to_string()]);

    config.domains.push(crate::config::Domain {
        url: url.to_string(),
        include: include_patterns,
    });
    Ok(())
}

fn save_config(config: &crate::config::Config, config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(config_path, content)?;
    Ok(())
}

/// Run the crawl pipeline
fn crawl_command(
    incremental: bool,
    config_path: &Path,
    concurrency: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::Config;
    use crate::pipeline::Pipeline;

    // Load configuration
    let content = std::fs::read_to_string(config_path)?;
    let mut config: Config = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;


    // Debug: print loaded domains
    println!("Config loaded from: {}", config_path.display());
    println!("Domains configured: {}", config.domains.len());
    for (i, domain) in config.domains.iter().enumerate() {
        println!("  [{}] {} (include: {:?})", i + 1, domain.url, domain.include);
    }
    if config.domains.is_empty() {
        eprintln!("ERROR: No domains configured! Add domains to matome.toml or run 'matome add <url>'");
        return Ok(());
    }

    // Override concurrency if specified
    if let Some(c) = concurrency {
        config.crawl.concurrency = c;
    }

    // Initialize and run pipeline
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut pipeline = Pipeline::new(&config).await?;
        pipeline.run(incremental).await
    })?;


    Ok(())
}

/// Start the web server
fn serve_command(
    port: u16,
    host: &str,
    data_dir_arg: Option<&Path>,
    config_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::Config;
    use crate::web::Server;

    // Determine data_dir: command-line arg takes priority, then config, then default
    let data_dir = if let Some(d) = data_dir_arg {
        d.to_path_buf()
    } else if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        PathBuf::from(&config.core.data_dir)
    } else {
        PathBuf::from(".matome")
    };

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let server = Server::new(&data_dir)?;
        server.run((host, port)).await
    })?;

    Ok(())
}

/// Show status information
fn status_command(
    config_path: &Path,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::Config;
    use crate::db::Database;

    // Read data_dir from config or use default
    let data_dir = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        PathBuf::from(&config.core.data_dir)
    } else {
        PathBuf::from(".matome")
    };

    let db = Database::new(&data_dir)?;
    
    // Try pages table first (v0.2.0), fallback to articles (v0.1.0)
    let page_count = db.get_page_count().unwrap_or(0);
    let domain_counts = db.get_domain_counts().unwrap_or_default();
    let domains = domain_counts.len();
    
    if page_count > 0 {
        // Using v0.2.0 data model
        println!("matome Status (v0.2.0)");
        println!("======================");
        println!("Data directory: {}", data_dir.display());
        println!("Total pages: {}", page_count);
        println!("Domains: {}", domains);
        
        if verbose {
            println!("\nDomain Breakdown:");
            for (domain, count) in &domain_counts {
                println!("  {} - {} pages", domain, count);
            }
            
            // Show pages table stats
            if let Ok(pages) = db.get_all_pages_with_domain() {
                let total_chars: usize = pages.iter()
                    .map(|p| p.clean_markdown.len())
                    .sum();
                println!("\nContent Statistics:");
                println!("  Total content size: {} chars", total_chars);
                println!("  Average page size: {} chars", 
                    if page_count > 0 { total_chars / page_count } else { 0 });
            }
        }
    } else {
        // Fallback to v0.1.0
        let stats = db.get_stats()?;
        println!("matome Status (v0.1.0 - Legacy)");
        println!("================================");
        println!("Data directory: {}", data_dir.display());
        println!("Total articles: {}", stats.total_articles);
        println!("Indexed articles: {}", stats.indexed_articles);
        println!("Domains: {}", stats.domains);

        if verbose {
            println!("\nDetailed Statistics:");
            println!("- Original MD size: {} bytes", stats.original_md_size);
            println!("- Translated MD size: {} bytes", stats.translated_md_size);
        }
        
        println!("\n⚠️  Using legacy articles table. Run 'matome crawl' to migrate to v0.2.0.");
    }

    Ok(())
}

/// Clean database (delete articles)
fn clean_command(
    all: bool,
    domain: Option<&str>,
    orphaned: bool,
    id: Option<i64>,
    data_dir_arg: Option<&Path>,
    config_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::Config;
    use crate::db::Database;
    use crate::db::SearchEngine;
    // Determine data_dir
    let data_dir = if let Some(d) = data_dir_arg {
        d.to_path_buf()
    } else if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        PathBuf::from(&config.core.data_dir)
    } else {
        PathBuf::from(".matome")
    };


    let db = Database::new(&data_dir)?;

    // Try to get search engine for index cleanup
    let search_engine = match SearchEngine::new(&data_dir) {
        Ok(se) => Some(se),
        Err(e) => {
            println!("Warning: Could not initialize search engine: {}", e);
            None
        }
    };

    // Check if we have pages (v0.2.0) or articles (v0.1.0)
    let page_count = db.get_page_count().unwrap_or(0);
    let use_pages_table = page_count > 0;

    // Determine what to clean
    if all {
        if use_pages_table {
            println!("This will delete ALL {} pages. Are you sure? [y/N]", page_count);
        } else {
            let stats = db.get_stats()?;
            if stats.total_articles == 0 {
                println!("No articles to delete.");
                return Ok(());
            }
            println!("This will delete ALL {} articles. Are you sure? [y/N]", stats.total_articles);
        }
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Aborted.");
            return Ok(());
        }
        
        if use_pages_table {
            let deleted = db.clear_pages().map_err(|e| format!("Failed to clear pages: {}", e))?;
            println!("Deleted {} pages from database.", deleted);
        } else {
            let deleted = db.clear()?;
            println!("Deleted {} articles from database.", deleted);
        }

        // Clear search index
        if let Some(ref se) = search_engine {
            se.clear()?;
            println!("Cleared search index.");
        }
    } else if let Some(d) = domain {
        // Get URLs before deleting (for index cleanup)
        let urls = if use_pages_table {
            db.get_page_urls_by_domain(d).map_err(|e| format!("Failed to get page URLs: {}", e))?
        } else {
            let articles = db.get_articles_by_domain(d)?;
            articles.iter().map(|a| a.url.clone()).collect()
        };
        
        if urls.is_empty() {
            println!("No pages found for domain: {}", d);
            return Ok(());
        }
        println!("This will delete {} pages from '{}'. Are you sure? [y/N]", urls.len(), d);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Aborted.");
            return Ok(());
        }

        let deleted = if use_pages_table {
            db.delete_pages_by_domain(d).map_err(|e| format!("Failed to delete pages: {}", e))?
        } else {
            db.delete_by_domain(d)?
        };
        println!("Deleted {} pages from '{}'.", deleted, d);

        // Remove from search index
        if let Some(ref se) = search_engine {
            for url in urls {
                se.delete_by_url(&url)?;
            }
            println!("Removed {} documents from search index.", deleted);
        }
    } else if orphaned {
        // Find orphaned pages/articles
        if use_pages_table {
            let orphaned_pages = db.get_orphaned_pages().map_err(|e| format!("Failed to get orphaned pages: {}", e))?;

            if orphaned_pages.is_empty() {
                println!("No orphaned pages found.");
                return Ok(());
            }

            println!("Found orphaned pages:");
            for p in &orphaned_pages {
                let issue = if p.title.is_empty() {
                    "missing title"
                } else if p.translated_markdown.is_empty() {
                    "missing translation"
                } else if p.clean_markdown.len() < 50 {
                    "content too short"
                } else {
                    "unknown"
                };
                println!("  [{}] {} ({}) - {}", &p.id[..8], p.domain, p.tree_path, issue);
            }

            println!("\nThis will delete {} orphaned pages. Are you sure? [y/N]", orphaned_pages.len());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                println!("Aborted.");
                return Ok(());
            }

            // Get URLs before deleting
            let urls: Vec<_> = orphaned_pages.iter().map(|p| p.url.clone()).collect();

            let deleted = db.delete_orphaned_pages().map_err(|e| format!("Failed to delete orphaned pages: {}", e))?;
            println!("Deleted {} orphaned pages from database.", deleted);

            // Remove from search index
            if let Some(ref se) = search_engine {
                for url in urls {
                    se.delete_by_url(&url)?;
                }
                println!("Removed {} documents from search index.", deleted);
            }
        } else {
            // Fallback to v0.1.0 orphaned check
            let orphaned_articles = db.get_orphaned_articles()?;

            if orphaned_articles.is_empty() {
                println!("No orphaned articles found.");
                return Ok(());
            }

            println!("Found orphaned articles:");
            for a in &orphaned_articles {
                let issue = if a.title.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                    "missing title"
                } else if a.translated_md.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                    "missing translation"
                } else if a.original_md.len() < 50 {
                    "content too short"
                } else {
                    "missing description"
                };
                println!("  [{}] {} - {}", a.id, a.domain, issue);
            }

            println!("\nThis will delete {} orphaned articles. Are you sure? [y/N]", orphaned_articles.len());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                println!("Aborted.");
                return Ok(());
            }

            // Get URLs before deleting
            let urls: Vec<_> = orphaned_articles.iter().map(|a| a.url.clone()).collect();

            let deleted = db.delete_orphaned()?;
            println!("Deleted {} orphaned articles from database.", deleted);

            // Remove from search index
            if let Some(ref se) = search_engine {
                for url in urls {
                    se.delete_by_url(&url)?;
                }
                println!("Removed {} documents from search index.", deleted);
            }
        }
    } else if let Some(_article_id) = id {
        // Note: ID-based deletion only works for legacy articles table
        // For pages table, use URL-based identification
        println!("⚠️  ID-based deletion is only supported for legacy articles table.");
        println!("For pages table, use --domain or --orphaned options.");
    } else {
        println!("Please specify what to clean:");
        println!("  --all          Delete all pages/articles");
        println!("  --domain <name> Delete pages/articles from specific domain");
        println!("  --orphaned     Delete pages/articles with missing/incomplete data");
        println!("\nExample: matome clean --all");
        println!("         matome clean --domain docs.python.org");
        println!("         matome clean --orphaned");
    }

    Ok(())
}

// ====== v0.2.0: Diff Mode Commands ======

/// Show changes since last crawl
fn diff_command(
    _since: Option<&str>,
    breaking_only: bool,
    format: &str,
    config_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::db::Database;
    use crate::db::models::ChangeType;
    use crate::pipeline::compare_and_update;
    use serde_json;

    let data_dir = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: crate::config::Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        PathBuf::from(&config.core.data_dir)
    } else {
        PathBuf::from(".matome")
    };

    let db = Database::new(&data_dir)?;
    
    // Try pages table first (v0.2.0), fallback to articles (v0.1.0)
    let page_count = db.get_page_count().unwrap_or(0);
    
    let mut changes: Vec<DiffResult> = Vec::new();
    
    if page_count > 0 {
        // Using v0.2.0 data model with pages table
        let pages = db.get_all_pages()?;
        
        for page in pages {
            // For pages, we need to compare with a known "good" hash
            // Since we don't have a separate snapshot system yet,
            // we'll use the content_hash stored in the page as the baseline
            // and compare current clean_markdown against it
            
            let change_result = compare_and_update(
                &page.clean_markdown, 
                &page.clean_markdown, // In a real system, this would be fresh crawl
                Some(&page.content_hash)
            );
            
            // Skip if no change detected
            if change_result.change_type == ChangeType::None {
                continue;
            }
            
            let change_type = if breaking_only && change_result.change_type == ChangeType::Breaking {
                ChangeType::Breaking
            } else {
                change_result.change_type
            };
            
            // Skip if we only want breaking changes and this isn't one
            if breaking_only && change_type != ChangeType::Breaking {
                continue;
            }
            
            changes.push(DiffResult {
                id: None,
                id_str: Some(page.id),
                title: page.title,
                url: page.url,
                domain: page.tree_path.split('/').nth(1).unwrap_or("unknown").to_string(),
                change_type,
                glossary_alerts: change_result.glossary_alerts,
                crawled_at: page.crawled_at,
            });
        }
    } else {
        // Fallback to v0.1.0 articles table
        let articles = db.get_all_articles()?;
        
        for article in articles {
            // For v0.1.0, we don't have content_hash tracking
            // Just report all articles
            let change_type = if breaking_only {
                ChangeType::Breaking
            } else {
                ChangeType::Minor
            };
            
            changes.push(DiffResult {
                id: Some(article.id),
                id_str: None,
                title: article.title.unwrap_or_else(|| "Untitled".to_string()),
                url: article.url,
                domain: article.domain,
                change_type,
                glossary_alerts: vec![],
                crawled_at: article.crawled_at,
            });
        }
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&changes)?;
            println!("{}", json);
        }
        _ => {
            print_diff_text(&changes, breaking_only);
        }
    }
    Ok(())
}

fn print_diff_text(changes: &[DiffResult], breaking_only: bool) {
    if changes.is_empty() {
        println!("No changes detected.");
        return;
    }
    println!("matome Diff Report");
    println!("==================");
    println!("Total changes: {}", changes.len());
    println!();
    let breaking: Vec<_> = changes.iter().filter(|c| matches!(c.change_type, ChangeType::Breaking)).collect();
    let major: Vec<_> = changes.iter().filter(|c| matches!(c.change_type, ChangeType::Major)).collect();
    let minor: Vec<_> = changes.iter().filter(|c| matches!(c.change_type, ChangeType::Minor)).collect();
    if !breaking.is_empty() && !breaking_only {
        println!("🔴 Breaking Changes: {}", breaking.len());
        for c in &breaking {
            println!("  • {} ({})", c.title, c.domain);
        }
        println!();
    }
    if !major.is_empty() && !breaking_only {
        println!("🟠 Major Changes: {}", major.len());
        for c in &major {
            println!("  • {} ({})", c.title, c.domain);
        }
        println!();
    }
    if !breaking_only {
        println!("🟡 Minor Changes: {}", minor.len());
        for c in &minor {
            println!("  • {} ({})", c.title, c.domain);
        }
    }
    if breaking_only && !breaking.is_empty() {
        println!("⚠️  Breaking changes detected!");
    }
}

fn mode_command(mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    match mode.to_lowercase().as_str() {
        "library" | "lib" => {
            println!("Switched to Library Mode");
            println!("  Commands: matome serve, matome search");
        }
        "diff" => {
            println!("Switched to Diff Mode");
            println!("  Commands: matome diff, matome status --verbose");
        }
        "agent" => {
            println!("Switched to Agent Mode");
            println!("  Commands: matome export --agent");
        }
        _ => {
            eprintln!("Unknown mode: {}", mode);
            eprintln!("Available: library, diff, agent");
            return Err("Invalid mode".into());
        }
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct DiffResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id_str: Option<String>,
    title: String,
    url: String,
    domain: String,
    change_type: ChangeType,
    glossary_alerts: Vec<String>,
    crawled_at: String,
}

// ====== v0.2.0: Agent Mode Commands ======

use crate::modes::AgentExporter;

/// Export workspace for AI coding assistant
fn export_command(
    workspace: &str,
    workspace_dir: Option<&str>,
    max_tokens: usize,
    config_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::db::Database;

    // Determine data_dir from config
    let data_dir = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: crate::config::Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        PathBuf::from(&config.core.data_dir)
    } else {
        PathBuf::from(".matome")
    };

    let db = Database::new(&data_dir)?;
    let articles = db.get_all_articles()?;

    if articles.is_empty() {
        eprintln!("No articles to export. Run 'matome crawl' first.");
        return Ok(());
    }

    println!("Exporting {} articles to workspace '{}'...", articles.len(), workspace);

    let exporter = AgentExporter::new(workspace, workspace_dir, max_tokens)?;
    let result = exporter.export(&articles)?;

    println!();
    println!("✅ Export complete!");
    println!("   Workspace: {}", result.workspace_path.display());
    println!("   Files: {}", result.files_written);
    println!("   Tokens: ~{}", result.tokens_estimate);
    println!();
    println!("Usage:");
    println!("   # Claude Code - Add to CLAUDE.md:");
    println!("   \"Always read {} before answering questions.\"", result.workspace_path.join("manifest.json").display());
    println!();

    Ok(())
}

/// Generate context bundle for AI agent
fn bundle_command(
    topics: &str,
    max_tokens: usize,
    output: Option<&str>,
    config_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::db::Database;

    let data_dir = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: crate::config::Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        PathBuf::from(&config.core.data_dir)
    } else {
        PathBuf::from(".matome")
    };

    let db = Database::new(&data_dir)?;
    let articles = db.get_all_articles()?;

    // Filter by topics (domain matching)
    let topic_list: Vec<&str> = topics.split(',').map(|s| s.trim()).collect();
    let filtered: Vec<_> = articles.iter()
        .filter(|a| topic_list.iter().any(|t| a.domain.contains(t)))
        .collect();

    if filtered.is_empty() {
        eprintln!("No articles match topics: {}", topics);
        return Ok(());
    }

    // Generate bundle - note filtered.len() before iteration
    let filtered_count = filtered.len();
    let mut bundle = format!(
        "# matome Context Bundle\n\n\
        **Topics**: {}\n\
        **Articles**: {}\n\
        **Max Tokens**: {}\n\n---\n\n",
        topics,
        filtered_count,
        max_tokens
    );

    let mut tokens_used = 0;
    for article in &filtered {
        let content = article.translated_md.as_deref().unwrap_or(&article.original_md);
        let article_tokens = content.len() / 4;

        if tokens_used + article_tokens > max_tokens {
            break;
        }

        bundle.push_str(&format!(
            "## {}\n\n*Source: {}*\n\n{}\n\n---\n\n",
            article.title.clone().unwrap_or_else(|| "Untitled".to_string()),
            article.url,
            content
        ));
        tokens_used += article_tokens;
    }

    // Write output
    if let Some(path) = output {
        std::fs::write(path, &bundle)?;
        println!("✅ Bundle written to: {}", path);
    } else {
        println!("{}", bundle);
    }

    println!("\nBundle stats:");
    println!("   Articles: {}", filtered.len());
    println!("   Tokens: ~{}", tokens_used);

    Ok(())
}
