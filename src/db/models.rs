//! Data models for v0.2.0 hierarchical structure
//!
//! Documents -> Sections -> Pages hierarchy

use serde::{Deserialize, Serialize};

/// Document: represents a crawled website/repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Document {
    pub id: String,                  // UUID v7
    pub base_url: String,            // e.g., "https://docs.rust-lang.org/"
    pub name: String,                // User-defined alias, e.g., "rust-book"
    pub config_json: Option<String>, // Serialized domain config
    pub created_at: String,          // ISO 8601 timestamp
}

/// Section: logical grouping within a document (e.g., "Getting Started", "API Reference")
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Section {
    pub id: String,          // UUID
    pub document_id: String, // FK to documents
    pub title: String,       // Display name, e.g., "Getting Started"
    pub path_prefix: String, // URL prefix, e.g., "/getting-started"
    pub sort_order: i32,     // Display order
}

/// Page: actual content with hierarchical information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,                  // UUID
    pub section_id: String,          // FK to sections
    pub url: String,                 // Original URL
    pub title: String,               // Page title
    pub tree_path: String,           // Hierarchical path, e.g., "/getting-started/installation"
    pub breadcrumbs: String,         // JSON array, e.g., "[\"Getting Started\", \"Installation\"]"
    pub content_hash: String,        // SHA-256 of normalized content
    pub doc_version: Option<String>, // Auto-detected or manual tag
    pub crawled_at: String,          // ISO 8601 timestamp
    pub raw_html: Option<String>,    // Optional: stored HTML (compressed)
    pub clean_markdown: String,      // Cleaned markdown content
    pub original_markdown: String,   // Before translation
    pub translated_markdown: String, // After translation
    pub meta_json: Option<String>,   // Code block count, token estimate, etc.
}

/// Page version: historical record for change tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PageVersion {
    pub id: i64,                      // Auto-increment
    pub page_id: String,              // FK to pages
    pub hash: String,                 // Content hash at this version
    pub diff_snippet: Option<String>, // Summary of major changes
    pub created_at: String,           // ISO 8601 timestamp
}

// === New data structures for pipeline ===

/// Page data for the new hierarchical model
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HierarchicalPage {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub tree_path: String,
    pub breadcrumbs: Vec<String>,
    pub clean_markdown: String,
    pub original_markdown: String,
    pub translated_markdown: String,
    pub domain: String,
    pub content_hash: String,
}

/// Tree node for UI rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub title: String,
    pub path: String,
    pub page_id: Option<String>,
    pub children: Vec<TreeNode>,
}

/// Change detection result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    None,     // Hash unchanged
    Minor,    // typo/formatting changes
    Major,    // section rewrite
    Breaking, // glossary priority terms changed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ChangeResult {
    pub change_type: ChangeType,
    pub old_hash: String,
    pub new_hash: String,
    pub glossary_alerts: Vec<String>,
    pub diff_snippet: Option<String>,
}

/// Change summary for diff mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ChangeSummary {
    pub page_id: String,
    pub page_title: String,
    pub tree_path: String,
    pub change_type: ChangeType,
    pub glossary_alerts: Vec<String>,
    pub changed_at: String,
}

/// Agent workspace manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub workspace: String,
    pub source_url: String,
    pub doc_version: Option<String>,
    pub crawled_at: String,
    pub total_files: usize,
    pub total_tokens_estimate: usize,
    pub structure_type: String,
    pub agent_contract: Vec<String>,
    pub sections: Vec<SectionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSummary {
    pub name: String,
    pub files: usize,
    pub tokens_estimate: usize,
}

/// Token budget for AI context windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub context_limit: usize,
    pub total_tokens: usize,
    pub recommended_reading_order: Vec<ReadingItem>,
    pub priority_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingItem {
    pub section: String,
    pub files: Vec<String>,
    pub tokens: usize,
}
