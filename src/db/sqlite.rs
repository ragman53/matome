//! SQLite database operations

use crate::db::DbError;
use crate::db::DbStats;
use crate::pipeline::TranslatedPage;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, info};

/// SQLite database wrapper
pub struct Database {
    conn: Mutex<Connection>,
    #[allow(dead_code)] // Future: debugging, logging
    path: PathBuf,
}

impl Database {
    /// Create a new database connection
    pub fn new(data_dir: &PathBuf) -> Result<Self, DbError> {
        let db_path = data_dir.join("matome.db");

        // Create directory if needed
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir)?;
        }

        let conn = Connection::open(&db_path)?;

        // Initialize schema
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS articles (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                url           TEXT UNIQUE NOT NULL,
                title         TEXT,
                description   TEXT,
                original_md   TEXT NOT NULL,
                translated_md TEXT,
                domain        TEXT NOT NULL,
                crawled_at    TEXT DEFAULT (datetime('now')),
                updated_at    TEXT DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_articles_domain ON articles(domain);
            CREATE INDEX IF NOT EXISTS idx_articles_url ON articles(url);
            CREATE INDEX IF NOT EXISTS idx_articles_crawled ON articles(crawled_at);
            ",
        )?;

        info!("Database initialized at: {}", db_path.display());

        Ok(Self {
            conn: Mutex::new(conn),
            path: db_path,
        })
    }

    /// Save or update an article
    pub fn save_article(&self, article: &TranslatedPage) -> Result<i64, DbError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO articles (url, title, description, original_md, translated_md, domain, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
            ON CONFLICT(url) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                original_md = excluded.original_md,
                translated_md = excluded.translated_md,
                updated_at = datetime('now')
            "#,
            params![
                article.url,
                article.title,
                article.description,
                article.original_md,
                article.translated_md,
                article.domain,
            ],
        )?;

        let id = conn.query_row(
            "SELECT id FROM articles WHERE url = ?1",
            params![article.url],
            |row| row.get(0),
        )?;

        debug!("Saved article: {} (id: {})", article.url, id);

        Ok(id)
    }

    /// Get article by ID
    pub fn get_article(&self, id: i64) -> Result<Option<ArticleRow>, DbError> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT id, url, title, description, original_md, translated_md, domain, crawled_at, updated_at
             FROM articles WHERE id = ?1",
            params![id],
            |row| {
                Ok(ArticleRow {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    original_md: row.get(4)?,
                    translated_md: row.get(5)?,
                    domain: row.get(6)?,
                    crawled_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        );

        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all articles
    pub fn get_all_articles(&self) -> Result<Vec<ArticleRow>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, description, original_md, translated_md, domain, crawled_at, updated_at
             FROM articles ORDER BY crawled_at DESC",
        )?;

        let articles = stmt.query_map([], |row| {
            Ok(ArticleRow {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                original_md: row.get(4)?,
                translated_md: row.get(5)?,
                domain: row.get(6)?,
                crawled_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        articles.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get articles by list of URLs (for search results)
    pub fn get_articles_by_urls(&self, urls: &[String]) -> Result<Vec<ArticleRow>, DbError> {
        if urls.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock().unwrap();
        let placeholders: Vec<String> = urls.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT id, url, title, description, original_md, translated_md, domain, crawled_at, updated_at
             FROM articles WHERE url IN ({})",
            placeholders.join(", ")
        );

        let mut stmt = conn.prepare(&query)?;

        let articles = stmt.query_map(rusqlite::params_from_iter(urls.iter()), |row| {
            Ok(ArticleRow {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                original_md: row.get(4)?,
                translated_md: row.get(5)?,
                domain: row.get(6)?,
                crawled_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        articles.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get articles by domain
    #[allow(dead_code)]
    pub fn get_articles_by_domain(&self, domain: &str) -> Result<Vec<ArticleRow>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, description, original_md, translated_md, domain, crawled_at, updated_at
             FROM articles WHERE domain = ?1 ORDER BY crawled_at DESC",
        )?;

        let articles = stmt.query_map(params![domain], |row| {
            Ok(ArticleRow {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                original_md: row.get(4)?,
                translated_md: row.get(5)?,
                domain: row.get(6)?,
                crawled_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        articles.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get all URLs for a domain
    pub fn get_urls_by_domain(&self, domain: &str) -> Result<HashSet<String>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT url FROM articles WHERE domain = ?1")?;
        let urls = stmt
            .query_map(params![domain], |row| row.get(0))?
            .collect::<Result<HashSet<_>, _>>()?;

        Ok(urls)
    }

    /// Search articles
    pub fn search_articles(&self, query: &str) -> Result<Vec<ArticleRow>, DbError> {
        let conn = self.conn.lock().unwrap();

        let pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, url, title, description, original_md, translated_md, domain, crawled_at, updated_at
             FROM articles
             WHERE title LIKE ?1 OR description LIKE ?1 OR translated_md LIKE ?1
             ORDER BY crawled_at DESC",
        )?;

        let articles = stmt.query_map(params![pattern], |row| {
            Ok(ArticleRow {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                original_md: row.get(4)?,
                translated_md: row.get(5)?,
                domain: row.get(6)?,
                crawled_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        articles.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DbStats, DbError> {
        let conn = self.conn.lock().unwrap();

        let total_articles: usize =
            conn.query_row("SELECT COUNT(*) FROM articles", [], |row| row.get(0))?;

        let domains: usize =
            conn.query_row("SELECT COUNT(DISTINCT domain) FROM articles", [], |row| {
                row.get(0)
            })?;

        let (original_md_size, translated_md_size): (i64, i64) = conn.query_row(
            "SELECT COALESCE(SUM(LENGTH(original_md)), 0), COALESCE(SUM(LENGTH(translated_md)), 0)
             FROM articles",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        Ok(DbStats {
            total_articles,
            indexed_articles: total_articles,
            domains,
            original_md_size: original_md_size as usize,
            translated_md_size: translated_md_size as usize,
        })
    }

    /// Delete article by ID
    #[allow(dead_code)]
    pub fn delete_article(&self, id: i64) -> Result<bool, DbError> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute("DELETE FROM articles WHERE id = ?1", params![id])?;

        Ok(deleted > 0)
    }

    /// Delete articles by domain
    pub fn delete_by_domain(&self, domain: &str) -> Result<usize, DbError> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute("DELETE FROM articles WHERE domain = ?1", params![domain])?;

        info!("Deleted {} articles from domain '{}'", deleted, domain);

        Ok(deleted)
    }

    /// Delete orphaned articles (missing title, description, or translation)
    pub fn delete_orphaned(&self) -> Result<usize, DbError> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute(
            r#"DELETE FROM articles WHERE 
                title IS NULL OR title = '' OR 
                description IS NULL OR description = '' OR 
                translated_md IS NULL OR translated_md = '' OR
                LENGTH(original_md) < 50"#,
            [],
        )?;

        info!("Deleted {} orphaned articles", deleted);

        Ok(deleted)
    }

    /// Get orphaned articles (for display before deletion)
    pub fn get_orphaned_articles(&self) -> Result<Vec<ArticleRow>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"SELECT id, url, title, description, original_md, translated_md, domain, crawled_at, updated_at
             FROM articles 
             WHERE title IS NULL OR title = '' OR 
                   description IS NULL OR description = '' OR 
                   translated_md IS NULL OR translated_md = '' OR
                   LENGTH(original_md) < 50
             ORDER BY domain, id"#,
        )?;

        let articles = stmt.query_map([], |row| {
            Ok(ArticleRow {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                original_md: row.get(4)?,
                translated_md: row.get(5)?,
                domain: row.get(6)?,
                crawled_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        articles.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Clear all articles
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<usize, DbError> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute("DELETE FROM articles", [])?;

        info!("Cleared {} articles", deleted);

        Ok(deleted)
    }
}

/// Article row from database
#[derive(Debug, Clone)]
pub struct ArticleRow {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub original_md: String,
    pub translated_md: Option<String>,
    pub domain: String,
    pub crawled_at: String,
    #[allow(dead_code)] // Future: display/update tracking
    pub updated_at: String,
}
