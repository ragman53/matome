//! HTTP request handlers
//!
//! Implements all web UI endpoints.

use crate::db::Database;
use crate::web::AppState;
use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
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
        let (status, message) = match self {
            HandlerError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            HandlerError::Database(e) => {
                error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            HandlerError::Render(e) => {
                error!("Render error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Render error")
            }
        };

        Html(format!(
            r#"<div class="error-page">
                <h1>Error</h1>
                <p>{}</p>
                <a href="/">Back to Home</a>
            </div>"#,
            message
        ))
        .into_response()
    }
}

/// Home page - article list
pub async fn index(State(state): State<Arc<AppState>>) -> Result<Html<String>, HandlerError> {
    let articles = state.db.get_all_articles()
        .map_err(|e| HandlerError::Database(e.to_string()))?;

    let html = render_index(&articles);
    Ok(Html(html))
}

/// Article detail page
pub async fn article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Html<String>, HandlerError> {
    let article = state.db.get_article(id)
        .map_err(|e| HandlerError::Database(e.to_string()))?
        .ok_or(HandlerError::NotFound)?;

    let html = render_article(&article, false);
    Ok(Html(html))
}

/// Article original (untranslated) view
pub async fn article_original(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Html<String>, HandlerError> {
    let article = state.db.get_article(id)
        .map_err(|e| HandlerError::Database(e.to_string()))?
        .ok_or(HandlerError::NotFound)?;

    let html = render_article(&article, true);
    Ok(Html(html))
}

/// Search page
#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Html<String>, HandlerError> {
    let q = query.q.as_deref().unwrap_or("");
    let articles = if q.is_empty() {
        state.db.get_all_articles()
    } else {
        state.db.search_articles(q)
    }
    .map_err(|e| HandlerError::Database(e.to_string()))?;

    let html = render_search(&articles, q);
    Ok(Html(html))
}

/// Search POST handler (for HTMX)
pub async fn search_post(
    State(state): State<Arc<AppState>>,
    Form(params): Form<SearchQuery>,
) -> Result<Html<String>, HandlerError> {
    let q = params.q.as_deref().unwrap_or("");
    let articles = if q.is_empty() {
        state.db.get_all_articles()
    } else {
        state.db.search_articles(q)
    }
    .map_err(|e| HandlerError::Database(e.to_string()))?;

    Ok(Html(render_article_list(&articles)))
}

/// Domains overview
pub async fn domains(State(state): State<Arc<AppState>>) -> Result<Html<String>, HandlerError> {
    let articles = state.db.get_all_articles()
        .map_err(|e| HandlerError::Database(e.to_string()))?;

    let html = render_domains(&articles);
    Ok(Html(html))
}

/// API endpoint - JSON articles list
pub async fn api_articles(State(state): State<Arc<AppState>>) -> Result<Json<Vec<ArticleJson>>, HandlerError> {
    let articles = state.db.get_all_articles()
        .map_err(|e| HandlerError::Database(e.to_string()))?;

    let json: Vec<ArticleJson> = articles
        .into_iter()
        .map(|a| ArticleJson {
            id: a.id,
            url: a.url,
            title: a.title.unwrap_or_default(),
            domain: a.domain,
            crawled_at: a.crawled_at,
        })
        .collect();

    Ok(Json(json))
}

// ============== Template rendering ==============

fn render_index(articles: &[crate::db::ArticleRow]) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>matome - Documentation Portal</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
</head>
<body class="bg-gray-100 min-h-screen">
    <header class="bg-white shadow-sm">
        <div class="container mx-auto px-4 py-4">
            <h1 class="text-2xl font-bold text-gray-800">📚 matome</h1>
            <p class="text-gray-600">Documentation Portal</p>
        </div>
    </header>

    <main class="container mx-auto px-4 py-8">
        <div class="mb-6">
            <form action="/search" method="get" class="flex gap-2">
                <input type="text" name="q" placeholder="Search articles..."
                    class="flex-1 px-4 py-2 border rounded-lg">
                <button type="submit" class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700">
                    Search
                </button>
            </form>
        </div>

        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {content}
        </div>
    </main>
</body>
</html>"#,
        content = render_article_list(articles)
    )
}

fn render_article_list(articles: &[crate::db::ArticleRow]) -> String {
    if articles.is_empty() {
        return r#"<div class="col-span-full text-center py-8 text-gray-500">
            No articles found. Run <code>matome crawl</code> to collect documents.
        </div>"#.to_string();
    }

    articles
        .iter()
        .map(|article| {
            let title = article.title.as_deref().unwrap_or("Untitled");
            let domain = &article.domain;
            let description = article.description.as_deref().unwrap_or("");
            let id = article.id;

            format!(
                r#"<a href="/article/{id}" class="block bg-white rounded-lg shadow hover:shadow-lg transition p-4">
                    <span class="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">{domain}</span>
                    <h3 class="mt-2 font-semibold text-gray-800 line-clamp-2">{title}</h3>
                    <p class="mt-1 text-sm text-gray-600 line-clamp-2">{description}</p>
                </a>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_article(article: &crate::db::ArticleRow, show_original: bool) -> String {
    let title = article.title.as_deref().unwrap_or("Untitled");
    let url = &article.url;
    let domain = &article.domain;
    let content = if show_original {
        &article.original_md
    } else {
        article.translated_md.as_deref().unwrap_or(&article.original_md)
    };

    // Convert markdown to HTML using pulldown-cmark
    use pulldown_cmark::{html, Parser, Options};
    let parser = Parser::new(content);
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);

    format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - matome</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <style>
        .prose {{ max-width: 80ch; }}
        .prose h1 {{ font-size: 1.75rem; font-weight: bold; margin: 1.5rem 0 1rem; }}
        .prose h2 {{ font-size: 1.5rem; font-weight: bold; margin: 1.25rem 0 0.75rem; }}
        .prose h3 {{ font-size: 1.25rem; font-weight: bold; margin: 1rem 0 0.5rem; }}
        .prose p {{ margin: 0.75rem 0; line-height: 1.7; }}
        .prose pre {{ background: #1f2937; color: #e5e7eb; padding: 1rem; border-radius: 0.5rem; overflow-x: auto; }}
        .prose code {{ background: #f3f4f6; padding: 0.125rem 0.25rem; border-radius: 0.25rem; font-size: 0.875em; }}
        .prose pre code {{ background: transparent; padding: 0; }}
        .prose ul, .prose ol {{ margin: 0.75rem 0; padding-left: 1.5rem; }}
        .prose li {{ margin: 0.25rem 0; }}
        .prose blockquote {{ border-left: 4px solid #d1d5db; padding-left: 1rem; color: #6b7280; }}
        .prose a {{ color: #2563eb; text-decoration: underline; }}
        .prose table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}
        .prose th, .prose td {{ border: 1px solid #d1d5db; padding: 0.5rem; }}
        .prose th {{ background: #f3f4f6; }}
    </style>
</head>
<body class="bg-gray-100 min-h-screen">
    <header class="bg-white shadow-sm">
        <div class="container mx-auto px-4 py-4 flex justify-between items-center">
            <div>
                <a href="/" class="text-gray-600 hover:text-gray-800">Back to Home</a>
                <h1 class="text-xl font-bold text-gray-800 mt-1">{title}</h1>
            </div>
            <div class="flex gap-2">
                <a href="/article/{id}" class="px-3 py-1 text-sm {original_class} rounded">Original</a>
                <a href="/article/{id}" class="px-3 py-1 text-sm {translated_class} rounded">Translated</a>
            </div>
        </div>
    </header>

    <main class="container mx-auto px-4 py-8">
        <article class="prose bg-white rounded-lg shadow p-6">
            {html_content}
        </article>

        <div class="mt-4 text-sm text-gray-500">
            <p>Source: <a href="{url}" class="text-blue-600">{url}</a></p>
            <p>Domain: {domain}</p>
        </div>
    </main>
</body>
</html>"#,
        id = article.id,
        original_class = if show_original { "bg-gray-200" } else { "bg-gray-100" },
        translated_class = if !show_original { "bg-gray-200" } else { "bg-gray-100" },
        html_content = html_content,
    )
}

fn render_search(articles: &[crate::db::ArticleRow], query: &str) -> String {
    let results_count = articles.len();

    format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Search: {query} - matome</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
</head>
<body class="bg-gray-100 min-h-screen">
    <header class="bg-white shadow-sm">
        <div class="container mx-auto px-4 py-4">
            <a href="/" class="text-gray-600 hover:text-gray-800">Back to Home</a>
            <h1 class="text-2xl font-bold text-gray-800 mt-2">Search Results</h1>
            <p class="text-gray-600">Found {results_count} articles</p>
        </div>
    </header>

    <main class="container mx-auto px-4 py-8">
        <div class="mb-6">
            <form action="/search" method="get" class="flex gap-2">
                <input type="text" name="q" value="{query}" placeholder="Search articles..."
                    class="flex-1 px-4 py-2 border rounded-lg">
                <button type="submit" class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700">
                    Search
                </button>
            </form>
        </div>

        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {content}
        </div>
    </main>
</body>
</html>"#,
        query = query,
        content = render_article_list(articles),
    )
}

fn render_domains(articles: &[crate::db::ArticleRow]) -> String {
    use std::collections::HashMap;

    let mut domain_counts: HashMap<String, usize> = HashMap::new();
    for article in articles {
        *domain_counts.entry(article.domain.clone()).or_insert(0) += 1;
    }

    let domain_list: Vec<(String, usize)> = domain_counts.into_iter().collect();
    let domain_html = domain_list
        .iter()
        .map(|(domain, count)| {
            format!(
                r#"<a href="/?domain={domain}" class="block bg-white rounded-lg shadow p-4">
                    <h3 class="font-semibold text-gray-800">{domain}</h3>
                    <p class="text-gray-600">{count} articles</p>
                </a>"#,
                domain = domain,
                count = count
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Domains - matome</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
</head>
<body class="bg-gray-100 min-h-screen">
    <header class="bg-white shadow-sm">
        <div class="container mx-auto px-4 py-4">
            <a href="/" class="text-gray-600 hover:text-gray-800">Back to Home</a>
            <h1 class="text-2xl font-bold text-gray-800 mt-2">Domains</h1>
        </div>
    </header>

    <main class="container mx-auto px-4 py-8">
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {domain_html}
        </div>
    </main>
</body>
</html>"#,
        domain_html = if domain_html.is_empty() {
            r#"<div class="col-span-full text-center py-8 text-gray-500">No domains found.</div>"#.to_string()
        } else {
            domain_html
        },
    )
}

#[derive(serde::Serialize)]
pub struct ArticleJson {
    id: i64,
    url: String,
    title: String,
    domain: String,
    crawled_at: String,
}
