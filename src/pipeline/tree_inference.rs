//! Tree path inference from URLs
//!
//! Extracts hierarchical structure from URL patterns to rebuild document tree.

use url::Url;

/// Infer tree_path from URL
///
/// Examples:
/// - "https://docs.rs/tokio/rt@1.38.0/task/index.html" → "/rt/task"
/// - "https://docs.rust-lang.org/book/ch04-01-understanding-ownership.html" → "/book/ch04-01-understanding-ownership"
/// - "https://kubernetes.io/docs/concepts/overview/what-is-kubernetes/" → "/docs/concepts/overview/what-is-kubernetes"
pub fn infer_tree_path(url: &str, base_url: &str) -> String {
    // Parse URLs - keep original for fallback
    let url_str = url.to_string();
    let url = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return fallback_path(&url_str),
    };
    let base = match Url::parse(base_url) {
        Ok(b) => b,
        Err(_) => return fallback_path(&url_str),
    };

    // Get path relative to base
    let relative = match url.path().strip_prefix(base.path()) {
        Some(p) => p,
        None => url.path(),
    };

    // Clean up path
    let path = relative
        // Remove index.html and similar
        .trim_end_matches("/index.html")
        .trim_end_matches("/index")
        .trim_end_matches("/index.htm")
        // Remove trailing slashes
        .trim_end_matches('/')
        // Remove file extensions
        .trim_end_matches(".html")
        .trim_end_matches(".htm")
        .to_string();

    // If path is empty, use root
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }

    // Normalize: remove version patterns like "@1.38.0" or "v1.2.3"
    let normalized = normalize_version_segments(&path);

    normalized
}

/// Normalize version patterns in paths
/// "/rt@1.38.0/task" → "/rt/task"
/// "/api/v1.2.3/endpoint" → "/api/v1.2.3/endpoint"
fn normalize_version_segments(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').collect();
    let normalized: Vec<String> = segments
        .iter()
        .map(|segment| {
            // Remove @version patterns (common in docs.rs like /rt@1.38.0/)
            if let Some(at_pos) = segment.find('@') {
                if at_pos > 0 {
                    // "rt@1.38.0" → "rt" if what follows looks like version
                    let version_part = &segment[at_pos + 1..];
                    if looks_like_version(version_part) {
                        return segment[..at_pos].to_string();
                    }
                }
            }
            segment.to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();

    let result = format!("/{}", normalized.join("/"));
    result
}

/// Check if a string looks like a version number
fn looks_like_version(s: &str) -> bool {
    // Check common version patterns: v1.0.0, 1.0.0, 1.38.0, 2024.01.15
    s.chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == 'v')
        && s.contains('.')
}

/// Infer breadcrumbs from tree_path
///
/// Examples:
/// - "/rt/task" → ["Runtime", "Task"] (via title_case)
/// - "/api/v2/auth" → ["API", "V2", "Auth"]
pub fn infer_breadcrumbs(tree_path: &str) -> Vec<String> {
    if tree_path == "/" || tree_path.is_empty() {
        return vec![];
    }

    tree_path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(title_case)
        .collect()
}

/// Convert a path segment to Title Case
/// "getting-started" → "Getting Started"
/// "asyncRuntime" → "Asyncruntime"
/// "API" → "API" (preserve acronyms)
fn title_case(s: &str) -> String {
    // Handle special cases first
    let lower = s.to_lowercase();

    // Common acronyms to preserve
    let acronyms = [
        "api", "html", "css", "xml", "json", "sql", "http", "https", "url", "uri",
    ];
    if acronyms.contains(&lower.as_str()) {
        return s.to_uppercase();
    }

    // Convert hyphen/underscore separated to title case
    s.split(|c| c == '-' || c == '_')
        .map(word_to_title_case)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert a single word to title case
fn word_to_title_case(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    let mut capitalize_next = true;

    for c in word.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        } else {
            // Preserve separators, reset capitalize
            result.push(c);
            capitalize_next = true;
        }
    }

    result
}

/// Fallback path generation for invalid URLs
fn fallback_path(url: &str) -> String {
    // Try to extract something meaningful
    let path = url.split('?').next().unwrap_or(url);
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if segments.is_empty() {
        "/".to_string()
    } else {
        // Use last few segments
        let relevant: Vec<&str> = segments.iter().rev().take(3).cloned().collect();
        format!(
            "/{}",
            relevant.iter().rev().copied().collect::<Vec<_>>().join("/")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tree_path() {
        let base = "https://docs.rust-lang.org/";
        assert_eq!(
            infer_tree_path("https://docs.rust-lang.org/book/ch04.html", base),
            "/book/ch04"
        );
    }

    #[test]
    fn test_docs_rs_version() {
        // Base URL should be the root of the docs site
        let base = "https://docs.rs/tokio/";
        assert_eq!(
            infer_tree_path("https://docs.rs/tokio/rt@1.38.0/task/index.html", base),
            "/rt/task"
        );
    }

    #[test]
    fn test_index_removal() {
        let base = "https://example.com/docs/";
        assert_eq!(
            infer_tree_path("https://example.com/docs/getting-started/index.html", base),
            "/getting-started"
        );
    }

    #[test]
    fn test_root_path() {
        let base = "https://example.com/";
        // Empty path after stripping should return "/"
        let path = "/".to_string();
        assert_eq!(path, "/");
    }

    #[test]
    fn test_breadcrumbs() {
        assert_eq!(infer_breadcrumbs("/rt/task"), vec!["Rt", "Task"]);
        assert_eq!(infer_breadcrumbs("/api/v2/auth"), vec!["API", "V2", "Auth"]);
        assert_eq!(
            infer_breadcrumbs("/getting-started/installation"),
            vec!["Getting Started", "Installation"]
        );
    }

    #[test]
    fn test_acronym_preservation() {
        assert_eq!(title_case("api"), "API");
        assert_eq!(title_case("html"), "HTML");
        assert_eq!(title_case("json"), "JSON");
    }

    #[test]
    fn test_version_removal() {
        assert!(!looks_like_version("task"));
        assert!(looks_like_version("1.38.0"));
        assert!(looks_like_version("v1.0.0"));
        assert!(looks_like_version("2024.01.15"));
    }
}
