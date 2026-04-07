//! Configuration file loading and type definitions
//!
//! Handles parsing and validation of matome.toml configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Data directory not found: {0}")]
    DataDirNotFound(PathBuf),
}

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Core settings
    #[serde(default)]
    pub core: CoreConfig,

    /// Domains to crawl
    #[serde(default)]
    pub domains: Vec<Domain>,

    /// Translation settings
    #[serde(default)]
    pub translate: TranslateConfig,

    /// Crawler settings
    #[serde(default)]
    pub crawl: CrawlConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            domains: Vec::new(),
            translate: TranslateConfig::default(),
            crawl: CrawlConfig::default(),
        }
    }
}

/// Core configuration settings
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CoreConfig {
    /// Directory for storing data (DB, index, etc.)
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
        }
    }
}

fn default_data_dir() -> String {
    ".matome".to_string()
}

/// Domain configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Domain {
    /// Base URL of the domain
    pub url: String,

    /// URL patterns to include (glob patterns)
    #[serde(default)]
    pub include: Vec<String>,
}

impl Domain {
    /// Get the domain name from URL
    pub fn name(&self) -> String {
        url::Url::parse(&self.url)
            .map(|u| u.host_str().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

/// Translation provider configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct TranslateConfig {
    /// Translation provider: "ollama", "deepl", "libretranslate"
    #[serde(default = "default_provider")]
    pub provider: String,

    /// Model name to use
    #[serde(default = "default_model")]
    pub model: String,

    /// Target language code
    #[serde(default = "default_target_lang")]
    pub target_lang: String,

    /// Glossary file path
    #[serde(default)]
    pub glossary_file: Option<String>,
}

impl Default for TranslateConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
            target_lang: default_target_lang(),
            glossary_file: None,
        }
    }
}

fn default_provider() -> String {
    "ollama".to_string()
}

fn default_model() -> String {
    "translategemma:latest".to_string()
}

fn default_target_lang() -> String {
    "ja".to_string()
}

/// Crawler configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CrawlConfig {
    /// Number of concurrent requests
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    /// Whether to respect robots.txt
    #[serde(default = "default_respect_robots")]
    pub respect_robots: bool,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum pages to crawl (0 = unlimited)
    #[serde(default)]
    pub max_pages: usize,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            concurrency: default_concurrency(),
            respect_robots: default_respect_robots(),
            timeout: default_timeout(),
            max_pages: 0,
        }
    }
}

fn default_concurrency() -> usize {
    8
}

fn default_respect_robots() -> bool {
    true
}

fn default_timeout() -> u64 {
    30
}

/// Glossary entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlossaryTerm {
    pub en: String,
    pub ja: String,
}

/// Glossary configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Glossary {
    pub terms: Vec<GlossaryTerm>,
}

impl Glossary {
    /// Load glossary from file
    pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let glossary: Glossary = toml::from_str(&content)?;
        Ok(glossary)
    }

    /// Apply glossary replacements to text
    pub fn apply(&self, text: &str) -> String {
        let mut result = text.to_string();
        for term in &self.terms {
            // Case-insensitive replacement
            let pattern =
                regex_lite::Regex::new(&format!("(?i)\\b{}\\b", regex_lite::escape(&term.en)))
                    .unwrap_or_else(|_| regex_lite::Regex::new("").unwrap());
            result = pattern.replace_all(&result, &term.ja).to_string();
        }
        result
    }
}

impl Default for Glossary {
    fn default() -> Self {
        Self { terms: Vec::new() }
    }
}

/// Article metadata stored in database
#[derive(Debug, Clone)]
pub struct Article {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub original_md: String,
    pub translated_md: Option<String>,
    pub domain: String,
    pub crawled_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.core.data_dir, ".matome");
        assert!(config.domains.is_empty());
    }

    #[test]
    fn test_domain_name() {
        let domain = Domain {
            url: "https://docs.rust-lang.org".to_string(),
            include: vec!["/**".to_string()],
        };
        assert_eq!(domain.name(), "docs.rust-lang.org");
    }

    #[test]
    fn test_glossary_apply() {
        let glossary = Glossary {
            terms: vec![
                GlossaryTerm {
                    en: "compiler".to_string(),
                    ja: "コンパイラ".to_string(),
                },
                GlossaryTerm {
                    en: "runtime".to_string(),
                    ja: "ランタイム".to_string(),
                },
            ],
        };

        let result = glossary.apply("The compiler handles runtime errors.");
        assert!(result.contains("コンパイラ"));
        assert!(result.contains("ランタイム"));
    }
}
