//! Change detection and version recording
//!
//! Compares content hashes to detect changes and records version history.

use crate::db::models::{ChangeResult, ChangeType};
use crate::pipeline::content_hash::compute_content_hash;

/// Compare content and determine change type
///
/// Returns ChangeResult with:
/// - Whether content changed
/// - Change classification (None/Minor/Major/Breaking)
/// - Glossary alerts for priority terms
#[allow(dead_code)]
pub fn compare_and_update(
    old_content: &str,
    new_content: &str,
    old_hash: Option<&str>,
) -> ChangeResult {
    let new_hash = compute_content_hash(new_content);

    // If no previous hash, this is a new page
    let Some(old_hash) = old_hash else {
        return ChangeResult {
            change_type: ChangeType::None, // New page, not a change
            old_hash: String::new(),
            new_hash,
            glossary_alerts: vec![],
            diff_snippet: None,
        };
    };

    // Compare hashes
    if old_hash == new_hash {
        return ChangeResult {
            change_type: ChangeType::None,
            old_hash: old_hash.to_string(),
            new_hash,
            glossary_alerts: vec![],
            diff_snippet: None,
        };
    }

    // Content changed - classify the change
    let change_type = classify_change(old_content, new_content);
    let glossary_alerts = detect_glossary_changes(old_content, new_content);
    let diff_snippet = generate_diff_snippet(old_content, new_content);

    ChangeResult {
        change_type,
        old_hash: old_hash.to_string(),
        new_hash,
        glossary_alerts,
        diff_snippet,
    }
}

/// Classify the type of change based on content diff
#[allow(dead_code)]
fn classify_change(old_content: &str, new_content: &str) -> ChangeType {
    // Calculate basic statistics
    let old_lines = old_content.lines().count();
    let new_lines = new_content.lines().count();
    let old_words = old_content.split_whitespace().count();
    let new_words = new_content.split_whitespace().count();

    // Calculate size change ratio
    let size_ratio = if old_words > 0 {
        new_words as f64 / old_words as f64
    } else {
        1.0
    };

    // Major change: significant size difference (>30% change)
    if !(0.7..=1.3).contains(&size_ratio) {
        return ChangeType::Major;
    }

    // Line count change
    let line_ratio = if old_lines > 0 {
        new_lines as f64 / old_lines as f64
    } else {
        1.0
    };

    if !(0.7..=1.3).contains(&line_ratio) {
        return ChangeType::Major;
    }

    // Default to minor change (typo fixes, formatting, small edits)
    ChangeType::Minor
}

/// Detect glossary priority term changes
#[allow(dead_code)]
fn detect_glossary_changes(_old_content: &str, _new_content: &str) -> Vec<String> {
    // This is a placeholder - actual implementation would check against glossary
    // For now, return empty vec
    // TODO: Integrate with glossary.rs to check priority terms
    vec![]
}

/// Generate a short diff snippet showing key changes
#[allow(dead_code)]
fn generate_diff_snippet(old_content: &str, new_content: &str) -> Option<String> {
    // Simple approach: find first substantial difference
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    // Find first line that differs
    for (i, (old_line, new_line)) in old_lines.iter().zip(new_lines.iter()).enumerate() {
        if old_line != new_line {
            // Return context around the change
            let start = i.saturating_sub(1);
            let end = (i + 2).min(new_lines.len());
            let snippet = new_lines[start..end].join("\n");
            return Some(format!("Line {}: {}\n", i + 1, snippet));
        }
    }

    // If all lines matched up to one set, check remaining lines
    if old_lines.len() != new_lines.len() {
        return Some(format!(
            "Content length changed: {} lines → {} lines",
            old_lines.len(),
            new_lines.len()
        ));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_old_hash() {
        let result = compare_and_update("content", "new content", None);
        assert_eq!(result.change_type, ChangeType::None);
        assert!(result.old_hash.is_empty());
        assert!(!result.new_hash.is_empty());
    }

    #[test]
    fn test_unchanged_content() {
        let content = "Hello, World!";
        let hash = compute_content_hash(content);
        let result = compare_and_update(content, content, Some(&hash));
        assert_eq!(result.change_type, ChangeType::None);
        assert_eq!(result.old_hash, result.new_hash);
    }

    #[test]
    fn test_minor_change() {
        let old = "Hello, World!";
        let new = "Hello, World!!"; // Added exclamation
        let old_hash = compute_content_hash(old);
        let result = compare_and_update(old, new, Some(&old_hash));
        assert_eq!(result.change_type, ChangeType::Minor);
        assert_ne!(result.old_hash, result.new_hash);
    }

    #[test]
    fn test_major_change() {
        let old = "Short text";
        let new = "This is a much longer piece of text that has been significantly expanded to demonstrate the major change detection";
        let old_hash = compute_content_hash(old);
        let result = compare_and_update(old, new, Some(&old_hash));
        assert_eq!(result.change_type, ChangeType::Major);
    }

    #[test]
    fn test_diff_snippet() {
        let old = "Line 1\nLine 2\nLine 3\nLine 4";
        let new = "Line 1\nLine 2 changed\nLine 3\nLine 4";
        let old_hash = compute_content_hash(old);
        let result = compare_and_update(old, new, Some(&old_hash));
        assert!(result.diff_snippet.is_some());
        assert!(result.diff_snippet.unwrap().contains("Line 2"));
    }
}
