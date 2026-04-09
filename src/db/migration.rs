//! Database migration for v0.2.0
//!
//! Adds hierarchical document structure tables:
//! - documents: sites/repositories
//! - sections: logical groupings
//! - pages: actual content with tree_path, content_hash, breadcrumbs
//! - page_versions: change history

use rusqlite::{Connection, Result};
use tracing::info;

use crate::db::DbError;

/// Run migration from v0.1.0 (flat articles) to v0.2.0 (hierarchical)
pub fn migrate_to_v0_2_0(conn: &Connection) -> Result<(), DbError> {
    info!("Running migration to v0.2.0...");

    // Create new tables
    conn.execute_batch(
        r#"
        -- Documents table: represents a crawled website/repository
        CREATE TABLE IF NOT EXISTS documents (
            id          TEXT PRIMARY KEY,
            base_url    TEXT UNIQUE NOT NULL,
            name        TEXT NOT NULL,
            config_json TEXT,
            created_at  TEXT DEFAULT (datetime('now'))
        );

        -- Sections table: logical groupings within a document
        CREATE TABLE IF NOT EXISTS sections (
            id          TEXT PRIMARY KEY,
            document_id TEXT REFERENCES documents(id) ON DELETE CASCADE,
            title       TEXT NOT NULL,
            path_prefix TEXT,
            sort_order  INTEGER DEFAULT 0,
            UNIQUE(document_id, path_prefix)
        );

        -- Pages table: actual content with hierarchical info
        -- This replaces the old articles table
        CREATE TABLE IF NOT EXISTS pages (
            id                  TEXT PRIMARY KEY,
            section_id          TEXT REFERENCES sections(id) ON DELETE CASCADE,
            url                 TEXT UNIQUE NOT NULL,
            title               TEXT,
            tree_path           TEXT NOT NULL,
            breadcrumbs         TEXT,
            content_hash        TEXT NOT NULL,
            doc_version         TEXT,
            crawled_at          TEXT DEFAULT (datetime('now')),
            raw_html            TEXT,
            clean_markdown      TEXT NOT NULL,
            original_markdown   TEXT,
            translated_markdown TEXT,
            meta_json           TEXT
        );

        -- Page versions table: change history
        CREATE TABLE IF NOT EXISTS page_versions (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            page_id       TEXT REFERENCES pages(id) ON DELETE CASCADE,
            hash          TEXT NOT NULL,
            diff_snippet  TEXT,
            created_at    TEXT DEFAULT (datetime('now'))
        );

        -- Indexes for new tables
        CREATE INDEX IF NOT EXISTS idx_pages_section ON pages(section_id);
        CREATE INDEX IF NOT EXISTS idx_pages_tree_path ON pages(tree_path);
        CREATE INDEX IF NOT EXISTS idx_pages_doc_version ON pages(doc_version);
        CREATE INDEX IF NOT EXISTS idx_pages_content_hash ON pages(content_hash);
        CREATE INDEX IF NOT EXISTS idx_sections_document ON sections(document_id);
        CREATE INDEX IF NOT EXISTS idx_page_versions_page ON page_versions(page_id);
        "#,
    )?;

    info!("Migration to v0.2.0 completed successfully");
    Ok(())
}

/// Check if migration is needed and run if necessary
pub fn check_and_migrate(conn: &Connection) -> Result<bool, DbError> {
    // Check if pages table exists (v0.2.0 indicator)
    let has_pages_table: bool = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='pages'",
        [],
        |row| row.get(0),
    )?;

    if has_pages_table {
        info!("Database already at v0.2.0 schema");
        return Ok(false);
    }

    // Need to migrate
    migrate_to_v0_2_0(conn)?;
    Ok(true)
}

/// Get migration status
pub fn get_migration_status(conn: &Connection) -> Result<MigrationStatus, DbError> {
    let has_articles = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='articles'",
        [],
        |row| row.get::<_, i64>(0),
    )? > 0;

    let has_pages = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='pages'",
        [],
        |row| row.get::<_, i64>(0),
    )? > 0;

    let has_documents = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='documents'",
        [],
        |row| row.get::<_, i64>(0),
    )? > 0;

    let status = match (has_articles, has_pages, has_documents) {
        (true, false, false) => "v0.1.0 - needs migration",
        (true, true, true) => "v0.2.0 - migrated",
        (false, true, true) => "v0.2.0 - fresh install",
        _ => "unknown",
    };

    Ok(MigrationStatus {
        has_articles,
        has_pages,
        has_documents,
        status: status.to_string(),
    })
}

#[derive(Debug)]
pub struct MigrationStatus {
    pub has_articles: bool,
    pub has_pages: bool,
    pub has_documents: bool,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_migration_status_v0_1() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).expect("failed to open db");

        // Create old schema
        conn.execute_batch(
            "CREATE TABLE articles (
                id INTEGER PRIMARY KEY,
                url TEXT UNIQUE NOT NULL,
                title TEXT,
                original_md TEXT NOT NULL,
                translated_md TEXT,
                domain TEXT NOT NULL
            );",
        )
        .expect("failed to create articles table");

        let status = get_migration_status(&conn).expect("failed to get status");
        assert!(status.has_articles);
        assert!(!status.has_pages);
        assert!(status.status.contains("v0.1.0"));
    }

    #[test]
    fn test_migration_status_v0_2() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).expect("failed to open db");

        // Create new schema directly
        migrate_to_v0_2_0(&conn).expect("failed to migrate");

        let status = get_migration_status(&conn).expect("failed to get status");
        assert!(!status.has_articles);
        assert!(status.has_pages);
        assert!(status.has_documents);
        assert!(status.status.contains("v0.2.0"));
    }
}
