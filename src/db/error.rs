//! Database error types

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Search index error: {0}")]
    Search(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
