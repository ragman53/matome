//! Glossary module
//!
//! Handles glossary parsing and term replacement with multi-language support.

use crate::config::GlossaryTerm;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlossaryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

/// Glossary for term replacement, cached per language
#[derive(Clone, Default)]
pub struct Glossary {
    /// Raw terms from the glossary file
    terms: Arc<Vec<GlossaryTerm>>,
}

impl Glossary {
    /// Load glossary from file
    pub fn load(path: &PathBuf) -> Result<Self, GlossaryError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config: GlossaryConfig = toml::from_str(&content)?;

        Ok(Self {
            terms: Arc::new(config.terms),
        })
    }

    /// Create glossary from terms
    pub fn from_terms(terms: Vec<GlossaryTerm>) -> Self {
        Self {
            terms: Arc::new(terms),
        }
    }

    /// Apply glossary replacements to text for a specific target language
    pub fn apply_for_lang(&self, text: &str, lang: &str) -> String {
        let mut result = text.to_string();

        for term in self.terms.iter() {
            if let Some(translation) = term.get_translation(lang) {
                let pattern =
                    regex_lite::Regex::new(&format!(r"(?i)\b{}\b", regex_lite::escape(&term.en)))
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

    /// Get replacement for a term in a specific language
    pub fn get(&self, term: &str, lang: &str) -> Option<String> {
        let term_lower = term.to_lowercase();
        self.terms
            .iter()
            .find(|t| t.en.to_lowercase() == term_lower)
            .and_then(|t| t.get_translation(lang).map(|s| s.to_string()))
    }

    /// Check if glossary has any terms
    pub fn has_terms(&self) -> bool {
        !self.terms.is_empty()
    }

    /// Get the number of terms in the glossary
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }
}

#[derive(serde::Deserialize)]
struct GlossaryConfig {
    terms: Vec<GlossaryTerm>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossary_apply_legacy_ja() {
        let glossary = Glossary::from_terms(vec![
            GlossaryTerm {
                en: "compiler".to_string(),
                ja: Some("コンパイラ".to_string()),
                translations: HashMap::new(),
            },
            GlossaryTerm {
                en: "runtime".to_string(),
                ja: Some("ランタイム".to_string()),
                translations: HashMap::new(),
            },
        ]);

        let result = glossary.apply("The compiler handles runtime errors.");
        assert!(result.contains("コンパイラ"));
        assert!(result.contains("ランタイム"));
    }

    #[test]
    fn test_glossary_apply_multilang() {
        let mut trans = HashMap::new();
        trans.insert("ja".to_string(), "コンパイラ".to_string());
        trans.insert("zh".to_string(), "编译器".to_string());
        trans.insert("ko".to_string(), "컴파일러".to_string());

        let glossary = Glossary::from_terms(vec![GlossaryTerm {
            en: "compiler".to_string(),
            ja: None,
            translations: trans,
        }]);

        let result_ja = glossary.apply_for_lang("The compiler is fast.", "ja");
        assert!(result_ja.contains("コンパイラ"));

        let result_zh = glossary.apply_for_lang("The compiler is fast.", "zh");
        assert!(result_zh.contains("编译器"));

        let result_ko = glossary.apply_for_lang("The compiler is fast.", "ko");
        assert!(result_ko.contains("컴파일러"));

        // Unknown language — no replacement
        let result_fr = glossary.apply_for_lang("The compiler is fast.", "fr");
        assert!(result_fr.contains("compiler"));
    }

    #[test]
    fn test_glossary_case_insensitive() {
        let mut trans = HashMap::new();
        trans.insert("ja".to_string(), "API".to_string());

        let glossary = Glossary::from_terms(vec![GlossaryTerm {
            en: "API".to_string(),
            ja: None,
            translations: trans,
        }]);

        let result = glossary.apply("Use the api for testing.");
        assert!(result.contains("API"));
    }
}
