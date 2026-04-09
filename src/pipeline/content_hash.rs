//! Content hashing for version tracking
//!
//! Provides SHA-256 based change detection.

use sha2::{Digest, Sha256};

/// Compute content hash for change detection
///
/// The content is normalized before hashing to ignore insignificant differences:
/// - Whitespace normalization
/// - Line ending unification
/// - Removal of certain HTML artifacts
pub fn compute_content_hash(content: &str) -> String {
    let normalized = normalize_for_comparison(content);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Normalize content for comparison
///
/// Removes insignificant differences while preserving semantic content:
/// - Collapses multiple whitespace to single space
/// - Normalizes line endings to \n
/// - Removes trailing whitespace from lines
fn normalize_for_comparison(content: &str) -> String {
    content
        // Normalize line endings
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        // Split by whitespace, filter empty, rejoin with single space
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_hash() {
        let hash1 = compute_content_hash("Hello, World!");
        let hash2 = compute_content_hash("Hello, World!");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_whitespace_normalization() {
        // Multiple spaces should be treated as one
        let hash1 = compute_content_hash("Hello,    World!");
        let hash2 = compute_content_hash("Hello, World!");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_line_ending_normalization() {
        // CRLF and LF should be equivalent
        let hash1 = compute_content_hash("Line1\r\nLine2");
        let hash2 = compute_content_hash("Line1\nLine2");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_content() {
        let hash1 = compute_content_hash("Hello");
        let hash2 = compute_content_hash("World");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_format() {
        let hash = compute_content_hash("test");
        // SHA-256 produces 64 character hex string
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
