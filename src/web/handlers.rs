//! HTTP request handlers
//!
//! Implements all web UI endpoints.

use crate::db::ArticleRow;
use crate::web::AppState;
use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

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

// ============== Template utilities ==============

/// Load template from file
fn load_template(name: &str) -> Option<String> {
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let template_path = template_dir.join(name);
    if template_path.exists() {
        std::fs::read_to_string(&template_path).ok()
    } else {
        None
    }
}

/// Simple template engine with {placeholder} substitution
fn render_template(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

// ============== Fetch articles ==============

fn fetch_articles(state: &Arc<AppState>, query: &str) -> Result<Vec<ArticleRow>, HandlerError> {
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
    Ok(articles)
}

// ============== Handler functions ==============

pub async fn index(State(state): State<Arc<AppState>>) -> Result<Html<String>, HandlerError> {
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    Ok(Html(render_index(&articles)))
}

pub async fn article(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Result<Html<String>, HandlerError> {
    let article = state.db.get_article(id).map_err(|e| HandlerError::Database(e.to_string()))?.ok_or(HandlerError::NotFound)?;
    Ok(Html(render_article(&article, false)))
}

pub async fn article_original(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Result<Html<String>, HandlerError> {
    let article = state.db.get_article(id).map_err(|e| HandlerError::Database(e.to_string()))?.ok_or(HandlerError::NotFound)?;
    Ok(Html(render_article(&article, true)))
}

#[derive(Deserialize)]
pub struct SearchQuery { q: Option<String> }

pub async fn search(State(state): State<Arc<AppState>>, Query(query): Query<SearchQuery>) -> Result<Html<String>, HandlerError> {
    let q = query.q.as_deref().unwrap_or("");
    let articles = fetch_articles(&state, q)?;
    Ok(Html(render_search(&articles, q)))
}

pub async fn search_post(State(state): State<Arc<AppState>>, Form(params): Form<SearchQuery>) -> Result<Html<String>, HandlerError> {
    let q = params.q.as_deref().unwrap_or("");
    let articles = fetch_articles(&state, q)?;
    Ok(Html(render_article_list(&articles)))
}

pub async fn domains(State(state): State<Arc<AppState>>) -> Result<Html<String>, HandlerError> {
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    Ok(Html(render_domains(&articles)))
}

pub async fn domain_articles(State(state): State<Arc<AppState>>, Path(domain): Path<String>) -> Result<Html<String>, HandlerError> {
    let articles = state.db.get_articles_by_domain(&domain).map_err(|e| HandlerError::Database(e.to_string()))?;
    let template = load_template("index.html").unwrap_or_else(|| get_inline_index_template().to_string());
    let content = render_article_list(&articles);
    let domain_count = articles.iter().map(|a| a.domain.clone()).collect::<std::collections::HashSet<_>>().len();
    let domain_nav = get_domain_nav(&articles);
    let rendered = render_template(&template, &[
        ("count", &articles.len().to_string()),
        ("content", &content),
        ("domain_nav", &domain_nav),
        ("domain_count", &domain_count.to_string()),
    ]);
    Ok(Html(rendered.replace("すべての記事", &format!("{} の記事", domain))))
}

pub async fn api_articles(State(state): State<Arc<AppState>>) -> Result<Json<Vec<ArticleJson>>, HandlerError> {
    let articles = state.db.get_all_articles().map_err(|e| HandlerError::Database(e.to_string()))?;
    Ok(Json(articles.into_iter().map(|a| ArticleJson {
        id: a.id, url: a.url, title: a.title.unwrap_or_default(), domain: a.domain, crawled_at: a.crawled_at
    }).collect()))
}

// ============== Template rendering ==============

fn render_index(articles: &[ArticleRow]) -> String {
    let template = load_template("index.html").unwrap_or_else(|| get_inline_index_template().to_string());
    let content = render_article_list(articles);
    let domains = get_domain_nav(articles);
    let domain_count = get_domain_count(articles);
    render_template(&template, &[
        ("count", &articles.len().to_string()),
        ("content", &content),
        ("domain_nav", &domains),
        ("domain_count", &domain_count.to_string()),
    ])
}

fn get_domain_nav(articles: &[ArticleRow]) -> String {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for a in articles { *counts.entry(a.domain.clone()).or_insert(0) += 1; }
    
    counts.iter().map(|(domain, count)| {
        format!(r#"<a href="/domain/{domain}" class="nav-item"><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>{domain}<span class="count">{count}</span></a>"#)
    }).collect::<Vec<_>>().join("
")
}

fn get_domain_count(articles: &[ArticleRow]) -> usize {
    use std::collections::HashSet;
    articles.iter().map(|a| a.domain.clone()).collect::<HashSet<_>>().len()
}

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

fn render_article(article: &ArticleRow, show_original: bool) -> String {
    let template = load_template("article.html").unwrap_or_else(|| get_inline_article_template().to_string());
    let content = if show_original { &article.original_md } else { article.translated_md.as_deref().unwrap_or(&article.original_md) };
    let html_content = markdown_to_html(content);
    let (original_class, translated_class) = if show_original { ("active", "") } else { ("", "active") };
    let title = article.title.as_deref().unwrap_or("Untitled");
    
    render_template(&template, &[
        ("id", &article.id.to_string()),
        ("title", title),
        ("url", &article.url),
        ("domain", &article.domain),
        ("content", &html_content),
        ("original_class", original_class),
        ("translated_class", translated_class),
    ])
}

fn render_search(articles: &[ArticleRow], query: &str) -> String {
    let template = load_template("search.html").unwrap_or_else(|| get_inline_search_template().to_string());
    let content = render_article_list(articles);
    render_template(&template, &[
        ("query", query),
        ("count", &articles.len().to_string()),
        ("content", &content),
    ])
}

fn render_domains(articles: &[ArticleRow]) -> String {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for a in articles { *counts.entry(a.domain.clone()).or_insert(0) += 1; }
    
    let domain_html: String = counts.iter().map(|(d, c)| format!(
        r#"<a href="/?domain={}" class="article-card" style="animation-delay: {}ms">
            <div class="p-5">
                <h3 class="font-semibold text-[var(--text-primary)]">{}</h3>
                <p class="mt-2 text-[var(--text-secondary)]">{} 記事</p>
            </div>
        </a>"#, d, 0, d, c
    )).collect::<Vec<_>>().join("\n");
    
    let html = if domain_html.is_empty() {
        r#"<div class="empty-state col-span-full text-center">
            <div class="text-5xl mb-4">🌐</div>
            <p class="text-lg text-[var(--text-secondary)]">ドメインが見つかりません</p>
        </div>"#.to_string()
    } else { domain_html };
    
    format!(r#"<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Domains - matome</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;600;700&family=IBM+Plex+Sans:wght@400;500;600&family=JetBrains+Mono:wght@400&display=swap" rel="stylesheet">
  <script src="https://cdn.tailwindcss.com"></script>
  <style>
    :root {{
      --bg-primary: #faf8f5;
      --bg-secondary: #ffffff;
      --bg-tertiary: #f5f1eb;
      --accent-warm: #e07a3a;
      --accent-cool: #3a6e8e;
      --text-primary: #2d2a26;
      --text-secondary: #6b6560;
      --text-muted: #9a958e;
      --border: #e5e0d8;
      --card-shadow: 0 4px 20px rgba(45, 42, 38, 0.08);
    }}
    body {{
      font-family: 'IBM Plex Sans', system-ui, sans-serif;
      background: var(--bg-primary);
      color: var(--text-primary);
    }}
    .font-display {{ font-family: 'Crimson Pro', Georgia, serif; }}
    .article-card {{
      background: var(--bg-secondary);
      border-radius: 16px;
      box-shadow: var(--card-shadow);
      transition: all 0.3s ease;
      border: 1px solid var(--border);
    }}
    .article-card:hover {{
      transform: translateY(-4px);
      box-shadow: 0 8px 30px rgba(45, 42, 38, 0.12);
    }}
    .empty-state {{
      background: var(--bg-secondary);
      border: 2px dashed var(--border);
      border-radius: 20px;
      padding: 4rem 2rem;
    }}
  </style>
</head>
<body>
  <header class="bg-white border-b border-[var(--border)] sticky top-0 z-50">
    <div class="max-w-7xl mx-auto px-6 py-4">
      <a href="/" class="flex items-center gap-2 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="19" y1="12" x2="5" y2="12"></line><polyline points="12 19 5 12 12 5"></polyline></svg>
        ホームに戻る
      </a>
    </div>
  </header>
  <main class="max-w-7xl mx-auto px-6 py-10">
    <h1 class="text-3xl font-display font-bold mb-8">ドメイン一覧</h1>
    <div class="grid gap-5 md:grid-cols-2 lg:grid-cols-3">{}</div>
  </main>
</body>
</html>"#, html)
}

fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Parser};
    let mut html = String::new();
    html::push_html(&mut html, Parser::new(markdown));
    html
}

// ============== Inline templates (fallback) ==============

fn get_inline_index_template() -> String {
    r#"<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>matome - ドキュメントポータル</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;600;700&family=IBM+Plex+Sans:wght@400;500;600&family=JetBrains+Mono:wght@400&display=swap" rel="stylesheet">
  <script src="https://cdn.tailwindcss.com"></script>
  <script src="https://unpkg.com/htmx.org@1.9.10"></script>
  <style>
    :root {
      --bg-primary: #faf8f5;
      --bg-secondary: #ffffff;
      --bg-tertiary: #f5f1eb;
      --accent-warm: #e07a3a;
      --accent-cool: #3a6e8e;
      --text-primary: #2d2a26;
      --text-secondary: #6b6560;
      --text-muted: #9a958e;
      --border: #e5e0d8;
      --card-shadow: 0 4px 20px rgba(45, 42, 38, 0.08);
    }
    body {
      font-family: 'IBM Plex Sans', system-ui, sans-serif;
      background: var(--bg-primary);
      color: var(--text-primary);
    }
    .font-display { font-family: 'Crimson Pro', Georgia, serif; }
    .article-card {
      background: var(--bg-secondary);
      border-radius: 16px;
      box-shadow: var(--card-shadow);
      transition: all 0.3s ease;
      border: 1px solid var(--border);
    }
    .article-card:hover {
      transform: translateY(-4px);
      box-shadow: 0 8px 30px rgba(45, 42, 38, 0.12);
    }
    .article-card:hover .card-title { color: var(--accent-warm); }
    .article-card { animation: fadeSlideUp 0.5s ease-out backwards; }
    @keyframes fadeSlideUp {
      from { opacity: 0; transform: translateY(20px); }
      to { opacity: 1; transform: translateY(0); }
    }
    .domain-badge {
      background: linear-gradient(135deg, var(--accent-cool), #4a8ab0);
      color: white;
      font-size: 0.7rem;
      padding: 0.35rem 0.75rem;
      border-radius: 20px;
      font-weight: 500;
    }
    .search-input {
      background: var(--bg-secondary);
      border: 2px solid var(--border);
      border-radius: 12px;
      padding: 0.875rem 1.25rem;
      transition: all 0.2s ease;
    }
    .search-input:focus {
      outline: none;
      border-color: var(--accent-warm);
      box-shadow: 0 0 0 4px rgba(224, 122, 58, 0.15);
    }
    .search-btn {
      background: linear-gradient(135deg, var(--accent-warm), #c86a30);
      color: white;
      border-radius: 12px;
      padding: 0.875rem 1.5rem;
      font-weight: 600;
      transition: all 0.2s ease;
      box-shadow: 0 4px 12px rgba(224, 122, 58, 0.3);
    }
    .search-btn:hover { transform: scale(1.02); }
    .empty-state {
      background: var(--bg-secondary);
      border: 2px dashed var(--border);
      border-radius: 20px;
      padding: 4rem 2rem;
    }
  </style>
</head>
<body>
  <header class="bg-white border-b border-[var(--border)] sticky top-0 z-50">
    <div class="max-w-7xl mx-auto px-6 py-4">
      <div class="flex items-center justify-between">
        <a href="/" class="flex items-center gap-3 group">
          <div class="w-12 h-12 bg-gradient-to-br from-[var(--accent-warm)] to-[var(--accent-cool)] rounded-xl flex items-center justify-center text-2xl">📖</div>
          <div>
            <h1 class="text-2xl font-display font-bold">matome</h1>
            <p class="text-sm text-[var(--text-muted)]">ドキュメントポータル</p>
          </div>
        </a>
      </div>
    </div>
  </header>
  <main class="max-w-7xl mx-auto px-6 py-10">
    <div class="text-center mb-10">
      <h2 class="text-4xl font-display font-bold mb-4">あなたの文献ライブラリ</h2>
      <p class="text-[var(--text-secondary)] max-w-xl mx-auto">ウェブから収集した有益なドキュメントを翻訳・整理</p>
    </div>
    <form action="/search" method="get" class="flex gap-3 max-w-2xl mx-auto mb-10">
      <input type="text" name="q" placeholder="キーワードで検索..." class="search-input flex-1">
      <button type="submit" class="search-btn">検索</button>
    </form>
    <div class="flex items-center justify-between mb-6">
      <h3 class="text-xl font-display font-semibold">すべての記事</h3>
      <p class="text-sm text-[var(--text-muted)]">{count} 件</p>
    </div>
    <div class="grid gap-5 md:grid-cols-2 lg:grid-cols-3">{content}</div>
  </main>
</body>
</html>"#.to_string()
}

fn get_inline_article_template() -> String {
    // Same template structure as index but for single article view
    r#"<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} - matome</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;600;700&family=IBM+Plex+Sans:wght@400;500;600&family=JetBrains+Mono:wght@400&display=swap" rel="stylesheet">
  <script src="https://cdn.tailwindcss.com"></script>
  <style>
    :root {
      --bg-primary: #faf8f5;
      --bg-secondary: #ffffff;
      --bg-tertiary: #f5f1eb;
      --accent-warm: #e07a3a;
      --accent-cool: #3a6e8e;
      --text-primary: #2d2a26;
      --text-secondary: #6b6560;
      --text-muted: #9a958e;
      --border: #e5e0d8;
    }
    * { box-sizing: border-box; }
    body {
      font-family: 'IBM Plex Sans', system-ui, sans-serif;
      background: var(--bg-primary);
      color: var(--text-primary);
      margin: 0;
    }
    .font-display { font-family: 'Crimson Pro', Georgia, serif; }
    .main-layout {
      display: flex;
      min-height: 100vh;
    }
    .sidebar {
      width: 280px;
      background: var(--bg-secondary);
      border-right: 1px solid var(--border);
      padding: 1.5rem;
      position: fixed;
      height: 100vh;
      overflow-y: auto;
    }
    .sidebar-title {
      font-family: 'Crimson Pro', serif;
      font-size: 1.25rem;
      font-weight: 700;
      margin-bottom: 1.5rem;
    }
    .nav-link {
      display: block;
      padding: 0.5rem 0;
      color: var(--text-secondary);
      text-decoration: none;
      font-size: 0.875rem;
    }
    .nav-link:hover { color: var(--accent-warm); }
    .content-area {
      margin-left: 280px;
      flex: 1;
      padding: 2rem 3rem;
    }
    .article-header {
      margin-bottom: 2rem;
    }
    .article-title {
      font-family: 'Crimson Pro', serif;
      font-size: 2rem;
      font-weight: 700;
      margin-bottom: 1rem;
    }
    .article-meta {
      display: flex;
      gap: 1.5rem;
      font-size: 0.875rem;
      color: var(--text-muted);
    }
    .article-meta a { color: var(--accent-cool); }
    .article-content {
      background: var(--bg-secondary);
      border-radius: 16px;
      padding: 2rem;
      border: 1px solid var(--border);
      line-height: 1.8;
    }
    .article-content h1, .article-content h2 { font-family: 'Crimson Pro', serif; }
    .article-content h2 { border-bottom: 2px solid var(--border); padding-bottom: 0.5rem; margin: 2rem 0 1rem; }
    .article-content h3 { margin: 1.5rem 0 0.75rem; }
    .article-content pre {
      background: #1e1e1e;
      color: #d4d4d4;
      padding: 1.25rem;
      border-radius: 12px;
      font-family: 'JetBrains Mono', monospace;
      overflow-x: auto;
    }
    .article-content code {
      background: var(--bg-tertiary);
      padding: 0.2rem 0.4rem;
      border-radius: 4px;
      font-family: 'JetBrains Mono', monospace;
      color: var(--accent-warm);
    }
    .article-content pre code { background: transparent; color: inherit; padding: 0; }
    .article-content blockquote {
      border-left: 4px solid var(--accent-warm);
      padding: 1rem 1.5rem;
      background: var(--bg-tertiary);
      border-radius: 0 12px 12px 0;
      margin: 1.5rem 0;
    }
    .toggle-group {
      display: flex;
      gap: 0.5rem;
      margin-top: 1rem;
    }
    .toggle-btn {
      padding: 0.5rem 1rem;
      border-radius: 8px;
      font-size: 0.875rem;
      text-decoration: none;
      background: var(--bg-tertiary);
      color: var(--text-secondary);
    }
    .toggle-btn:hover { background: var(--border); }
    .toggle-btn.active { background: var(--accent-cool); color: white; }
    .back-link {
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      color: var(--text-secondary);
      text-decoration: none;
      font-size: 0.875rem;
      margin-bottom: 1.5rem;
    }
    .back-link:hover { color: var(--accent-warm); }
    @media (max-width: 800px) {
      .sidebar { display: none; }
      .content-area { margin-left: 0; padding: 1.5rem; }
    }
  </style>
</head>
<body>
  <div class="main-layout">
    <aside class="sidebar">
      <a href="/" class="sidebar-title">📖 matome</a>
      <nav>
        <a href="/" class="nav-link">← すべての記事</a>
      </nav>
      <div class="toggle-group">
        <a href="/article/{id}" class="toggle-btn {translated_class}">翻訳</a>
        <a href="/article/{id}/original" class="toggle-btn {original_class}">原文</a>
      </div>
    </aside>
    <main class="content-area">
      <a href="/" class="back-link">← すべての記事に戻る</a>
      <header class="article-header">
        <h1 class="article-title">{title}</h1>
        <div class="article-meta">
          <a href="{url}" target="_blank">元記事を見る →</a>
          <span>{domain}</span>
        </div>
      </header>
      <article class="article-content">{content}</article>
    </main>
  </div>
</body>
</html>"#.to_string()
}

fn get_inline_search_template() -> String {
    r#"<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>検索結果: {query} - matome</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;600;700&family=IBM+Plex+Sans:wght@400;500;600&display=swap" rel="stylesheet">
  <script src="https://cdn.tailwindcss.com"></script>
  <style>
    :root {
      --bg-primary: #faf8f5;
      --bg-secondary: #ffffff;
      --bg-tertiary: #f5f1eb;
      --accent-warm: #e07a3a;
      --accent-cool: #3a6e8e;
      --text-primary: #2d2a26;
      --text-secondary: #6b6560;
      --text-muted: #9a958e;
      --border: #e5e0d8;
      --card-shadow: 0 4px 20px rgba(45, 42, 38, 0.08);
    }
    body {
      font-family: 'IBM Plex Sans', system-ui, sans-serif;
      background: var(--bg-primary);
      color: var(--text-primary);
    }
    .font-display { font-family: 'Crimson Pro', Georgia, serif; }
    .article-card {
      background: var(--bg-secondary);
      border-radius: 16px;
      box-shadow: var(--card-shadow);
      transition: all 0.3s ease;
      border: 1px solid var(--border);
    }
    .article-card:hover { transform: translateY(-4px); }
    .article-card { animation: fadeSlideUp 0.5s ease-out backwards; }
    @keyframes fadeSlideUp {
      from { opacity: 0; transform: translateY(20px); }
      to { opacity: 1; transform: translateY(0); }
    }
    .domain-badge {
      background: linear-gradient(135deg, var(--accent-cool), #4a8ab0);
      color: white;
      font-size: 0.7rem;
      padding: 0.35rem 0.75rem;
      border-radius: 20px;
      font-weight: 500;
    }
    .search-input {
      background: var(--bg-secondary);
      border: 2px solid var(--border);
      border-radius: 12px;
      padding: 0.875rem 1.25rem;
    }
    .search-input:focus { outline: none; border-color: var(--accent-warm); }
    .search-btn {
      background: linear-gradient(135deg, var(--accent-warm), #c86a30);
      color: white;
      border-radius: 12px;
      padding: 0.875rem 1.5rem;
      font-weight: 600;
    }
    .result-count {
      background: var(--bg-tertiary);
      padding: 0.5rem 1rem;
      border-radius: 20px;
    }
  </style>
</head>
<body>
  <header class="bg-white border-b border-[var(--border)] sticky top-0 z-50">
    <div class="max-w-7xl mx-auto px-6 py-4">
      <a href="/" class="flex items-center gap-2 text-[var(--text-secondary)]">← ホームに戻る</a>
    </div>
  </header>
  <main class="max-w-7xl mx-auto px-6 py-10">
    <h1 class="text-3xl font-display font-bold mb-6">検索結果</h1>
    <form action="/search" method="get" class="flex gap-3 max-w-2xl mx-auto mb-10">
      <input type="text" name="q" value="{query}" class="search-input flex-1">
      <button type="submit" class="search-btn">検索</button>
    </form>
    <div class="flex items-center justify-between mb-6">
      <h3 class="text-xl font-display font-semibold">検索結果</h3>
      <p class="result-count">{count} 件</p>
    </div>
    <div class="grid gap-5 md:grid-cols-2 lg:grid-cols-3">{content}</div>
  </main>
</body>
</html>"#.to_string()
}

// ============== API types ==============

#[derive(serde::Serialize)]
pub struct ArticleJson {
    id: i64, url: String, title: String, domain: String, crawled_at: String,
}
