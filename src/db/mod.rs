//! Database module
//!
//! Handles SQLite storage and Tantivy full-text search.

mod error;
pub mod migration;
pub mod models;
pub mod search;
mod sqlite;

pub use error::DbError;
pub use migration::generate_uuid_from_string;
pub use search::SearchEngine;
pub use sqlite::{ArticleRow, Database};

/// Database statistics
#[derive(Debug, Clone, Default)]
pub struct DbStats {
    pub total_articles: usize,
    pub indexed_articles: usize,
    pub domains: usize,
    pub original_md_size: usize,
    pub translated_md_size: usize,
}
