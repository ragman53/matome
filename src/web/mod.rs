//! Web server module
//!
//! Handles HTTP serving with Axum and HTMX for the web UI.

mod handlers;
mod templates;

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
pub enum ServerError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Template error: {0}")]
    Template(String),
}

/// Application state
pub struct AppState {
    pub data_dir: PathBuf,
    pub db: crate::db::Database,
}

/// Create and configure the web server
pub fn create_app(data_dir: PathBuf) -> Result<Router, ServerError> {
    let db = crate::db::Database::new(&data_dir)
        .map_err(|e| ServerError::Http(e.to_string()))?;

    let state = Arc::new(AppState { data_dir, db });

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
        .route("/api/articles", get(handlers::api_articles))
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
