# matome Implementation Plan

**Purpose**: Rust CLI tool that collects articles from specified URLs, translates to Japanese, and provides a local web portal for browsing.

**Last Updated**: 2026-04-07

---

## 1. Technology Stack (Confirmed)

| Layer | Technology | Reason |
|-------|-------------|--------|
| CLI Argument Parsing | clap | Standard Rust CLI crate, derive macro support |
| Configuration Files | toml, serde | INI-like simplicity, de facto standard |
| HTTP Client | reqwest | Async, tokio integration, procedural API |
| HTML Parsing | scraper | CSS selectors, lightweight vs zerocalorie |
| Content Extraction | readability-rs | De facto for article extraction |
| Markdown Conversion | html2md | Simple HTML → MD conversion |
| Markdown → HTML | comrak | CommonMark + extended syntax support |
| SQLite | rusqlite | Synchronous, bundled SQLite |
| Full-Text Search | tantivy | Rust-native high-speed search engine |
| Japanese NLP | lindera | Morphological analysis (Tantivy Japanese support) |
| Web Server | axum | Lightweight, tower middleware support |
| Templates | askama | Rust-native, safe, derive macro |
| Dynamic HTML | htmx | Server-rendered + partial updates |
| CSS | Tailwind CDN | No build required, CDN usage |

**Dependency Crates (Cargo.toml Initial Values)**

```toml
[dependencies]
# CLI & Config
clap = { version = "4", features = ["derive"] }
toml = "0.8"
serde = { version = "1", features = ["derive"] }

# Async & Network
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }

# HTML/Markdown
scraper = "0.21"
readability-rs = "0.1"
html2md = "0.5"

# Database
rusqlite = { version = "0.32", features = ["bundled"] }
tantivy = "0.22"
tantivy-query = "0.22"
lindera = { version = "0.25", features = ["default-dictionary"] }

# Web Server
axum = "0.7"
askama = "0.12"
comrak = "0.44"

# Utilities
futures = "0.3"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
url = "2"
sitemap = "0.4"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
```

---

## 2. Phase-by-Phase Implementation Plan

### Phase 0: Foundation (CLI & Configuration Base)

**Goal**: Implement CLI command skeleton and configuration file parsing

| # | File | Content | Notes |
|---|------|---------|-------|
| 0-1 | `Cargo.toml` | Dependency definitions | All crates managed here |
| 0-2 | `src/main.rs` | Entry point, panic handler | Log initialization here |
| 0-3 | `src/cli.rs` | CLI argument definitions with clap | Init/Add/Crawl/Serve/Status |
| 0-4 | `src/config.rs` | matome.toml loading & type definitions | `Config`, `Domain`, `Translate`, `Crawl` |

**Deliverables**: `matome --help` shows help, `matome init` generates config files

**Tests**: Unit tests for CLI argument parsing

```
src/
├── main.rs      # panic hook, tracing init, routing
├── cli.rs       # clap derive(Subcommand, Args)
└── config.rs    # Config struct, load(), init()
```

---

### Phase 1: Crawler (Web Traversal)

**Goal**: Fetch HTML from specified URLs, parse sitemap.xml/robots.txt

| # | File | Content | Notes |
|---|------|---------|-------|
| 1-1 | `src/pipeline/crawler.rs` | HTTP fetch, sitemap parsing | reqwest + sitemap crate |
| 1-2 | `src/pipeline/mod.rs` | Pipeline orchestration functions | High-level API calling crawler |

**Function Design**:

```rust
// Input: Start URL + include patterns
// Output: Vec<Page> { url, raw_html }
pub async fn crawl_domain(start_url: &str, include: &[String]) -> Result<Vec<Page>>
```

**Tests**: Unit tests for sitemap.xml parsing, robots.txt compliance

---

### Phase 2: Extraction (HTML → Markdown Conversion)

**Goal**: Extract main content from raw HTML and convert to clean Markdown

| # | File | Content | Notes |
|---|------|---------|-------|
| 2-1 | `src/pipeline/extractor.rs` | HTML→Markdown conversion pipeline | readability → scraper → html2md |
| 2-2 | `tests/extractor.rs` | Property tests for HTML→MD conversion | Verify readability accuracy |

**Function Design**:

```rust
// Input: raw_html (String)
// Output: ExtractedPage { url, markdown, title, description }
pub fn extract_markdown(raw_html: &str, url: &str) -> Result<ExtractedPage>
```

**Tests**: Conversion tests from various HTML structures (edge cases)

---

### Phase 3: Translation

**Goal**: Translate Markdown to Japanese, term replacement with glossary

| # | File | Content | Notes |
|---|------|---------|-------|
| 3-1 | `src/pipeline/translator.rs` | Ollama/DeepL/LibreTranslate clients | Abstracted with trait |
| 3-2 | `src/pipeline/glossary.rs` | glossary.toml parsing & term replacement | Local translation |
| 3-3 | `glossary.example.toml` | Terminology glossary sample | User customization |

**Trait Design**:

```rust
pub trait Translator {
    async fn translate(&self, md: &str) -> Result<String>;
}

pub struct OllamaTranslator { ... }
pub struct DeepLTranslator { ... }
pub struct LocalGlossaryTranslator { ... }
```

**Function Design**:

```rust
// Input: markdown + Translator trait object
// Output: translated_markdown
pub async fn translate_markdown(
    md: &str,
    translator: &dyn Translator,
    glossary: &Glossary,
) -> Result<String>
```

**Tests**: Mock tests for Ollama/DeepL clients (integration test)

---

### Phase 4: Storage & Search (Persistence)

**Goal**: Data storage and Japanese full-text search with SQLite + Tantivy

| # | File | Content | Notes |
|---|------|---------|-------|
| 4-1 | `src/db/sqlite.rs` | SQLite operations | Metadata, MD storage |
| 4-2 | `src/db/search.rs` | Tantivy + Lindera index | Japanese support |
| 4-3 | `src/db/mod.rs` | DB initialization functions | Table creation, index building |

**Table Design**:

```sql
CREATE TABLE articles (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    url           TEXT UNIQUE NOT NULL,
    title         TEXT,
    description   TEXT,
    original_md   TEXT NOT NULL,
    translated_md TEXT,
    domain        TEXT NOT NULL,
    crawled_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_articles_domain ON articles(domain);
CREATE INDEX idx_articles_url ON articles(url);
```

**Tests**: SQLite CRUD tests, Tantivy search tests (integration test)

---

### Phase 5: Web Server & UI (Browsing UI)

**Goal**: Lightweight web browsing UI with Axum + HTMX

| # | File | Content | Notes |
|---|------|---------|-------|
| 5-1 | `src/web/mod.rs` | Axum router configuration | `/`, `/article/:id`, `/search` |
| 5-2 | `src/web/handlers.rs` | Endpoint implementations | Includes HTML rendering |
| 5-3 | `templates/` | Askama templates | List, detail, search views |
| 5-4 | `assets/` | Tailwind CDN link, custom CSS | |

**Endpoints**:

| Path | Method | Description |
|------|--------|-------------|
| `/` | GET | Domain-grouped article list |
| `/article/:id` | GET | Translated MD display |
| `/search` | GET | Japanese full-text search |
| `/article/:id/original` | GET | Original language display |

**Tests**: HTTP tests for endpoints (axum-test)

---

### Phase 6: Polish (Quality Improvement)

**Goal**: Incremental updates, enhanced error handling, bug fixes

| # | Task | Content |
|---|------|---------|
| 6-1 | Incremental Crawl | `crawl --incremental` fetches only differences from last run |
| 6-2 | Error Handling | thiserror expansion, full error cause display |
| 6-3 | CLI=status | DB/Index statistics display |
| 6-4 | Logging | tracing introduction, structured logging |

**Incremental Strategy**:

```rust
// 1. Get saved URL list from DB
// 2. Only new URLs become crawl targets
// 3. Update detection: ETag/Last-Modified based diff detection
pub async fn crawl_incremental(config: &Config) -> Result<CrawlReport>
```

---

## 3. File Creation Order (Dependency Order)

```
Phase 0
  ├── Cargo.toml
  ├── src/main.rs
  ├── src/cli.rs
  └── src/config.rs

Phase 1 (depends on Phase 0)
  ├── src/pipeline/
  │   ├── mod.rs
  │   └── crawler.rs

Phase 2 (depends on Phase 1)
  └── src/pipeline/extractor.rs

Phase 3 (depends on Phase 2)
  ├── src/pipeline/translator.rs
  ├── src/pipeline/glossary.rs
  └── glossary.example.toml

Phase 4 (depends on Phase 3)
  ├── src/db/
  │   ├── mod.rs
  │   ├── sqlite.rs
  │   └── search.rs

Phase 5 (depends on Phase 4)
  ├── src/web/
  │   ├── mod.rs
  │   ├── handlers.rs
  ├── templates/
  └── assets/

Phase 6 (depends on all phases)
  └── Incremental update, log enhancement
```

**Principle**: Lower phases do not call higher phases. Each module only cares about input and output.

---

## 4. Test Strategy

| Phase | Test Type | Target |
|-------|-----------|--------|
| Phase 0 | Unit Test | CLI argument parsing, config file parsing |
| Phase 1 | Unit + Integration | sitemap parsing, robots.txt compliance |
| Phase 2 | Property Test | HTML→MD conversion accuracy |
| Phase 3 | Mock Test | Ollama/DeepL clients |
| Phase 4 | Integration | SQLite CRUD, Tantivy search |
| Phase 5 | HTTP Test | Axum endpoints |
| Phase 6 | E2E | Full pipeline execution |

**Test URL (for debugging)**:

```toml
[[domain]]
url = "https://example.com"
include = ["/**"]
```

---

## 5. Final Directory Structure

```
matome/
├── Cargo.toml
├── glossary.example.toml     # Terminology glossary sample
├── .gitignore
├── src/
│   ├── main.rs               # Entry point
│   ├── cli.rs                # CLI argument definitions
│   ├── config.rs             # Config file loading
│   ├── pipeline/
│   │   ├── mod.rs            # Orchestration
│   │   ├── crawler.rs       # HTTP fetch, sitemap parsing
│   │   ├── extractor.rs      # HTML→Markdown conversion
│   │   ├── translator.rs     # Translation clients
│   │   └── glossary.rs      # Glossary parsing & replacement
│   ├── db/
│   │   ├── mod.rs            # DB initialization
│   │   ├── sqlite.rs         # SQLite operations
│   │   └── search.rs         # Tantivy search
│   └── web/
│       ├── mod.rs            # Axum router
│       ├── handlers.rs       # Endpoints
│       └── templates.rs     # Askama template loader
├── templates/                # Askama HTML templates
│   ├── base.html
│   ├── index.html
│   ├── article.html
│   └── search.html
└── assets/                   # Static files
    └── style.css             # Tailwind + custom
```

---

## 6. Refactoring Criteria

### When to Extract Functions/Modules

- **Same processing appears in 2+ places** → Extract to function
- **Function exceeds 20 lines** → Consider splitting
- **Implementing multiple traits** → Consider strategy pattern

### Testability

- **Side effects outside function** → Make input args, output return values
- **Using global state** → Convert to Context struct

### Performance

- **Async function called synchronously** → Separate with `tokio::spawn`
- **Returning large Vec** → Consider changing to `Stream`

### Acceptable Trade-offs (v1)

- Circular references (shouldn't occur)
- Dynamic library support
- WASM builds

---

## 7. Next Actions

**Ready to start**: Phase 0 (Foundation) implementation

```
1. Create Cargo.toml
2. Create src/main.rs skeleton
3. Define clap in src/cli.rs
4. Define config file types in src/config.rs
5. Verify `matome init` command works
```

After Phase 0 is complete, commit. Then proceed to Phase 1.

---

*This file is updated according to implementation progress. Update last modified date when changing.*
