//! Full-text search engine using Tantivy
//!
//! Handles Japanese full-text search indexing and querying.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Mutex;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, Searcher, TantivyDocument};
use thiserror::Error;
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Index error: {0}")]
    Index(String),
    #[error("Query parse error: {0}")]
    QueryParse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Create index with schema
fn create_index(index_path: &PathBuf, schema: Schema) -> Result<Index, SearchError> {
    std::fs::create_dir_all(index_path)?;
    Index::create_in_dir(index_path, schema).map_err(|e| SearchError::Index(e.to_string()))
}

/// Build search schema and return all field references
fn build_schema() -> (Schema, Field, Field, Field, Field, Field) {
    let mut schema_builder = Schema::builder();
    let id_field = schema_builder.add_i64_field("id", STORED | INDEXED);
    let url_field = schema_builder.add_text_field("url", STRING | STORED);
    let title_field = schema_builder.add_text_field("title", TEXT | STORED);
    let content_field = schema_builder.add_text_field("content", TEXT | STORED);
    let domain_field = schema_builder.add_text_field("domain", STRING | STORED);
    let schema = schema_builder.build();
    (
        schema,
        id_field,
        url_field,
        title_field,
        content_field,
        domain_field,
    )
}

/// Create index reader with reload policy
fn create_reader(index: &Index) -> Result<IndexReader, SearchError> {
    index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()
        .map_err(|e| SearchError::Index(e.to_string()))
}

/// Create index writer with buffer size
fn create_writer(index: &Index) -> Result<IndexWriter, SearchError> {
    index
        .writer(50_000_000)
        .map_err(|e| SearchError::Index(e.to_string()))
}

/// Search engine configuration
pub struct SearchEngine {
    index: Index,
    reader: IndexReader,
    writer: Mutex<IndexWriter>,
    id_field: Field,
    url_field: Field,
    title_field: Field,
    content_field: Field,
    domain_field: Field,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new(data_dir: &PathBuf) -> Result<Self, SearchError> {
        let index_path = data_dir.join("search_index");
        let (schema, id_field, url_field, title_field, content_field, domain_field) =
            build_schema();

        let index = if index_path.exists() {
            Index::open_in_dir(&index_path).map_err(|e| SearchError::Index(e.to_string()))?
        } else {
            create_index(&index_path, schema)?
        };

        let reader = create_reader(&index)?;
        let writer = create_writer(&index)?;

        info!("Search index initialized at: {}", index_path.display());

        Ok(Self {
            index,
            reader,
            writer: Mutex::new(writer),
            id_field,
            url_field,
            title_field,
            content_field,
            domain_field,
        })
    }

    /// Generate a deterministic ID from URL
    #[allow(dead_code)]
    fn generate_id(&self, url: &str) -> i64 {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        hasher.finish() as i64
    }

    /// Index a document
    /// Returns the generated document ID
    pub fn index_document(
        &self,
        url: &str,
        title: &str,
        content: &str,
        domain: &str,
    ) -> Result<i64, SearchError> {
        let mut writer = self.writer.lock().unwrap();

        // Generate a simple hash ID from URL (deterministic)
        let id = {
            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            hasher.finish() as i64
        };

        // Delete existing document with same URL
        let term = tantivy::Term::from_field_text(self.url_field, url);
        writer.delete_term(term);

        // Add new document
        let doc = doc!(
            self.id_field => id,
            self.url_field => url,
            self.title_field => title,
            self.content_field => content,
            self.domain_field => domain,
        );

        writer
            .add_document(doc)
            .map_err(|e| SearchError::Index(e.to_string()))?;

        writer
            .commit()
            .map_err(|e| SearchError::Index(e.to_string()))?;

        debug!("Indexed document: {} ({})", title, id);

        Ok(id)
    }

    /// Search documents
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let searcher: Searcher = self.reader.searcher();

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.title_field, self.content_field]);

        let query = query_parser
            .parse_query(query)
            .map_err(|e| SearchError::QueryParse(e.to_string()))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| SearchError::Index(e.to_string()))?;

        let mut results = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| SearchError::Index(e.to_string()))?;

            let id = retrieved_doc
                .get_first(self.id_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let url = retrieved_doc
                .get_first(self.url_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let title = retrieved_doc
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let domain = retrieved_doc
                .get_first(self.domain_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(SearchResult {
                id,
                url,
                title,
                domain,
            });
        }

        Ok(results)
    }

    /// Clear the index
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<(), SearchError> {
        let mut writer = self.writer.lock().unwrap();

        writer
            .delete_all_documents()
            .map_err(|e| SearchError::Index(e.to_string()))?;

        writer
            .commit()
            .map_err(|e| SearchError::Index(e.to_string()))?;

        info!("Search index cleared");

        Ok(())
    }

    /// Delete from search index by URL
    pub fn delete_by_url(&self, url: &str) -> Result<(), SearchError> {
        let mut writer = self.writer.lock().unwrap();
        let term = tantivy::Term::from_field_text(self.url_field, url);
        writer.delete_term(term);
        writer
            .commit()
            .map_err(|e| SearchError::Index(e.to_string()))?;
        debug!("Deleted document from search index: {}", url);
        Ok(())
    }

    /// Rebuild the entire search index from database
    #[allow(dead_code)]
    pub fn rebuild_from_db(
        &self,
        articles: &[(String, String, String, String)],
    ) -> Result<(), SearchError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut writer = self.writer.lock().unwrap();

        // Clear all documents
        writer
            .delete_all_documents()
            .map_err(|e| SearchError::Index(e.to_string()))?;
        writer
            .commit()
            .map_err(|e| SearchError::Index(e.to_string()))?;

        // Re-add all articles
        for (url, title, content, domain) in articles {
            let id = {
                let mut hasher = DefaultHasher::new();
                url.hash(&mut hasher);
                hasher.finish() as i64
            };
            let doc = doc!(
                self.id_field => id,
                self.url_field => url.as_str(),
                self.title_field => title.as_str(),
                self.content_field => content.as_str(),
                self.domain_field => domain.as_str(),
            );
            writer
                .add_document(doc)
                .map_err(|e| SearchError::Index(e.to_string()))?;
        }

        writer
            .commit()
            .map_err(|e| SearchError::Index(e.to_string()))?;
        info!("Search index rebuilt with {} documents", articles.len());
        Ok(())
    }

    /// Get document count
    #[allow(dead_code)]
    pub fn doc_count(&self) -> Result<u64, SearchError> {
        let searcher: Searcher = self.reader.searcher();
        Ok(searcher.num_docs())
    }
}

/// Search result from Tantivy index
///
/// Note: The `id` field is a Tantivy internal document ID (derived from URL hash),
/// NOT the SQLite AUTOINCREMENT row ID. Use the `url` field to fetch full article
/// data from SQLite via `Database::get_articles_by_urls()`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Tantivy document ID (URL hash, not SQLite row ID)
    pub id: i64,
    /// Article URL (use this to fetch from SQLite)
    pub url: String,
    /// Article title
    pub title: String,
    /// Article domain
    pub domain: String,
}
