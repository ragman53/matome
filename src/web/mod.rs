//! Web server module
//!
//! Handles HTTP serving with Axum and HTMX for the web UI.

mod handlers;
mod templates;
mod tree_nav;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ServerError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Template error: {0}")]
    Template(String),
}

use crate::db::Database;

/// Application state
pub struct AppState {
    pub db: Database,
    pub search_engine: Option<crate::db::SearchEngine>,
}

/// Create and configure the web server
pub fn create_app(data_dir: PathBuf) -> Result<Router, ServerError> {
    let db = Database::new(&data_dir)
        .map_err(|e| ServerError::Http(e.to_string()))?;

    // Initialize search engine
    let search_engine = match crate::db::SearchEngine::new(&data_dir) {
        Ok(se) => {
            info!("Search engine initialized");
            Some(se)
        }
        Err(e) => {
            tracing::warn!("Failed to initialize search engine: {}", e);
            None
        }
    };

    let state = Arc::new(AppState { db, search_engine });

    // Configure CORS for HTMX
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build routes
    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/article/:id", get(handlers::article))
        .route("/article/:id/original", get(handlers::article_original))
        .route("/search", get(handlers::search))
        .route("/search", post(handlers::search_post))
        .route("/domains", get(handlers::domains))
        .route("/domain/:domain", get(handlers::domain_articles))
        .route("/api/articles", get(handlers::api_articles))
        // v0.2.0: Tree navigation routes
        .route("/tree/*path", get(handlers::tree_page))
        .route("/tree", get(handlers::tree_root))
        .route("/api/tree", get(handlers::api_tree))
        .route("/api/pages", get(handlers::api_pages))  // v0.2.0: Pages API
        // v0.2.0: Diff Mode routes
        .route("/diff", get(handlers::diff_page))
        .route("/api/changes", get(handlers::api_changes))
        .layer(cors)
        .with_state(state);

    Ok(app)
}

/// Start the server
pub async fn run(port: u16, host: &str, data_dir: PathBuf) -> Result<(), ServerError> {
    let app = create_app(data_dir)?;

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| ServerError::Http(format!("Invalid address: {}", e)))?;

    info!("Starting server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| ServerError::Http(format!("Failed to bind: {}", e)))?;

    axum::serve(listener, app).await
        .map_err(|e| ServerError::Http(format!("Server error: {}", e)))?;

    Ok(())
}

/// Server wrapper for CLI
pub struct Server {
    data_dir: PathBuf,
}

impl Server {
    pub fn new(data_dir: &PathBuf) -> Result<Self, ServerError> {
        Ok(Self {
            data_dir: data_dir.clone(),
        })
    }

    pub async fn run(self, addr: (&str, u16)) -> Result<(), ServerError> {
        run(addr.1, addr.0, self.data_dir).await
    }
}
