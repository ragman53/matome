//! Database migration for v0.2.0
//!
//! Adds hierarchical document structure tables:
//! - documents: sites/repositories
//! - sections: logical groupings
//! - pages: actual content with tree_path, content_hash, breadcrumbs
//! - page_versions: change history

use rusqlite::{params, Connection, Result};
use tracing::{info, warn};

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

    // Migrate data from articles to new tables
    migrate_articles_data(conn)?;

    info!("Migration to v0.2.0 completed successfully");
    Ok(())
}

/// Migrate data from articles table to new hierarchical structure
#[allow(clippy::type_complexity)]
fn migrate_articles_data(conn: &Connection) -> Result<(), DbError> {
    // Check if articles table exists
    let has_articles_table: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='articles'",
        [],
        |row| row.get(0),
    )?;
    let has_articles_table = has_articles_table > 0;

    if !has_articles_table {
        info!("No articles table - fresh install");
        return Ok(());
    }

    // Check if articles table has data
    let article_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM articles", [], |row| row.get(0))?;

    if article_count == 0 {
        info!("No articles to migrate");
        return Ok(());
    }

    info!("Migrating {} articles to new data model...", article_count);

    // Get unique domains (these become documents)
    let mut domains: Vec<String> = Vec::new();
    let mut stmt = conn.prepare("SELECT DISTINCT domain FROM articles")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    for row in rows {
        domains.push(row?);
    }

    info!("Found {} unique domains", domains.len());

    // Create document for each domain and migrate articles
    for domain in &domains {
        // Generate document ID from domain name
        let doc_id = generate_uuid_from_string(domain);
        let doc_name = domain.split('.').next().unwrap_or(domain).to_string();

        // Insert document
        conn.execute(
            "INSERT OR IGNORE INTO documents (id, base_url, name) VALUES (?1, ?2, ?3)",
            params![doc_id, format!("https://{}", domain), doc_name],
        )?;

        // Create default section for the document
        let section_id = format!("{}-root", doc_id);
        conn.execute(
            "INSERT OR IGNORE INTO sections (id, document_id, title, path_prefix) VALUES (?1, ?2, ?3, ?4)",
            params![section_id, doc_id, domain, ""],
        )?;

        // Get all articles for this domain
        let mut article_stmt = conn.prepare(
            "SELECT id, url, title, description, original_md, translated_md, crawled_at FROM articles WHERE domain = ?1"
        )?;

        let articles: Vec<(
            i64,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
            String,
        )> = article_stmt
            .query_map(params![domain], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Migrate each article to pages table
        for (old_id, url, title, description, original_md, translated_md, crawled_at) in articles {
            let page_id = generate_uuid_from_string(&url);

            // Compute content hash
            let content_hash = compute_sha256_hash(&original_md);

            // Generate tree_path from URL
            let tree_path = generate_tree_path_from_url(&url, domain);

            // Generate breadcrumbs JSON
            let breadcrumbs = generate_breadcrumbs(&tree_path);

            conn.execute(
                r#"
                INSERT OR REPLACE INTO pages (
                    id, section_id, url, title, tree_path, breadcrumbs,
                    content_hash, crawled_at, clean_markdown, original_markdown, translated_markdown,
                    meta_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
                params![
                    page_id,
                    section_id,
                    url,
                    title,
                    tree_path,
                    breadcrumbs,
                    content_hash,
                    crawled_at,
                    original_md,
                    original_md,
                    translated_md,
                    serde_json::json!({
                        "description": description,
                        "original_article_id": old_id,
                        "migrated": true
                    }).to_string(),
                ],
            )?;
        }
    }

    // Verify migration
    let page_count: i64 = conn.query_row("SELECT COUNT(*) FROM pages", [], |row| row.get(0))?;
    let doc_count: i64 = conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
    let section_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM sections", [], |row| row.get(0))?;

    info!(
        "Migration complete: {} documents, {} sections, {} pages created",
        doc_count, section_count, page_count
    );

    if page_count != article_count {
        warn!(
            "Article count mismatch: {} articles vs {} pages",
            article_count, page_count
        );
    }

    Ok(())
}

/// Generate a UUID-like string from a string for consistent IDs
pub fn generate_uuid_from_string(s: &str) -> String {
    use sha2::Digest;
    use sha2::Sha256 as HashTrait;

    let mut hasher = HashTrait::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();

    // Format as UUID v5-like (using SHA256)
    let hash_hex = format!("{:x}", result);
    format!(
        "{:8}-{:4}-{}-{}-{}",
        &hash_hex[0..8],
        &hash_hex[8..12],
        &hash_hex[12..16],
        &hash_hex[16..20],
        &hash_hex[20..32]
    )
}

/// Compute SHA-256 hash of content
fn compute_sha256_hash(content: &str) -> String {
    use sha2::Digest;
    use sha2::Sha256 as HashTrait;

    let mut hasher = HashTrait::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Generate tree_path from URL and domain
fn generate_tree_path_from_url(url: &str, domain: &str) -> String {
    // Remove domain from URL and create path
    let path = url.replace(&format!("https://{}", domain), "");
    let path = path.replace(&format!("http://{}", domain), "");

    // If empty, use root
    if path.is_empty() || path == "/" {
        return format!("/{}/_root", domain.split('.').next().unwrap_or(domain));
    }

    // Clean up the path
    let clean_path = path.trim_end_matches('/');
    format!(
        "/{}{}",
        domain.split('.').next().unwrap_or(domain),
        clean_path
    )
}

/// Generate breadcrumbs JSON from tree_path
fn generate_breadcrumbs(tree_path: &str) -> String {
    let segments: Vec<&str> = tree_path.split('/').filter(|s| !s.is_empty()).collect();

    let crumbs: Vec<serde_json::Value> = segments
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let path_parts: Vec<&str> = segments[..=idx].to_vec();
            let path = format!("/tree/{}", path_parts.join("/"));
            serde_json::json!({
                "title": to_title_case(s),
                "path": path
            })
        })
        .collect();

    serde_json::to_string(&crumbs).unwrap_or_else(|_| "[]".to_string())
}

/// Convert path segment to Title Case
fn to_title_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let acronyms = [
        "api", "html", "css", "xml", "json", "sql", "http", "https", "url", "uri",
    ];
    if acronyms.contains(&lower.as_str()) {
        return s.to_uppercase();
    }
    s.split(['-', '_'])
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
    #[allow(dead_code)]
    pub has_articles: bool,
    #[allow(dead_code)]
    pub has_pages: bool,
    #[allow(dead_code)]
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
