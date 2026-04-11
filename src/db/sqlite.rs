//! SQLite database operations

use crate::db::migration::{check_and_migrate, get_migration_status};
use crate::db::DbError;
use crate::db::DbStats;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// SQLite database wrapper
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    #[allow(dead_code)] // Future: debugging, logging
    path: PathBuf,
}

impl Database {
    /// Create a new database connection
    pub fn new(data_dir: &Path) -> Result<Self, DbError> {
        let db_path = data_dir.join("matome.db");

        // Create directory if needed
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir)?;
        }

        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrency
        // WAL mode allows concurrent reads while writing, improving performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Initialize legacy schema (articles table for backwards compatibility)
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

        // Check migration status and run if needed
        match get_migration_status(&conn) {
            Ok(status) => {
                if status.status.contains("v0.1.0") {
                    warn!("Database is at v0.1.0 schema. Consider running: matome migrate");
                } else if status.status.contains("v0.2.0") {
                    info!("Database schema: {}", status.status);
                }
                // Try to run migration
                if let Err(e) = check_and_migrate(&conn) {
                    warn!("Migration check failed (non-critical): {}", e);
                }
            }
            Err(e) => {
                warn!("Could not check migration status: {}", e);
            }
        }

        info!("Database initialized at: {}", db_path.display());

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            path: db_path,
        })
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

// ============== v0.2.0: New data model methods ==============

impl Database {
    /// Save or update a page (new v0.2.0 data model)
    #[allow(dead_code)]
    pub fn save_page(&self, page: &crate::db::models::Page) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO pages (
                id, section_id, url, title, tree_path, breadcrumbs,
                content_hash, doc_version, crawled_at, raw_html,
                clean_markdown, original_markdown, translated_markdown, meta_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                page.id,
                page.section_id,
                page.url,
                page.title,
                page.tree_path,
                page.breadcrumbs,
                page.content_hash,
                page.doc_version,
                page.crawled_at,
                page.raw_html,
                page.clean_markdown,
                page.original_markdown,
                page.translated_markdown,
                page.meta_json,
            ],
        )?;

        debug!("Saved page: {} ({})", page.title, page.url);

        Ok(())
    }

    /// Get all pages (v0.2.0)
    pub fn get_all_pages(&self) -> Result<Vec<crate::db::models::Page>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, section_id, url, title, tree_path, breadcrumbs,
                   content_hash, doc_version, crawled_at, raw_html,
                   clean_markdown, original_markdown, translated_markdown, meta_json
            FROM pages ORDER BY tree_path
            "#,
        )?;

        let pages = stmt.query_map([], |row| {
            Ok(crate::db::models::Page {
                id: row.get(0)?,
                section_id: row.get(1)?,
                url: row.get(2)?,
                title: row.get(3)?,
                tree_path: row.get(4)?,
                breadcrumbs: row.get(5)?,
                content_hash: row.get(6)?,
                doc_version: row.get(7)?,
                crawled_at: row.get(8)?,
                raw_html: row.get(9)?,
                clean_markdown: row.get(10)?,
                original_markdown: row.get(11)?,
                translated_markdown: row.get(12)?,
                meta_json: row.get(13)?,
            })
        })?;

        pages.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get all documents (v0.2.0)
    #[allow(dead_code)]
    pub fn get_all_documents(&self) -> Result<Vec<crate::db::models::Document>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, base_url, name, config_json, created_at FROM documents ORDER BY name",
        )?;

        let docs = stmt.query_map([], |row| {
            Ok(crate::db::models::Document {
                id: row.get(0)?,
                base_url: row.get(1)?,
                name: row.get(2)?,
                config_json: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        docs.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get pages with tree paths for UI (v0.2.0)
    pub fn get_pages_with_tree(&self) -> Result<Vec<(String, String)>, DbError> {
        let pages = self.get_all_pages()?;
        Ok(pages.into_iter().map(|p| (p.tree_path, p.title)).collect())
    }

    /// Get domain counts from pages table (v0.2.0)
    pub fn get_domain_counts(&self) -> Result<Vec<(String, usize)>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT d.name as domain, COUNT(p.id) as count
            FROM documents d
            LEFT JOIN sections s ON s.document_id = d.id
            LEFT JOIN pages p ON p.section_id = s.id
            GROUP BY d.id, d.name
            ORDER BY d.name
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get pages by domain (for v0.2.0 migration)
    #[allow(dead_code)] // Future: domain-specific page queries
    pub fn get_pages_by_domain(&self, domain: &str) -> Result<Vec<crate::db::models::Page>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT p.id, p.section_id, p.url, p.title, p.tree_path, p.breadcrumbs,
                   p.content_hash, p.doc_version, p.crawled_at, p.raw_html,
                   p.clean_markdown, p.original_markdown, p.translated_markdown, p.meta_json
            FROM pages p
            JOIN sections s ON s.id = p.section_id
            JOIN documents d ON d.id = s.document_id
            WHERE d.name = ?1 OR d.base_url LIKE ?2
            ORDER BY p.crawled_at DESC
            "#,
        )?;

        let pages = stmt.query_map(params![domain, format!("%{}", domain)], |row| {
            Ok(crate::db::models::Page {
                id: row.get(0)?,
                section_id: row.get(1)?,
                url: row.get(2)?,
                title: row.get(3)?,
                tree_path: row.get(4)?,
                breadcrumbs: row.get(5)?,
                content_hash: row.get(6)?,
                doc_version: row.get(7)?,
                crawled_at: row.get(8)?,
                raw_html: row.get(9)?,
                clean_markdown: row.get(10)?,
                original_markdown: row.get(11)?,
                translated_markdown: row.get(12)?,
                meta_json: row.get(13)?,
            })
        })?;

        pages.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get page count from pages table
    pub fn get_page_count(&self) -> Result<usize, DbError> {
        let conn = self.conn.lock().unwrap();

        let count: usize = conn.query_row("SELECT COUNT(*) FROM pages", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get all pages with domain info for status/command use
    pub fn get_all_pages_with_domain(&self) -> Result<Vec<PageWithDomain>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT p.id, p.url, p.title, p.tree_path, p.content_hash, p.crawled_at,
                   p.clean_markdown, p.original_markdown, p.translated_markdown, p.meta_json,
                   d.name as domain
            FROM pages p
            JOIN sections s ON s.id = p.section_id
            JOIN documents d ON d.id = s.document_id
            ORDER BY p.crawled_at DESC
            "#,
        )?;

        let pages = stmt.query_map([], |row| {
            Ok(PageWithDomain {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                tree_path: row.get(3)?,
                content_hash: row.get(4)?,
                crawled_at: row.get(5)?,
                clean_markdown: row.get(6)?,
                original_markdown: row.get(7)?,
                translated_markdown: row.get(8)?,
                meta_json: row.get(9)?,
                domain: row.get(10)?,
            })
        })?;

        pages.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Delete page by ID
    #[allow(dead_code)] // Future: page-specific deletion
    pub fn delete_page(&self, id: &str) -> Result<bool, DbError> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute("DELETE FROM pages WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }

    /// Delete pages by domain
    pub fn delete_pages_by_domain(&self, domain: &str) -> Result<usize, DbError> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute(
            r#"DELETE FROM pages WHERE section_id IN (
                SELECT s.id FROM sections s
                JOIN documents d ON d.id = s.document_id
                WHERE d.name = ?1 OR d.base_url LIKE ?2
            )"#,
            params![domain, format!("%{}", domain)],
        )?;

        info!("Deleted {} pages from domain '{}'", deleted, domain);
        Ok(deleted)
    }

    /// Delete orphaned pages
    pub fn delete_orphaned_pages(&self) -> Result<usize, DbError> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute(
            r#"DELETE FROM pages WHERE
                title IS NULL OR title = '' OR
                translated_markdown IS NULL OR translated_markdown = '' OR
                LENGTH(clean_markdown) < 50"#,
            [],
        )?;

        info!("Deleted {} orphaned pages", deleted);
        Ok(deleted)
    }

    /// Get orphaned pages
    pub fn get_orphaned_pages(&self) -> Result<Vec<PageWithDomain>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"SELECT p.id, p.url, p.title, p.tree_path, p.content_hash, p.crawled_at,
                      p.clean_markdown, p.original_markdown, p.translated_markdown, p.meta_json,
                      d.name as domain
               FROM pages p
               JOIN sections s ON s.id = p.section_id
               JOIN documents d ON d.id = s.document_id
               WHERE p.title IS NULL OR p.title = '' OR
                     p.translated_markdown IS NULL OR p.translated_markdown = '' OR
                     LENGTH(p.clean_markdown) < 50
               ORDER BY d.name, p.id"#,
        )?;

        let pages = stmt.query_map([], |row| {
            Ok(PageWithDomain {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                tree_path: row.get(3)?,
                content_hash: row.get(4)?,
                crawled_at: row.get(5)?,
                clean_markdown: row.get(6)?,
                original_markdown: row.get(7)?,
                translated_markdown: row.get(8)?,
                meta_json: row.get(9)?,
                domain: row.get(10)?,
            })
        })?;

        pages.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Clear all pages
    pub fn clear_pages(&self) -> Result<usize, DbError> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute("DELETE FROM pages", [])?;
        info!("Cleared {} pages", deleted);
        Ok(deleted)
    }

    /// Get page by URL
    #[allow(dead_code)] // Future: URL-based page lookup
    pub fn get_page_by_url(&self, url: &str) -> Result<Option<crate::db::models::Page>, DbError> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            r#"SELECT p.id, p.section_id, p.url, p.title, p.tree_path, p.breadcrumbs,
                      p.content_hash, p.doc_version, p.crawled_at, p.raw_html,
                      p.clean_markdown, p.original_markdown, p.translated_markdown, p.meta_json
               FROM pages p WHERE p.url = ?1"#,
            params![url],
            |row| {
                Ok(crate::db::models::Page {
                    id: row.get(0)?,
                    section_id: row.get(1)?,
                    url: row.get(2)?,
                    title: row.get(3)?,
                    tree_path: row.get(4)?,
                    breadcrumbs: row.get(5)?,
                    content_hash: row.get(6)?,
                    doc_version: row.get(7)?,
                    crawled_at: row.get(8)?,
                    raw_html: row.get(9)?,
                    clean_markdown: row.get(10)?,
                    original_markdown: row.get(11)?,
                    translated_markdown: row.get(12)?,
                    meta_json: row.get(13)?,
                })
            },
        );

        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get page URLs by domain for search index cleanup
    pub fn get_page_urls_by_domain(&self, domain: &str) -> Result<HashSet<String>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"SELECT p.url FROM pages p
               JOIN sections s ON s.id = p.section_id
               JOIN documents d ON d.id = s.document_id
               WHERE d.name = ?1 OR d.base_url LIKE ?2"#,
        )?;

        let urls = stmt
            .query_map(params![domain, format!("%{}", domain)], |row| row.get(0))?
            .collect::<Result<HashSet<_>, _>>()?;

        Ok(urls)
    }
}

/// Page row from database with domain info
#[derive(Debug, Clone)]
#[allow(dead_code)] // Future: full v0.2.0 feature integration
pub struct PageWithDomain {
    pub id: String,
    pub url: String,
    pub title: String,
    pub tree_path: String,
    pub content_hash: String,
    pub crawled_at: String,
    pub clean_markdown: String,
    pub original_markdown: String,
    pub translated_markdown: String,
    pub meta_json: Option<String>,
    pub domain: String,
}
