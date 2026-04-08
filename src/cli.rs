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
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Host to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
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
            Command::Serve { port, host, data_dir } => {
                serve_command(*port, host, data_dir.as_ref())?;
            }
            Command::Status {
                config,
                verbose,
            } => {
                status_command(config, *verbose)?;
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
    use std::fs;

    let mut config = if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        toml::from_str::<crate::config::Config>(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?
    } else {
        // Create new config with default values
        crate::config::Config::default()
    };

    // Add new domain
    let include_patterns = include
        .map(|v| v.to_vec())
        .unwrap_or_else(|| vec!["/**".to_string()]);

    config.domains.push(crate::config::Domain {
        url: url.to_string(),
        include: include_patterns,
    });

    // Write back to file
    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(config_path, content)?;

    println!("Added domain: {}", url);
    println!("Updated: {}", config_path.display());

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
    data_dir: Option<&PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::web::Server;

    let data_dir = data_dir
        .map(|p| p.clone())
        .unwrap_or_else(|| PathBuf::from(".matome"));

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let server = Server::new(&data_dir)?;
        server.run((host, port)).await
    })?;

    Ok(())
}

/// Show status information
fn status_command(
    _config_path: &PathBuf,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::db::Database;

    let data_dir = PathBuf::from(".matome");

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
