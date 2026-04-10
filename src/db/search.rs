//! Full-text search engine using Tantivy
//!
//! Handles Japanese full-text search indexing and querying.
//! v0.2.0: Added tree_path and doc_version fields for hierarchical search.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
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
fn create_index(index_path: &Path, schema: Schema) -> Result<Index, SearchError> {
    std::fs::create_dir_all(index_path)?;
    Index::create_in_dir(index_path, schema).map_err(|e| SearchError::Index(e.to_string()))
}

/// Build search schema and return all field references
/// v0.2.0: Added tree_path and doc_version for hierarchical search
fn build_schema() -> (Schema, Field, Field, Field, Field, Field, Field, Field) {
    let mut schema_builder = Schema::builder();
    let id_field = schema_builder.add_i64_field("id", STORED | INDEXED);
    let url_field = schema_builder.add_text_field("url", STRING | STORED);
    let title_field = schema_builder.add_text_field("title", TEXT | STORED);
    let content_field = schema_builder.add_text_field("content", TEXT | STORED);
    let domain_field = schema_builder.add_text_field("domain", STRING | STORED);
    // v0.2.0: New fields for hierarchical document structure
    let tree_path_field = schema_builder.add_text_field("tree_path", STRING | STORED);
    let doc_version_field = schema_builder.add_text_field("doc_version", STRING | STORED);

    let schema = schema_builder.build();
    (
        schema,
        id_field,
        url_field,
        title_field,
        content_field,
        domain_field,
        tree_path_field,
        doc_version_field,
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
    tree_path_field: Field,   // v0.2.0
    doc_version_field: Field, // v0.2.0
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new(data_dir: &Path) -> Result<Self, SearchError> {
        let index_path = data_dir.join("search_index");
        let (
            schema,
            id_field,
            url_field,
            title_field,
            content_field,
            domain_field,
            tree_path_field,
            doc_version_field,
        ) = build_schema();

        let index = if index_path.exists() {
            // Check if existing index needs upgrade (v0.2.0)
            match Index::open_in_dir(&index_path) {
                Ok(idx) => idx,
                Err(_) => {
                    // Index is corrupted or incompatible, recreate it
                    tracing::warn!("Search index needs rebuild, deleting...");
                    std::fs::remove_dir_all(&index_path).ok();
                    create_index(&index_path, schema)?
                }
            }
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
            tree_path_field,
            doc_version_field,
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
        self.index_document_with_tree(url, title, content, domain, None, None)
    }

    /// Index a document with hierarchical metadata
    /// v0.2.0: Added tree_path and doc_version support
    pub fn index_document_with_tree(
        &self,
        url: &str,
        title: &str,
        content: &str,
        domain: &str,
        tree_path: Option<&str>,
        doc_version: Option<&str>,
    ) -> Result<i64, SearchError> {
        let mut writer = self.writer.lock().unwrap();

        let id = {
            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            hasher.finish() as i64
        };

        let term = tantivy::Term::from_field_text(self.url_field, url);
        writer.delete_term(term);

        let mut doc = doc!(
            self.id_field => id,
            self.url_field => url,
            self.title_field => title,
            self.content_field => content,
            self.domain_field => domain,
        );

        // v0.2.0: Add hierarchical fields if available
        if let Some(path) = tree_path {
            doc.add_text(self.tree_path_field, path);
        }
        if let Some(version) = doc_version {
            doc.add_text(self.doc_version_field, version);
        }

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

            let tree_path = retrieved_doc
                .get_first(self.tree_path_field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let doc_version = retrieved_doc
                .get_first(self.doc_version_field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            results.push(SearchResult {
                id,
                url,
                title,
                domain,
                tree_path,
                doc_version,
            });
        }

        Ok(results)
    }

    /// Search with faceted filtering
    /// v0.2.0: Filter by tree_path or doc_version
    #[allow(dead_code)]
    pub fn search_with_facets(
        &self,
        query: &str,
        tree_path_filter: Option<&str>,
        version_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let searcher: Searcher = self.reader.searcher();

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.title_field, self.content_field]);

        let mut query_str = query.to_string();

        // Add facet filters if specified
        if let Some(path) = tree_path_filter {
            query_str = format!("{} +tree_path:{}", query_str, path);
        }
        if let Some(version) = version_filter {
            query_str = format!("{} +doc_version:{}", query_str, version);
        }

        let query = query_parser
            .parse_query(&query_str)
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

            let tree_path = retrieved_doc
                .get_first(self.tree_path_field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let doc_version = retrieved_doc
                .get_first(self.doc_version_field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            results.push(SearchResult {
                id,
                url,
                title,
                domain,
                tree_path,
                doc_version,
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

        writer
            .delete_all_documents()
            .map_err(|e| SearchError::Index(e.to_string()))?;
        writer
            .commit()
            .map_err(|e| SearchError::Index(e.to_string()))?;

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
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Tantivy document ID (URL hash, not SQLite row ID)
    #[allow(dead_code)]
    pub id: i64,
    /// Article URL (use this to fetch from SQLite)
    pub url: String,
    /// Article title
    #[allow(dead_code)]
    pub title: String,
    /// Article domain
    #[allow(dead_code)]
    pub domain: String,
    /// v0.2.0: Hierarchical tree path
    #[allow(dead_code)]
    pub tree_path: Option<String>,
    /// v0.2.0: Document version
    #[allow(dead_code)]
    pub doc_version: Option<String>,
}
