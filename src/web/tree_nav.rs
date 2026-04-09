//! v0.2.0 Tree navigation helpers
//!
//! Generates hierarchical document tree for sidebar navigation.

use crate::db::models::TreeNode;
use std::collections::HashMap;

/// Build tree structure from flat list of tree_path entries
///
/// Input: [("/getting-started", "Getting Started"), ("/getting-started/installation", "Installation")]
/// Output: Tree structure with nested children
pub fn build_tree_from_paths(paths: &[(String, String)]) -> Vec<TreeNode> {
    let mut root: Vec<TreeNode> = Vec::new();
    let mut node_map: HashMap<String, usize> = HashMap::new();

    for (tree_path, title) in paths {
        let segments: Vec<&str> = tree_path.split('/').filter(|s| !s.is_empty()).collect();

        if segments.is_empty() {
            continue;
        }

        let mut current_level = &mut root;
        let mut current_path = String::new();

        for (i, segment) in segments.iter().enumerate() {
            if !current_path.is_empty() {
                current_path.push('/');
            }
            current_path.push_str(segment);

            let is_leaf = i == segments.len() - 1;
            let node_title = if is_leaf {
                title.clone()
            } else {
                to_title_case(segment)
            };

            // Check if this node already exists
            if let Some(&idx) = node_map.get(&current_path) {
                current_level = &mut current_level[idx].children;
            } else {
                let new_idx = current_level.len();
                node_map.insert(current_path.clone(), new_idx);

                current_level.push(TreeNode {
                    title: node_title,
                    path: current_path.clone(),
                    page_id: if is_leaf {
                        Some(format!("page-{}", new_idx))
                    } else {
                        None
                    },
                    children: Vec::new(),
                });

                current_level = &mut current_level[new_idx].children;
            }
        }
    }

    // Sort children by path
    sort_tree_recursive(&mut root);
    root
}

/// Sort tree nodes recursively by path
fn sort_tree_recursive(nodes: &mut [TreeNode]) {
    nodes.sort_by(|a, b| a.path.cmp(&b.path));
    for node in nodes.iter_mut() {
        sort_tree_recursive(&mut node.children);
    }
}

/// Convert path segment to Title Case
fn to_title_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let acronyms = [
        "api", "html", "css", "xml", "json", "sql", "http", "https", "url", "uri", "gui", "cli",
    ];
    if acronyms.contains(&lower.as_str()) {
        return s.to_uppercase();
    }
    s.split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generate HTML for tree navigation sidebar
pub fn render_tree_nav(nodes: &[TreeNode], depth: usize) -> String {
    let mut html = String::new();

    for node in nodes {
        let indent = "  ".repeat(depth);
        let _path_id = format!("tree-{}", node.path.replace('/', "-"));
        let has_children = !node.children.is_empty();

        if has_children {
            // Section header (not clickable)
            html.push_str(&format!(
                "{}<div class=\"nav-section\">\n{}  <div class=\"nav-section-title\">{}</div>\n",
                indent, indent, node.title
            ));
            html.push_str(&render_tree_nav(&node.children, depth + 2));
            html.push_str(&format!("{}</div>\n", indent));
        } else {
            // Leaf node (clickable page link)
            html.push_str(&format!(
                r#"<a href="/tree{}" class="nav-item tree-leaf" data-path="{}">
  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>
  {}
</a>\n"#,
                node.path, node.path, node.title
            ));
        }
    }

    html
}

/// Generate simple domain-based navigation (fallback for flat data)
pub fn render_domain_nav(domains: &[(String, usize)]) -> String {
    let mut html = String::new();
    html.push_str(
        r#"<div class="nav-section">
<div class="nav-section-title">ドメイン別</div>
"#,
    );

    for (domain, count) in domains {
        html.push_str(&format!(
            r#"<a href="/domain/{}" class="nav-item">
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
{}
<span class="count">{}</span>
</a>
"#,
            domain, domain, count
        ));
    }

    html.push_str("</div>\n");
    html
}

/// Generate breadcrumb HTML from path segments
pub fn render_breadcrumbs(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return String::new();
    }

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut html = String::new();
    html.push_str(r#"<div class="breadcrumb">"#);
    html.push_str(r#"<a href="/">ホーム</a>"#);

    let mut cum_path = String::new();
    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            html.push_str(r#"<span>/</span>"#);
        }
        if !cum_path.is_empty() {
            cum_path.push('/');
        }
        cum_path.push_str(segment);

        if i == segments.len() - 1 {
            // Current page (not clickable)
            html.push_str(&format!(r#"<span>{}</span>"#, to_title_case(segment)));
        } else {
            // Ancestor (clickable)
            html.push_str(&format!(
                r#"<a href="/tree/{}">{}</a>"#,
                cum_path,
                to_title_case(segment)
            ));
        }
    }

    html.push_str("</div>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("getting-started"), "Getting Started");
        assert_eq!(to_title_case("API"), "API");
        assert_eq!(to_title_case("json"), "JSON");
    }

    #[test]
    fn test_build_tree() {
        let paths = vec![
            (
                "/getting-started".to_string(),
                "Getting Started".to_string(),
            ),
            (
                "/getting-started/installation".to_string(),
                "Installation".to_string(),
            ),
            ("/api".to_string(), "API Reference".to_string()),
        ];

        let tree = build_tree_from_paths(&paths);
        // Tree is sorted alphabetically
        assert_eq!(tree.len(), 2); // api, getting-started

        // Check alphabetical order: api < getting-started
        assert_eq!(tree[0].title, "API Reference"); // api comes first
        assert_eq!(tree[1].title, "Getting Started");

        // Check nested children under getting-started
        assert_eq!(tree[1].children.len(), 1);
        assert_eq!(tree[1].children[0].title, "Installation");
    }

    #[test]
    fn test_breadcrumbs() {
        let html = render_breadcrumbs("/api/v2/auth");
        assert!(html.contains("API"));
        assert!(html.contains("V2"));
        assert!(html.contains("Auth"));
        assert!(html.contains("/tree/api"));
        assert!(html.contains("/tree/api/v2"));
    }
}
