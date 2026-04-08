//! CLI argument definitions using clap
//!
//! Defines all command-line interface commands and arguments.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
                serve_command(*port, host, data_dir.as_ref(), config)?;
            }
            Command::Status {
                config,
                verbose,
            } => {
                status_command(config, *verbose)?;
            }
            Command::Clean { all, domain, orphaned, id, config, data_dir } => {
                clean_command(*all, domain.as_deref(), *orphaned, *id, data_dir.as_ref(), config)?;
            }
        }
        Ok(())
    }
}

/// Initialize configuration files
fn init_command(output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
    config_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_or_create_config(config_path)?;
    add_domain_to_config(&mut config, url, include)?;
    save_config(&config, config_path)?;
    println!("Added domain: {}", url);
    println!("Updated: {}", config_path.display());
    Ok(())
}

fn load_or_create_config(config_path: &PathBuf) -> Result<crate::config::Config, Box<dyn std::error::Error>> {
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

fn save_config(config: &crate::config::Config, config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(config_path, content)?;
    Ok(())
}

/// Run the crawl pipeline
fn crawl_command(
    incremental: bool,
    config_path: &PathBuf,
    concurrency: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::Config;
    use crate::pipeline::Pipeline;

    // Load configuration
    let content = std::fs::read_to_string(config_path)?;
    let mut config: Config = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

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
    data_dir_arg: Option<&PathBuf>,
    config_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::Config;
    use crate::web::Server;

    // Determine data_dir: command-line arg takes priority, then config, then default
    let data_dir = if let Some(d) = data_dir_arg {
        d.clone()
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
    config_path: &PathBuf,
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
    let stats = db.get_stats()?;

    println!("matome Status");
    println!("=============");
    println!("Data directory: {}", data_dir.display());
    println!("Total articles: {}", stats.total_articles);
    println!("Indexed articles: {}", stats.indexed_articles);
    println!("Domains: {}", stats.domains);

    if verbose {
        println!("\nDetailed Statistics:");
        println!("- Original MD size: {} bytes", stats.original_md_size);
        println!("- Translated MD size: {} bytes", stats.translated_md_size);
    }

    Ok(())
}

/// Clean database (delete articles)
fn clean_command(
    all: bool,
    domain: Option<&str>,
    orphaned: bool,
    id: Option<i64>,
    data_dir_arg: Option<&PathBuf>,
    config_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::Config;
    use crate::db::Database;
    use crate::db::SearchEngine;
    // Determine data_dir
    let data_dir = if let Some(d) = data_dir_arg {
        d.clone()
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

    // Determine what to clean
    if all {
        let stats = db.get_stats()?;
        if stats.total_articles == 0 {
            println!("No articles to delete.");
            return Ok(());
        }
        println!("This will delete ALL {} articles. Are you sure? [y/N]", stats.total_articles);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Aborted.");
            return Ok(());
        }
        let deleted = db.clear()?;
        println!("Deleted {} articles from database.", deleted);
        
        // Clear search index
        if let Some(ref se) = search_engine {
            se.clear()?;
            println!("Cleared search index.");
        }
    } else if let Some(d) = domain {
        let articles = db.get_articles_by_domain(d)?;
        if articles.is_empty() {
            println!("No articles found for domain: {}", d);
            return Ok(());
        }
        println!("This will delete {} articles from '{}'. Are you sure? [y/N]", articles.len(), d);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Aborted.");
            return Ok(());
        }
        
        // Get URLs before deleting (for index cleanup)
        let urls: Vec<_> = articles.iter().map(|a| a.url.clone()).collect();
        
        let deleted = db.delete_by_domain(d)?;
        println!("Deleted {} articles from '{}'.", deleted, d);
        
        // Remove from search index
        if let Some(ref se) = search_engine {
            for url in urls {
                se.delete_by_url(&url)?;
            }
            println!("Removed {} documents from search index.", deleted);
        }
    } else if orphaned {
        // Find orphaned articles
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
    } else if let Some(article_id) = id {
        // Get URL before deleting
        let article = db.get_article(article_id)?;
        let url = article.as_ref().map(|a| a.url.clone());
        
        if db.delete_article(article_id)? {
            println!("Deleted article {} from database.", article_id);
            
            // Remove from search index
            if let (Some(ref se), Some(ref article_url)) = (search_engine, url) {
                se.delete_by_url(article_url)?;
                println!("Removed document from search index.");
            }
        } else {
            println!("Article {} not found.", article_id);
        }
    } else {
        println!("Please specify what to clean:");
        println!("  --all          Delete all articles");
        println!("  --domain <name> Delete articles from specific domain");
        println!("  --orphaned     Delete articles with missing/incomplete data");
        println!("  --id <id>      Delete specific article by ID");
        println!("\nExample: matome clean --all");
        println!("         matome clean --domain developer.mozilla.org");
        println!("         matome clean --orphaned");
        println!("         matome clean --id 123");
    }

    Ok(())
}
