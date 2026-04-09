//! Token counter using tiktoken for accurate AI context window estimation
//!
//! Uses OpenAI's cl100k_base encoding (same as GPT-4, Claude, etc.)

use thiserror::Error;
use tiktoken_rs::{cl100k_base, CoreBPE};

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("TikToken initialization failed: {0}")]
    InitError(String),
    #[error("Token encoding failed: {0}")]
    EncodingError(String),
}

/// Token counter using tiktoken-rs
///
/// Provides accurate token counting for AI context windows using the cl100k_base
/// encoding (used by GPT-4, Claude 3, and many other modern AI models).
pub struct TokenCounter {
    encoder: CoreBPE,
}

impl TokenCounter {
    /// Create a new token counter
    pub fn new() -> Result<Self, TokenError> {
        let encoder = cl100k_base().map_err(|e| TokenError::InitError(e.to_string()))?;
        Ok(Self { encoder })
    }

    /// Count tokens in a string
    pub fn count(&self, text: &str) -> usize {
        self.encoder.encode_ordinary(text).len()
    }

    /// Count tokens with special tokens (for chat formats)
    pub fn count_with_special(&self, text: &str) -> usize {
        use std::collections::HashSet;
        self.encoder.encode(text, HashSet::new()).len()
    }

    /// Estimate tokens from file content
    pub fn count_file(&self, content: &str) -> usize {
        self.count(content)
    }

    /// Count tokens in multiple strings
    pub fn count_batch(&self, texts: &[&str]) -> usize {
        texts.iter().map(|t| self.count(t)).sum()
    }

    /// Check if content fits within a token budget
    pub fn fits_in_budget(&self, content: &str, budget: usize) -> bool {
        self.count(content) <= budget
    }

    /// Get remaining tokens after content
    pub fn remaining(&self, content: &str, total_budget: usize) -> usize {
        total_budget.saturating_sub(self.count(content))
    }

    /// Split text to fit within token budget (approximate)
    pub fn split_to_fit(&self, text: &str, max_tokens: usize) -> Vec<String> {
        let tokens = self.encoder.encode_ordinary(text);
        if tokens.len() <= max_tokens {
            return vec![text.to_string()];
        }

        let chars_per_token = text.len() / tokens.len();
        let _approx_chars = max_tokens * chars_per_token;

        // Split by lines first
        let lines: Vec<&str> = text.lines().collect();
        let mut chunks: Vec<String> = Vec::new();
        let mut current_chunk = String::new();
        let mut current_tokens = 0;

        for line in lines {
            let line_tokens = self.count(line);
            if current_tokens + line_tokens > max_tokens && !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
                current_tokens = 0;
            }
            current_chunk.push_str(line);
            current_chunk.push('\n');
            current_tokens += line_tokens;
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new().expect("Failed to initialize token counter")
    }
}

impl TokenCounter {
    /// Create a fallback counter that uses character-based estimation
    /// (1 token ≈ 4 characters - less accurate but always works)
    pub fn fallback() -> Self {
        // Return a wrapper that uses the fallback method
        // This is a dummy encoder that we'll handle specially
        todo!("Use character-based fallback")
    }
}

/// Fallback token counter using character ratio (less accurate)
pub struct FallbackTokenCounter;

impl FallbackTokenCounter {
    /// Estimate tokens using character ratio (1 token ≈ 4 chars)
    pub fn count(&self, text: &str) -> usize {
        text.chars().count() / 4
    }

    /// Create new fallback counter
    pub fn new() -> Self {
        Self
    }
}

impl Default for FallbackTokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Context budget calculator for AI agents
#[derive(Debug, Clone)]
pub struct ContextBudget {
    pub total_limit: usize,
    pub system_tokens: usize,
    pub reserved_tokens: usize,
}

impl ContextBudget {
    pub fn new(total_limit: usize) -> Self {
        Self {
            total_limit,
            system_tokens: 500,   // Reserve for system prompts
            reserved_tokens: 100, // Reserve for formatting
        }
    }

    /// Available tokens for user content
    pub fn available(&self) -> usize {
        self.total_limit
            .saturating_sub(self.system_tokens)
            .saturating_sub(self.reserved_tokens)
    }

    /// Check if content fits
    pub fn fits(&self, tokens: usize) -> bool {
        tokens <= self.available()
    }

    /// Remaining tokens
    pub fn remaining(&self, used: usize) -> usize {
        self.available().saturating_sub(used)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counter() {
        let counter = TokenCounter::new().unwrap();

        // Test basic counting
        let text = "Hello, world!";
        let tokens = counter.count(text);
        assert!(tokens > 0, "Should count some tokens");

        // Test empty string
        assert_eq!(counter.count(""), 0);

        // Test code blocks
        let code = "fn main() { println!(\"hello\"); }";
        let code_tokens = counter.count(code);
        assert!(code_tokens > 0);
    }

    #[test]
    fn test_context_budget() {
        let budget = ContextBudget::new(128000);

        assert_eq!(budget.total_limit, 128000);
        assert!(budget.available() < 128000); // Subtract system/reserved
        assert!(budget.fits(1000));
        assert!(!budget.fits(200000));
    }
}
