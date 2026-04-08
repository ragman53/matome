//! Configuration file loading and type definitions
//!
//! Handles parsing and validation of matome.toml configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Configuration error types
#[allow(dead_code)]
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
    #[serde(default, rename = "domain")]
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
    "translategemma:4b".to_string()
}

fn default_target_lang() -> String {
    "ja".to_string()
}

/// Supported target languages
pub const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("ja", "Japanese"),
    ("zh", "Chinese"),
    ("zh-CN", "Chinese (Simplified)"),
    ("zh-TW", "Chinese (Traditional)"),
    ("ko", "Korean"),
    ("es", "Spanish"),
    ("fr", "French"),
    ("de", "German"),
    ("pt", "Portuguese"),
    ("it", "Italian"),
    ("ru", "Russian"),
    ("ar", "Arabic"),
    ("hi", "Hindi"),
    ("th", "Thai"),
    ("vi", "Vietnamese"),
    ("id", "Indonesian"),
];

/// Get the human-readable language name for a language code
pub fn language_name(lang_code: &str) -> &'static str {
    SUPPORTED_LANGUAGES
        .iter()
        .find(|(code, _)| *code == lang_code)
        .map(|(_, name)| *name)
        .unwrap_or("Unknown")
}

/// Get the HTML lang attribute for a language code
#[allow(dead_code)]
pub fn html_lang(lang_code: &str) -> &str {
    match lang_code {
        "zh" | "zh-CN" => "zh-CN",
        "zh-TW" => "zh-TW",
        code => code,
    }
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

/// Glossary entry supporting multi-language translations
///
/// Each term has a source (`en`) and one or more target language translations.
/// Supports both legacy single-language format (`ja = "..."`) and
/// multi-language format (`translations = { ja = "...", zh = "..." }`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlossaryTerm {
    /// Source term (typically English)
    pub en: String,

    /// Legacy: Japanese translation only (backward compatible)
    /// Prefer using `translations` for multi-language support.
    #[serde(default)]
    pub ja: Option<String>,

    /// Multi-language translations (lang_code → translation)
    /// Example: `translations = { ja = "コンパイラ", zh = "编译器" }`
    #[serde(default)]
    pub translations: std::collections::HashMap<String, String>,
}

impl GlossaryTerm {
    /// Get the translation for a specific target language.
    /// Falls back to `ja` field for backward compatibility.
    pub fn get_translation(&self, lang: &str) -> Option<&str> {
        self.translations
            .get(lang)
            .map(|s| s.as_str())
            .or_else(|| self.ja.as_deref().filter(|_| lang == "ja"))
    }
}

/// Glossary configuration (duplicate - use pipeline/glossary.rs)
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Glossary {
    pub terms: Vec<GlossaryTerm>,
}

#[allow(dead_code)]
impl Glossary {
    /// Load glossary from file
    pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let glossary: Glossary = toml::from_str(&content)?;
        Ok(glossary)
    }

    /// Apply glossary replacements to text for a specific target language
    pub fn apply_for_lang(&self, text: &str, lang: &str) -> String {
        let mut result = text.to_string();
        for term in &self.terms {
            if let Some(translation) = term.get_translation(lang) {
                let pattern =
                    regex_lite::Regex::new(&format!("(?i)\\b{}\\b", regex_lite::escape(&term.en)))
                        .unwrap_or_else(|_| regex_lite::Regex::new("").unwrap());
                result = pattern.replace_all(&result, translation).to_string();
            }
        }
        result
    }

    /// Apply glossary replacements to text (defaults to Japanese for backward compatibility)
    pub fn apply(&self, text: &str) -> String {
        self.apply_for_lang(text, "ja")
    }
}

impl Default for Glossary {
    fn default() -> Self {
        Self { terms: Vec::new() }
    }
}

/// Article metadata stored in database
#[allow(dead_code)]
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
    fn test_glossary_apply_legacy_ja() {
        let glossary = Glossary {
            terms: vec![
                GlossaryTerm {
                    en: "compiler".to_string(),
                    ja: Some("コンパイラ".to_string()),
                    translations: std::collections::HashMap::new(),
                },
                GlossaryTerm {
                    en: "runtime".to_string(),
                    ja: Some("ランタイム".to_string()),
                    translations: std::collections::HashMap::new(),
                },
            ],
        };

        let result = glossary.apply("The compiler handles runtime errors.");
        assert!(result.contains("コンパイラ"));
        assert!(result.contains("ランタイム"));
    }

    #[test]
    fn test_glossary_apply_multilang() {
        let mut translations = std::collections::HashMap::new();
        translations.insert("ja".to_string(), "コンパイラ".to_string());
        translations.insert("zh".to_string(), "编译器".to_string());
        translations.insert("ko".to_string(), "컴파일러".to_string());

        let glossary = Glossary {
            terms: vec![GlossaryTerm {
                en: "compiler".to_string(),
                ja: None,
                translations,
            }],
        };

        // Japanese
        let result_ja = glossary.apply_for_lang("The compiler is fast.", "ja");
        assert!(result_ja.contains("コンパイラ"));

        // Chinese
        let result_zh = glossary.apply_for_lang("The compiler is fast.", "zh");
        assert!(result_zh.contains("编译器"));

        // Korean
        let result_ko = glossary.apply_for_lang("The compiler is fast.", "ko");
        assert!(result_ko.contains("컴파일러"));

        // Unknown language — no replacement
        let result_fr = glossary.apply_for_lang("The compiler is fast.", "fr");
        assert!(result_fr.contains("compiler"));
    }

    #[test]
    fn test_language_helpers() {
        assert_eq!(language_name("ja"), "Japanese");
        assert_eq!(language_name("zh"), "Chinese");
        assert_eq!(language_name("ko"), "Korean");
        assert_eq!(language_name("xx"), "Unknown");

        assert_eq!(html_lang("ja"), "ja");
        assert_eq!(html_lang("zh"), "zh-CN");
        assert_eq!(html_lang("zh-TW"), "zh-TW");
    }
}
