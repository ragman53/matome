# matome Implementation Plan

**Purpose**: Rust CLI tool that collects articles from specified URLs, translates to Japanese, and provides a local web portal for browsing.

**Last Updated**: 2026-04-08

---

## 1. Technology Stack (Confirmed)

| Layer | Technology | Reason |
|-------|-------------|--------|
| CLI Argument Parsing | clap | Standard Rust CLI crate, derive macro support |
| Configuration Files | toml, serde | TOML simplicity, de facto standard |
| HTTP Client | reqwest | Async, tokio integration, procedural API |
| HTML Parsing | scraper | CSS selectors, lightweight |
| Content Extraction | readability-rs | De facto for article extraction |
| Markdown Conversion | html2md | Simple HTML → MD conversion |
| Markdown → HTML | pulldown-cmark | CommonMark parsing for web rendering |
| SQLite | rusqlite | Synchronous, bundled SQLite |
| Full-Text Search | tantivy | Rust-native high-speed search engine |
| Web Server | axum | Lightweight, tower middleware support |
| Dynamic HTML | htmx | Server-rendered + partial updates |
| CSS | Tailwind CDN | No build required, CDN usage |

---

## 2. Implementation Status

### ✅ Phase 0: Foundation (COMPLETE)

| # | File | Status | Notes |
|---|------|--------|-------|
| 0-1 | `Cargo.toml` | ✅ | Dependencies defined |
| 0-2 | `src/main.rs` | ✅ | Entry point with tracing |
| 0-3 | `src/cli.rs` | ✅ | Commands: init, add, crawl, serve, status |
| 0-4 | `src/config.rs` | ✅ | Config parsing with multi-language support |

### ✅ Phase 1: Crawler (COMPLETE)

| # | File | Status | Notes |
|---|------|--------|-------|
| 1-1 | `src/pipeline/crawler.rs` | ✅ | HTTP fetch, sitemap/robots.txt parsing |
| 1-2 | `src/pipeline/mod.rs` | ✅ | Pipeline orchestration |

### ✅ Phase 2: Extraction (COMPLETE)

| # | File | Status | Notes |
|---|------|--------|-------|
| 2-1 | `src/pipeline/extractor.rs` | ✅ | readability-rs + html2md |

### ✅ Phase 3: Translation (COMPLETE)

| # | File | Status | Notes |
|---|------|--------|-------|
| 3-1 | `src/pipeline/translator.rs` | ✅ | Ollama/DeepL API client, code block preservation |
| 3-2 | `src/pipeline/glossary.rs` | ✅ | Multi-language glossary with term replacement |

### ✅ Phase 4: Storage & Search (COMPLETE)

| # | File | Status | Notes |
|---|------|--------|-------|
| 4-1 | `src/db/sqlite.rs` | ✅ | SQLite operations |
| 4-2 | `src/db/search.rs` | ✅ | Tantivy full-text search engine |
| 4-3 | `src/db/mod.rs` | ✅ | DB module exports |
| 4-4 | `src/db/error.rs` | ✅ | Error types |

### ✅ Phase 5: Web Server & UI (COMPLETE)

| # | File | Status | Notes |
|---|------|--------|-------|
| 5-1 | `src/web/mod.rs` | ✅ | Axum router, SearchEngine integration |
| 5-2 | `src/web/handlers.rs` | ✅ | All endpoints with full-text search |
| 5-3 | `templates/` | N/A | Using inline HTML (simple approach) |
| 5-4 | `assets/` | ✅ | Static directory |

---

## 3. Core Features

### 3.1 Data Pipeline Flow

```
Crawl → Extract → Translate → Apply Glossary → Store → Index
   ↓         ↓          ↓            ↓          ↓        ↓
 Raw    Markdown    Japanese     Terminology   SQLite   Tantivy
 HTML             Translation   Replacement
```

### 3.2 Glossary System

- **Multi-language support**: Terms can have translations for multiple languages
- **Language-specific replacement**: `Glossary::apply_for_lang(text, "ja")`
- **Backward compatible**: Legacy `ja` field still works
- **Case-insensitive matching**: `API` matches `api`, `Api`, etc.

**Example glossary.toml:**
```toml
[[terms]]
en = "compiler"
ja = "コンパイラ"

[[terms]]
en = "runtime"
translations = { ja = "ランタイム", zh = "运行时" }
```

### 3.3 Search Engine

- **Full-text search**: Tantivy-powered search in title and content
- **Fallback**: SQLite LIKE search if Tantivy unavailable
- **URL-based indexing**: Documents indexed by URL

---

## 4. Current File Structure

```
matome/
├── Cargo.toml
├── matome.toml              # Configuration file
├── glossary.example.toml     # Terminology glossary sample
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # CLI argument definitions
│   ├── config.rs            # Config parsing, multi-language types
│   ├── pipeline/
│   │   ├── mod.rs           # Pipeline orchestration (Glossary + Search integrated)
│   │   ├── crawler.rs      # HTTP fetch, sitemap parsing
│   │   ├── extractor.rs     # HTML→Markdown conversion
│   │   ├── translator.rs    # Ollama/DeepL translation
│   │   └── glossary.rs      # Multi-language glossary
│   ├── db/
│   │   ├── mod.rs           # DB module exports
│   │   ├── sqlite.rs        # SQLite operations
│   │   ├── search.rs        # Tantivy search engine
│   │   └── error.rs         # Error types
│   └── web/
│       ├── mod.rs           # Axum router + SearchEngine
│       └── handlers.rs      # All endpoints with full-text search
├── templates/               # (Not used - inline HTML)
└── assets/                  # Static files directory
```

---

## 5. CLI Commands

| Command | Description | Status |
|---------|-------------|--------|
| `matome init` | Generate config templates | ✅ |
| `matome add <url>` | Add domain to config | ✅ |
| `matome crawl [--incremental]` | Execute full pipeline | ✅ |
| `matome serve [--port <port>] [--host <host>]` | Start web server | ✅ |
| `matome status [--verbose]` | Display statistics | ✅ |

---

## 6. Configuration

### matome.toml

```toml
[core]
data-dir = ".matome"

[[domains]]
url = "https://docs.example.com/"
include = ["/**"]

[translate]
provider = "ollama"           # or "deepl", "none"
model = "translategemma:4bb"
target-lang = "ja"
glossary-file = "glossary.toml"

[crawl]
concurrency = 8
respect-robots = true
timeout = 30
max-pages = 0
```

### glossary.toml

```toml
[[terms]]
en = "compiler"
ja = "コンパイラ"

[[terms]]
en = "API"
translations = { ja = "API", zh = "API", ko = "API" }
```

---

## 7. Known Issues

### Build Warnings (20 warnings)

Some types and functions are defined but unused. Candidates for cleanup:
- `src/config.rs`: ConfigError, html_lang, Glossary, Article
- `src/pipeline/glossary.rs`: Some methods
- `src/db/`: Some unused methods and fields
- `src/web/`: Some unused variants

---

## 8. Recent Commits

| Commit | Description |
|--------|-------------|
| `df058c9` | feat: integrate glossary and search engine into pipeline |
| `c4284c4` | fix: Update default model to translategemma:latest |
| `73d24fe` | Initial commit |

---

*This file is updated according to implementation progress.*