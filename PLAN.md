# matome Implementation Plan

**Purpose**: Rust CLI tool that collects articles from specified URLs, translates to Japanese, and provides a local web portal for browsing.

**Last Updated**: 2026-04-08

---

## 1. Technology Stack (Confirmed)

| Layer | Technology | Reason |
|-------|-------------|--------|
| CLI Argument Parsing | clap | Standard Rust CLI crate, derive macro support |
| Configuration Files | toml, serde | INI-like simplicity, de facto standard |
| HTTP Client | reqwest | Async, tokio integration, procedural API |
| HTML Parsing | scraper | CSS selectors, lightweight |
| Content Extraction | readability-rs | De facto for article extraction |
| Markdown Conversion | html2md | Simple HTML → MD conversion |
| Markdown → HTML | pulldown-cmark | CommonMark parsing for web rendering |
| SQLite | rusqlite | Synchronous, bundled SQLite |
| Full-Text Search | tantivy | Rust-native high-speed search engine |
| Japanese NLP | lindera | Morphological analysis (Tantivy Japanese support) |
| Web Server | axum | Lightweight, tower middleware support |
| Dynamic HTML | htmx | Server-rendered + partial updates |
| CSS | Tailwind CDN | No build required, CDN usage |

---

## 2. Implementation Status

### ✅ Phase 0: Foundation (COMPLETE)

**Goal**: Implement CLI command skeleton and configuration file parsing

| # | File | Status | Notes |
|---|------|--------|-------|
| 0-1 | `Cargo.toml` | ✅ | Dependencies defined |
| 0-2 | `src/main.rs` | ✅ | Entry point with tracing |
| 0-3 | `src/cli.rs` | ✅ | Commands: init, add, crawl, serve, status |
| 0-4 | `src/config.rs` | ⚠️ | Partial - types defined but some unused |

### ✅ Phase 1: Crawler (COMPLETE)

**Goal**: Fetch HTML from specified URLs, parse sitemap.xml/robots.txt

| # | File | Status | Notes |
|---|------|--------|-------|
| 1-1 | `src/pipeline/crawler.rs` | ✅ | HTTP fetch, sitemap/robots.txt parsing |
| 1-2 | `src/pipeline/mod.rs` | ✅ | Pipeline orchestration |

### ✅ Phase 2: Extraction (COMPLETE)

**Goal**: Extract main content from raw HTML and convert to clean Markdown

| # | File | Status | Notes |
|---|------|--------|-------|
| 2-1 | `src/pipeline/extractor.rs` | ✅ | readability-rs + html2md |

### ✅ Phase 3: Translation (COMPLETE)

**Goal**: Translate Markdown to Japanese via Ollama API

| # | File | Status | Notes |
|---|------|--------|-------|
| 3-1 | `src/pipeline/translator.rs` | ✅ | Ollama API client |
| 3-2 | `src/pipeline/glossary.rs` | ⚠️ | Defined but **NOT integrated** into pipeline |

### ⚠️ Phase 4: Storage (COMPLETE, SEARCH NOT INTEGRATED)

**Goal**: Data storage and Japanese full-text search with SQLite + Tantivy

| # | File | Status | Notes |
|---|------|--------|-------|
| 4-1 | `src/db/sqlite.rs` | ✅ | SQLite operations |
| 4-2 | `src/db/search.rs` | ⚠️ | **Defined but NEVER USED** - search uses SQLite LIKE instead |
| 4-3 | `src/db/mod.rs` | ✅ | DB initialization |
| 4-4 | `src/db/error.rs` | ✅ | Error types |

### ⚠️ Phase 5: Web Server & UI (PARTIAL)

**Goal**: Lightweight web browsing UI with Axum + HTMX

| # | File | Status | Notes |
|---|------|--------|-------|
| 5-1 | `src/web/mod.rs` | ✅ | Axum router setup |
| 5-2 | `src/web/handlers.rs` | ⚠️ | Works but uses inline HTML, no templates |
| 5-3 | `templates/` | ❌ | **EMPTY** - templates directory exists but no files |
| 5-4 | `assets/` | ✅ | Static directory exists |
| 5-5 | `src/web/templates.rs` | ⚠️ | **Never used** - handlers render HTML inline |

---

## 3. Critical Issues to Fix

### Issue #1: 27 Build Warnings - Dead Code

Many types and functions are defined but never used. This indicates:
- Features are partially implemented but not connected
- Code complexity without value
- Potential refactoring needed

**Affected modules:**
- `src/config.rs`: GlossaryTerm, Glossary, Article (never constructed)
- `src/pipeline/glossary.rs`: Full module defined but never used
- `src/db/search.rs`: SearchEngine, SearchResult, SearchError (never used)
- `src/db/sqlite.rs`: get_articles_by_domain, delete_article, clear (never used)
- `src/web/`: AppState::data_dir, ServerError::Template, HandlerError::Render, load_template

### Issue #2: Search Engine Not Integrated

The Tantivy search engine in `src/db/search.rs` is defined but completely unused.
- Current search uses SQLite LIKE queries
- Full-text search with Japanese support not working
- SearchEngine methods are all dead code

### Issue #3: Glossary Not Integrated

The glossary module in `src/pipeline/glossary.rs` is defined but never imported or used in the translation pipeline.
- Config has `glossary_file` option but it's not loaded
- Glossary replacement doesn't happen during translation

### Issue #4: Templates Directory Empty

The `templates/` directory exists but contains no Askama template files.
- Handlers render HTML inline using format! macro
- Should migrate to proper template files for maintainability

### Issue #5: Web UI Toggle Bug

In `handlers.rs`, the article page toggle buttons both link to the same path:
```rust
// Current (WRONG):
<a href="/article/{id}" class="px-3 py-1...">Original</a>
<a href="/article/{id}" class="px-3 py-1...">Translated</a>

// Should be:
<a href="/article/{id}/original" class="...">Original</a>
<a href="/article/{id}" class="...">Translated</a>
```

---

## 4. Current File Structure

```
matome/
├── Cargo.toml
├── matome.toml              # Configuration file
├── glossary.example.toml    # Terminology glossary sample
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # CLI argument definitions
│   ├── config.rs            # Config file loading (PARTIAL)
│   ├── pipeline/
│   │   ├── mod.rs           # Pipeline orchestration
│   │   ├── crawler.rs       # HTTP fetch, sitemap parsing
│   │   ├── extractor.rs     # HTML→Markdown conversion
│   │   ├── translator.rs    # Ollama translation
│   │   └── glossary.rs      # ⚠️ DEFINED BUT NEVER USED
│   ├── db/
│   │   ├── mod.rs           # DB initialization
│   │   ├── sqlite.rs        # SQLite operations
│   │   ├── search.rs        # ⚠️ DEFINED BUT NEVER USED
│   │   └── error.rs         # DB errors
│   └── web/
│       ├── mod.rs           # Axum router
│       ├── handlers.rs       # ⚠️ INLINE HTML, has toggle bug
│       └── templates.rs      # ⚠️ NEVER USED
├── templates/               # ❌ EMPTY - should have Askama templates
└── assets/                  # Static files directory
```

---

## 5. CLI Commands (Implemented)

| Command | Description | Status |
|---------|-------------|--------|
| `matome init` | Generate templates for `matome.toml` and `glossary.toml` | ✅ |
| `matome add <url>` | Add target domain to `matome.toml` | ✅ |
| `matome crawl [--incremental]` | Execute pipeline | ✅ |
| `matome serve [--port <port>] [--host <host>]` | Start local web server | ✅ |
| `matome status [--verbose]` | Display DB statistics | ✅ |

---

## 6. Next Actions Priority

### 🔴 Priority 1: Fix Critical Integration Issues

1. **Integrate Tantivy Search Engine**
   - Connect SearchEngine to pipeline for indexing
   - Replace SQLite LIKE with full-text search
   - Add Japanese morphological analysis support

2. **Integrate Glossary into Translation Pipeline**
   - Load glossary from config file
   - Apply term replacements during translation

3. **Fix Web UI Toggle Bug**
   - Correct article view toggle links

### 🟡 Priority 2: Clean Up Dead Code

1. **Remove or Use Unused Types**
   - Decide: remove unused types or implement their missing functionality
   - Clean up build warnings for maintainability

2. **Migrate to Template Files (Optional)**
   - Create proper Askama templates
   - Or remove template infrastructure if inline HTML is preferred

### 🟢 Priority 3: Testing & Polish

1. **End-to-End Testing**
   - Test crawl with real domain
   - Test web server functionality
   - Verify translation quality

2. **Performance Optimization**
   - Tune concurrency settings
   - Add progress indicators

---

## 7. Recent Commits

| Commit | Description |
|--------|-------------|
| `c4284c4` | fix: Update default model to translategemma:latest and refactor imports |
| `73d24fe` | Initial commit: matome - Rust CLI for documentation collection and translation |

---

*This file is updated according to implementation progress. Update last modified date when changing.*