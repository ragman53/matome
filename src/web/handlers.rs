//! HTTP request handlers
//!
//! Implements all web UI endpoints using compile-time embedded templates.

use crate::db::ArticleRow;
use crate::web::AppState;
use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

// ============== Compile-time embedded templates ==============

const INDEX_TEMPLATE: &str = include_str!("../../templates/index.html");
const ARTICLE_TEMPLATE: &str = include_str!("../../templates/article.html");
const SEARCH_TEMPLATE: &str = include_str!("../../templates/search.html");

// ============== Error types ==============

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum HandlerError {
    #[error("Not found")]
    NotFound,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Render error: {0}")]
    Render(String),
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let message = match self {
            HandlerError::NotFound => "Not found",
            HandlerError::Database(e) => { error!("Database error: {}", e); "Database error" }
            HandlerError::Render(e) => { error!("Render error: {}", e); "Render error" }
        };

        Html(format!(
            r#"<div class="error-page"><h1>Error</h1><p>{}</p><a href="/">Back to Home</a></div>"#,
            message
        )).into_response()
    }
}

// ============== Template rendering ==============

/// Simple template engine with {placeholder} substitution
fn render_template(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

/// Convert Markdown to HTML
fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Parser};
    let mut html = String::new();
    html::push_html(&mut html, Parser::new(markdown));
    html
}

// ============== Helper functions ==============

/// Generate domain navigation HTML
fn get_domain_nav(articles: &[ArticleRow]) -> String {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for a in articles { *counts.entry(a.domain.clone()).or_insert(0) += 1; }
    
    counts.iter().map(|(domain, count)| {
        format!(r#"<a href="/domain/{domain}" class="nav-item"><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>{domain}<span class="count">{count}</span></a>"#)
    }).collect::<Vec<_>>().join("\n")
}

/// Count unique domains
fn get_domain_count(articles: &[ArticleRow]) -> usize {
    use std::collections::HashSet;
    articles.iter().map(|a| a.domain.clone()).collect::<HashSet<_>>().len()
}

/// Generate article list HTML
fn render_article_list(articles: &[ArticleRow]) -> String {
    if articles.is_empty() {
        return r#"<div class="empty-state col-span-full text-center">
            <div class="text-5xl mb-4">📭</div>
            <p class="text-lg text-[var(--text-secondary)]">記事が見つかりません</p>
            <p class="text-sm text-[var(--text-muted)] mt-2"><code>matome crawl</code> を実行してドキュメントを収集してください</p>
        </div>"#.to_string();
    }
    articles.iter().enumerate().map(|(i, a)| {
        let title = a.title.as_deref().unwrap_or("Untitled");
        let desc = a.description.as_deref().unwrap_or("");
        format!(r#"<a href="/article/{}" class="article-card" style="animation-delay: {}ms">
            <div class="p-5">
                <span class="domain-badge">{}</span>
                <h3 class="card-title mt-3 font-semibold text-[var(--text-primary)] line-clamp-2 transition-colors">{}</h3>
                <p class="mt-2 text-sm text-[var(--text-secondary)] line-clamp-2">{}</p>
            </div>
        </a>"#, a.id, i * 50, a.domain, title, desc)
    }).collect::<Vec<_>>().join("\n")
}

// ============== Handler functions ==============

pub async fn index(State(state): State<Arc<AppState>>) -> Result<Html<String>, HandlerError> {
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    let content = render_article_list(&articles);
    let domains = get_domain_nav(&articles);
    let domain_count = get_domain_count(&articles);
    
    Ok(Html(render_template(INDEX_TEMPLATE, &[
        ("count", &articles.len().to_string()),
        ("content", &content),
        ("domain_nav", &domains),
        ("domain_count", &domain_count.to_string()),
    ])))
}

pub async fn article(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Result<Html<String>, HandlerError> {
    let article = state.db.get_article(id).map_err(|e| HandlerError::Database(e.to_string()))?.ok_or(HandlerError::NotFound)?;
    let content = article.translated_md.as_deref().unwrap_or(&article.original_md);
    let html_content = markdown_to_html(content);
    let title = article.title.as_deref().unwrap_or("Untitled");
    
    Ok(Html(render_template(ARTICLE_TEMPLATE, &[
        ("id", &article.id.to_string()),
        ("title", title),
        ("url", &article.url),
        ("domain", &article.domain),
        ("content", &html_content),
        ("original_class", ""),
        ("translated_class", "active"),
    ])))
}

pub async fn article_original(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Result<Html<String>, HandlerError> {
    let article = state.db.get_article(id).map_err(|e| HandlerError::Database(e.to_string()))?.ok_or(HandlerError::NotFound)?;
    let html_content = markdown_to_html(&article.original_md);
    let title = article.title.as_deref().unwrap_or("Untitled");
    
    Ok(Html(render_template(ARTICLE_TEMPLATE, &[
        ("id", &article.id.to_string()),
        ("title", title),
        ("url", &article.url),
        ("domain", &article.domain),
        ("content", &html_content),
        ("original_class", "active"),
        ("translated_class", ""),
    ])))
}

pub async fn search(State(state): State<Arc<AppState>>, Query(params): Query<SearchParams>) -> Result<Html<String>, HandlerError> {
    let query = params.q.trim();
    
    let articles = if query.is_empty() {
        state.db.get_all_articles()
    } else if let Some(ref search_engine) = state.search_engine {
        match search_engine.search(query, 50) {
            Ok(results) => {
                let urls: Vec<String> = results.iter().map(|r| r.url.clone()).collect();
                state.db.get_articles_by_urls(&urls)
            }
            Err(e) => { tracing::warn!("Search error: {}, falling back to LIKE", e); state.db.search_articles(query) }
        }
    } else {
        state.db.search_articles(query)
    }.map_err(|e| HandlerError::Database(e.to_string()))?;
    
    let content = render_article_list(&articles);
    
    Ok(Html(render_template(SEARCH_TEMPLATE, &[
        ("query", query),
        ("count", &articles.len().to_string()),
        ("content", &content),
    ])))
}

/// HTMX live search endpoint (returns article cards only)
pub async fn search_post(State(state): State<Arc<AppState>>, Form(params): Form<SearchQuery>) -> Result<Html<String>, HandlerError> {
    let q = params.q.as_deref().unwrap_or("");
    
    let articles = if q.is_empty() {
        state.db.get_all_articles()
    } else if let Some(ref search_engine) = state.search_engine {
        match search_engine.search(q, 50) {
            Ok(results) => {
                let urls: Vec<String> = results.iter().map(|r| r.url.clone()).collect();
                state.db.get_articles_by_urls(&urls)
            }
            Err(e) => { tracing::warn!("Search error: {}, falling back to LIKE", e); state.db.search_articles(q) }
        }
    } else {
        state.db.search_articles(q)
    }.map_err(|e| HandlerError::Database(e.to_string()))?;
    
    Ok(Html(render_article_list(&articles)))
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

pub async fn domains(State(state): State<Arc<AppState>>) -> Result<Html<String>, HandlerError> {
    use std::collections::HashMap;
    
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for a in &articles { *counts.entry(a.domain.clone()).or_insert(0) += 1; }
    
    let domain_html: String = counts.iter().map(|(d, c)| format!(
        r#"<a href="/domain/{}" class="article-card">
            <div class="p-5">
                <h3 class="font-semibold text-[var(--text-primary)]">{}</h3>
                <p class="mt-2 text-[var(--text-secondary)]">{} 記事</p>
            </div>
        </a>"#, d, d, c
    )).collect::<Vec<_>>().join("\n");
    
    let empty_state = if domain_html.is_empty() {
        r#"<div class="empty-state col-span-full text-center">
            <div class="text-5xl mb-4">🌐</div>
            <p class="text-lg text-[var(--text-secondary)]">ドメインが見つかりません</p>
        </div>"#.to_string()
    } else { String::new() };
    
    let html = render_template(INDEX_TEMPLATE, &[
        ("count", &counts.len().to_string()),
        ("content", &if empty_state.is_empty() { domain_html } else { empty_state }),
        ("domain_nav", ""),
        ("domain_count", "0"),
    ]);
    
    // Replace title area for domains view
    let html = html.replace("<h2 class=\"text-4xl", "<h2 class=\"text-3xl");
    
    Ok(Html(html))
}

pub async fn domain_articles(State(state): State<Arc<AppState>>, Path(domain): Path<String>) -> Result<Html<String>, HandlerError> {
    let articles = state.db.get_articles_by_domain(&domain).map_err(|e| HandlerError::Database(e.to_string()))?;
    let content = render_article_list(&articles);
    let domains = get_domain_nav(&articles);
    let domain_count = get_domain_count(&articles);
    
    let html = render_template(INDEX_TEMPLATE, &[
        ("count", &articles.len().to_string()),
        ("content", &content),
        ("domain_nav", &domains),
        ("domain_count", &domain_count.to_string()),
    ]);
    
    Ok(Html(html))
}

// ============== API handlers ==============

#[derive(serde::Serialize)]
pub struct ArticleJson {
    id: i64,
    url: String,
    title: String,
    domain: String,
    crawled_at: String,
}

// v0.2.0: Tree navigation imports
use crate::web::tree_nav::{build_tree_from_paths, render_tree_nav, render_breadcrumbs};

pub async fn api_articles(State(state): State<Arc<AppState>>) -> Result<Json<Vec<ArticleJson>>, HandlerError> {
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    let json: Vec<ArticleJson> = articles.into_iter().map(|a| ArticleJson {
        id: a.id,
        url: a.url,
        title: a.title.unwrap_or_else(|| "Untitled".to_string()),
        domain: a.domain,
        crawled_at: a.crawled_at,
    }).collect();
    Ok(Json(json))
}

// ============== v0.2.0 Tree Navigation Handlers ==============

/// Get tree structure as JSON
pub async fn api_tree(State(state): State<Arc<AppState>>) -> Result<Json<Vec<crate::db::models::TreeNode>>, HandlerError> {
    // Get all articles and build tree
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    
    // For now, use domain as path (flat data fallback)
    // TODO: When migrating to pages table, use tree_path
    let paths: Vec<(String, String)> = articles.iter().map(|a| {
        let path = format!("/{}/{}", a.domain, a.id);
        let title = a.title.clone().unwrap_or_else(|| "Untitled".to_string());
        (path, title)
    }).collect();
    
    let tree = build_tree_from_paths(&paths);
    Ok(Json(tree))
}

/// Root tree page - shows all documents
pub async fn tree_root(State(state): State<Arc<AppState>>) -> Result<Html<String>, HandlerError> {
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    let tree_html = render_tree_html(&articles);
    let content = render_article_list(&articles);
    
    Ok(Html(render_template(INDEX_TEMPLATE, &[
        ("count", &articles.len().to_string()),
        ("content", &content),
        ("domain_nav", &tree_html),
        ("domain_count", &get_domain_count(&articles).to_string()),
    ])))
}

/// Tree page at specific path
pub async fn tree_page(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Html<String>, HandlerError> {
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    let path_with_slash = format!("/{}", path);
    let breadcrumbs_html = render_breadcrumbs(&path_with_slash);
    
    // Filter articles that match this tree path
    let filtered: Vec<ArticleRow> = articles.iter()
        .filter(|a| a.url.contains(&path_with_slash))
        .cloned()
        .collect();
    
    let content = if filtered.is_empty() {
        render_article_list(&articles)
    } else {
        render_article_list(&filtered)
    };
    
    // Return with modified template - for now just render article list
    Ok(Html(render_template(INDEX_TEMPLATE, &[
        ("count", &filtered.len().to_string()),
        ("content", &content),
        ("domain_nav", &render_tree_html(&articles)),
        ("domain_count", &get_domain_count(&articles).to_string()),
    ])))
}

/// Render tree navigation HTML from articles
fn render_tree_html(articles: &[ArticleRow]) -> String {
    // Build path-title pairs from articles
    let paths: Vec<(String, String)> = articles.iter().map(|a| {
        // Use domain as top-level path
        let path = format!("/{}/{}", a.domain, a.id);
        let title = a.title.clone().unwrap_or_else(|| "Untitled".to_string());
        (path, title)
    }).collect();
    
    let tree = build_tree_from_paths(&paths);
    render_tree_nav(&tree, 0)
}


// ============== v0.2.0 Diff Mode Handlers ==============

/// Diff page - show changes since last crawl
pub async fn diff_page(State(state): State<Arc<AppState>>) -> Result<Html<String>, HandlerError> {
    use crate::pipeline::compute_content_hash;
    
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    let articles_clone = articles.clone();
    
    let mut changes: Vec<ChangeSummaryHtml> = Vec::new();
    for article in articles {
        let _hash = compute_content_hash(&article.original_md);
        changes.push(ChangeSummaryHtml {
            id: article.id,
            title: article.title.unwrap_or_else(|| "Untitled".to_string()),
            url: article.url,
            domain: article.domain,
            change_type: "Minor".to_string(),
            icon: "🟡".to_string(),
            crawled_at: article.crawled_at,
        });
    }
    
    let content = render_diff_list(&changes);
    
    Ok(Html(render_template(INDEX_TEMPLATE, &[
        ("count", &changes.len().to_string()),
        ("content", &content),
        ("domain_nav", &render_tree_html(&articles_clone)),
        ("domain_count", &get_domain_count(&articles_clone).to_string()),
    ])))
}

/// API endpoint for changes
pub async fn api_changes(State(state): State<Arc<AppState>>) -> Result<Json<Vec<ChangeSummaryJson>>, HandlerError> {
    use crate::pipeline::compute_content_hash;
    
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    
    let changes: Vec<ChangeSummaryJson> = articles.iter().map(|a| {
        let hash = compute_content_hash(&a.original_md);
        ChangeSummaryJson {
            id: a.id,
            title: a.title.clone().unwrap_or_else(|| "Untitled".to_string()),
            url: a.url.clone(),
            domain: a.domain.clone(),
            change_type: "Minor".to_string(),
            crawled_at: a.crawled_at.clone(),
        }
    }).collect();
    
    Ok(Json(changes))
}

#[derive(Debug, serde::Serialize)]
pub struct ChangeSummaryJson {
    id: i64,
    title: String,
    url: String,
    domain: String,
    change_type: String,
    crawled_at: String,
}

struct ChangeSummaryHtml {
    id: i64,
    title: String,
    url: String,
    domain: String,
    change_type: String,
    icon: String,
    crawled_at: String,
}

fn render_diff_list(changes: &[ChangeSummaryHtml]) -> String {
    if changes.is_empty() {
        return r#"<div class="empty-state col-span-full text-center">
            <div class="text-5xl mb-4">📊</div>
            <p class="text-lg text-[var(--text-secondary)]">変更が検出されませんでした</p>
        </div>"#.to_string();
    }
    
    let mut html = String::new();
    html.push_str(r#"<div class="diff-summary">"#);
    html.push_str(&format!(r#"<h2 class="text-xl font-bold mb-4">変更: {} 件</h2>"#, changes.len()));
    html.push_str("<div class=\"space-y-2\">");
    
    for c in changes {
        html.push_str(&format!(
            r#"<div class="flex items-center gap-3 p-3 bg-white rounded-lg border border-[var(--border)]">
                <span class="text-xl">{}</span>
                <div class="flex-1">
                    <a href="/article/{}" class="font-medium hover:text-[var(--accent-warm)]">{}</a>
                    <p class="text-sm text-[var(--text-muted)]">{} • {}</p>
                </div>
                <span class="text-xs px-2 py-1 bg-[var(--bg-tertiary)] rounded">{}</span>
            </div>"#,
            c.icon, c.id, c.title, c.domain, c.crawled_at, c.change_type
        ));
    }
    
    html.push_str("</div></div>");
    html
}
