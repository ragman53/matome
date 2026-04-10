//! Glossary module
//!
//! Handles glossary parsing and term replacement with multi-language support.

use crate::config::GlossaryTerm;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlossaryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

/// Fallback regex pattern for empty term (matches nothing)
/// Using const ensures the pattern is pre-compiled and won't panic
const EMPTY_PATTERN: &str = r"(?!)";

/// Create regex pattern for word boundary matching (case-insensitive)
fn create_term_pattern(term: &str) -> regex_lite::Regex {
    // Use unwrap_or with a const fallback that never panics
    // The empty pattern (?! ) is a negative lookahead that never matches
    regex_lite::Regex::new(&format!(r"(?i)\b{}\b", regex_lite::escape(term))).unwrap_or_else(|_| {
        regex_lite::Regex::new(EMPTY_PATTERN).expect("EMPTY_PATTERN is always valid")
    })
}

/// Replace all occurrences of a term in text
fn replace_term(text: &str, term: &str, replacement: &str) -> String {
    let pattern = create_term_pattern(term);
    pattern.replace_all(text, replacement).to_string()
}

/// Glossary for term replacement, cached per language
#[derive(Clone, Default)]
pub struct Glossary {
    /// Raw terms from the glossary file
    terms: Arc<Vec<GlossaryTerm>>,
}

impl Glossary {
    /// Load glossary from file
    pub fn load(path: &Path) -> Result<Self, GlossaryError> {
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
    #[allow(dead_code)]
    pub fn from_terms(terms: Vec<GlossaryTerm>) -> Self {
        Self {
            terms: Arc::new(terms),
        }
    }

    /// Apply glossary replacements to text for a specific target language
    pub fn apply_for_lang(&self, text: &str, lang: &str) -> String {
        self.terms.iter().fold(text.to_string(), |result, term| {
            term.get_translation(lang)
                .map(|t| replace_term(&result, &term.en, t))
                .unwrap_or(result)
        })
    }

    /// Apply glossary replacements to text (defaults to Japanese for backward compatibility)
    #[allow(dead_code)]
    pub fn apply(&self, text: &str) -> String {
        self.apply_for_lang(text, "ja")
    }

    /// Get replacement for a term in a specific language
    #[allow(dead_code)]
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
    use std::collections::HashMap;

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

        assert!(glossary
            .apply_for_lang("The compiler is fast.", "ja")
            .contains("コンパイラ"));
        assert!(glossary
            .apply_for_lang("The compiler is fast.", "zh")
            .contains("编译器"));
        assert!(glossary
            .apply_for_lang("The compiler is fast.", "ko")
            .contains("컴파일러"));
        assert!(!glossary
            .apply_for_lang("The compiler is fast.", "fr")
            .contains("コンパイラ"));
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
        assert!(glossary.apply("Use the api for testing.").contains("API"));
    }
}
