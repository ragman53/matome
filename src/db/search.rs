//! Full-text search engine using Tantivy
//!
//! Handles Japanese full-text search indexing and querying.

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

        // Build schema
        let mut schema_builder = Schema::builder();

        let id_field = schema_builder.add_i64_field("id", STORED | INDEXED);
        let url_field = schema_builder.add_text_field("url", STRING | STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let domain_field = schema_builder.add_text_field("domain", STRING | STORED);

        let schema = schema_builder.build();

        // Create or open index
        let index = if index_path.exists() {
            Index::open_in_dir(&index_path).map_err(|e| SearchError::Index(e.to_string()))?
        } else {
            std::fs::create_dir_all(&index_path)?;
            Index::create_in_dir(&index_path, schema.clone())
                .map_err(|e| SearchError::Index(e.to_string()))?
        };

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| SearchError::Index(e.to_string()))?;

        let writer = index
            .writer(50_000_000)
            .map_err(|e| SearchError::Index(e.to_string()))?;

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

    /// Index a document
    pub fn index_document(
        &self,
        id: i64,
        url: &str,
        title: &str,
        content: &str,
        domain: &str,
    ) -> Result<(), SearchError> {
        let mut writer = self.writer.lock().unwrap();

        // Delete existing document with same ID
        let term = tantivy::Term::from_field_i64(self.id_field, id);
        writer.delete_term(term);

        // Add new document
        let mut doc = doc!(
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

        Ok(())
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

    /// Get document count
    pub fn doc_count(&self) -> Result<u64, SearchError> {
        let searcher: Searcher = self.reader.searcher();
        Ok(searcher.num_docs())
    }
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub domain: String,
}
