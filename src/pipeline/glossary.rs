//! Glossary module
//!
//! Handles glossary parsing and term replacement.

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

/// Glossary for term replacement
#[derive(Clone)]
pub struct Glossary {
    terms: Arc<HashMap<String, String>>,
}

impl Glossary {
    /// Load glossary from file
    pub fn load(path: &PathBuf) -> Result<Self, GlossaryError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config: GlossaryConfig = toml::from_str(&content)?;

        let mut terms = HashMap::new();
        for term in config.terms {
            terms.insert(term.en.to_lowercase(), term.ja);
        }

        Ok(Self {
            terms: Arc::new(terms),
        })
    }

    /// Create glossary from terms
    pub fn from_terms(terms: Vec<GlossaryTerm>) -> Self {
        let mut term_map = HashMap::new();
        for term in terms {
            term_map.insert(term.en.to_lowercase(), term.ja);
        }
        Self {
            terms: Arc::new(term_map),
        }
    }

    /// Apply glossary replacements to text
    pub fn apply(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Simple word boundary replacement
        for (en, ja) in self.terms.iter() {
            // Case-insensitive word boundary replacement
            let pattern = regex_lite::Regex::new(&format!(r"(?i)\b{}\b", regex_lite::escape(en)))
                .unwrap_or_else(|_| regex_lite::Regex::new("").unwrap());

            result = pattern.replace_all(&result, ja).to_string();
        }

        result
    }

    /// Get replacement for a term
    pub fn get(&self, term: &str) -> Option<&String> {
        self.terms.get(&term.to_lowercase())
    }
}

impl Default for Glossary {
    fn default() -> Self {
        Self {
            terms: Arc::new(HashMap::new()),
        }
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
    fn test_glossary_apply() {
        let glossary = Glossary::from_terms(vec![
            GlossaryTerm {
                en: "compiler".to_string(),
                ja: "コンパイラ".to_string(),
            },
            GlossaryTerm {
                en: "runtime".to_string(),
                ja: "ランタイム".to_string(),
            },
        ]);

        let result = glossary.apply("The compiler handles runtime errors.");
        assert!(result.contains("コンパイラ"));
        assert!(result.contains("ランタイム"));
    }

    #[test]
    fn test_glossary_case_insensitive() {
        let glossary = Glossary::from_terms(vec![GlossaryTerm {
            en: "API".to_string(),
            ja: "API".to_string(),
        }]);

        let result = glossary.apply("Use the api for testing.");
        assert!(result.contains("API"));
    }
}
