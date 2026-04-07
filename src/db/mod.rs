//! Database module
//!
//! Handles SQLite storage and Tantivy full-text search.

mod error;
mod search;
mod sqlite;

pub use error::DbError;
pub use sqlite::{ArticleRow, Database};

use crate::config::Config;
use std::path::PathBuf;

/// Database statistics
#[derive(Debug, Clone, Default)]
pub struct DbStats {
    pub total_articles: usize,
    pub indexed_articles: usize,
    pub domains: usize,
    pub original_md_size: usize,
    pub translated_md_size: usize,
}

impl Config {
    /// Initialize database directory
    pub fn init_data_dir(&self) -> Result<PathBuf, DbError> {
        let data_dir = PathBuf::from(&self.core.data_dir);

        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        Ok(data_dir)
    }
}
