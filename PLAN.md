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
| Markdown → HTML | pulldown-cmark | CommonMark parsing for web rendering |
| SQLite | rusqlite | Synchronous, bundled SQLite |
| Full-Text Search | tantivy | Rust-native high-speed search engine |
| Web Server | axum | Lightweight, tower middleware support |
| Dynamic HTML | htmx | Server-rendered + partial updates |
| CSS | Tailwind CDN | No build required, CDN usage |
| Fonts | Google Fonts | Crimson Pro, IBM Plex Sans, JetBrains Mono |

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
| 2-1 | `src/pipeline/extractor.rs` | ✅ | scraper + custom HTML→MD conversion |

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
| 5-3 | `templates/` | ✅ | HTML templates for sidebar + grid layout |
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

### 3.2 Web UI Design

**Documentation Portal Layout** with sidebar navigation:

```
┌──────────────────────────────────────────────────────┐
│ ┌─────────┐ ┌────────────────────────────────────┐   │
│ │ LOGO    │ │ Breadcrumb          🔍 [Search]     │   │
│ │─────────│ │────────────────────────────────────│   │
│ │ 🔍 Search│ │ Stats: X articles, Y domains       │   │
│ │─────────│ │────────────────────────────────────│   │
│ │ Overview│ │ ┌────────┐ ┌────────┐ ┌────────┐  │   │
│ │  Home   │ │ │Article │ │Article │ │Article │  │   │
│ │  Search │ │ │ Card   │ │ Card   │ │ Card   │  │   │
│ │─────────│ │ └────────┘ └────────┘ └────────┘  │   │
│ │ Domains │ │ ┌────────┐ ┌────────┐              │   │
│ │  📖 md  │ │ │Article │ │Article │              │   │
│ │  📖 moz │ │ │ Card   │ │ Card   │              │   │
│ └─────────┘ │ └────────┘ └────────┘              │   │
│   Sidebar   │         Main Content                │   │
└─────────────┴────────────────────────────────────┘   │
```

**Article Reading View** with sidebar + content layout:

```
┌──────────────────────────────────────────────────────┐
│ ┌─────────────┐ ┌────────────────────────────────┐  │
│ │ 📖 matome   │ │ ← Back to all articles         │  │
│ │─────────────│ │                                │  │
│ │ ← All       │ │ Article Title                  │  │
│ │─────────────│ │ ─────────────────────────────  │  │
│ │ Language:   │ │ [翻訳] [原文]                  │  │
│ │ [翻訳][原文]│ │                                │  │
│ └─────────────┘ │ Article content in rendered    │  │
│    Sidebar      │ Markdown with styling...       │  │
└─────────────────┴────────────────────────────────┘  │
```

### 3.3 Search Features

- **Quick Search Modal**: Press ⌘K anywhere to open search overlay
- **Live Results**: HTMX-powered live search with debouncing
- **Keyboard Shortcuts**: ⌘K (search), Escape (close)
- **Domain Filtering**: Click domain in sidebar to filter articles

### 3.4 Glossary System

- **Multi-language support**: Terms can have translations for multiple languages
- **Language-specific replacement**: `Glossary::apply_for_lang(text, "ja")`
- **Backward compatible**: Legacy `ja` field still works
- **Case-insensitive matching**: `API` matches `api`, `Api`, etc.

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
│   │   ├── mod.rs           # Pipeline orchestration
│   │   ├── crawler.rs       # HTTP fetch, sitemap parsing
│   │   ├── extractor.rs     # HTML→Markdown conversion
│   │   ├── translator.rs    # Ollama/DeepL translation
│   │   └── glossary.rs       # Multi-language glossary
│   ├── db/
│   │   ├── mod.rs           # DB module exports
│   │   ├── sqlite.rs        # SQLite operations
│   │   ├── search.rs        # Tantivy search engine
│   │   └── error.rs         # Error types
│   └── web/
│       ├── mod.rs           # Axum router + SearchEngine
│       ├── handlers.rs      # All endpoints + template rendering
│       └── templates.rs     # Template utilities
├── templates/               # HTML templates (sidebar + grid layout)
│   ├── index.html          # Main portal view
│   ├── article.html         # Article reading view
│   └── search.html         # Search results view
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

## 6. Web API Endpoints

| Path | Method | Description |
|------|--------|-------------|
| `/` | GET | Article list with sidebar navigation |
| `/article/:id` | GET | Translated article view |
| `/article/:id/original` | GET | Original language article view |
| `/search?q=<query>` | GET | Search results page |
| `/domains` | GET | Domain overview |
| `/domain/:domain` | GET | Articles filtered by domain |
| `/api/articles` | GET | JSON API for articles |

---

## 7. Configuration

### matome.toml

```toml
[core]
data_dir = ".matome"

[[domains]]
url = "https://docs.example.com/"
include = ["/**"]

[translate]
provider = "ollama"           # or "deepl", "none"
model = "translategemma:4bb"
target_lang = "ja"
glossary_file = "glossary.toml"

[crawl]
concurrency = 8
respect_robots = true
timeout = 30
max_pages = 0
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

## 8. Code Quality Status

| Metric | Value | Notes |
|--------|-------|-------|
| Tests | ✅ 11/11 passing | Unit tests for core functionality |
| Build | ✅ Compiles | Minimal warnings |
| Complexity | ✅ Well-structured | Helper functions extracted |

---

## 9. Recent Changes

### 2026-04-08: Web UI Redesign

- **New Sidebar Layout**: Fixed left sidebar with navigation
  - Logo and article count
  - Quick search button with ⌘K hint
  - Overview section (Home, Search)
  - Domain section with article counts

- **Improved Search**:
  - Press ⌘K anywhere to open search modal
  - HTMX-powered live search with 300ms debounce
  - Keyboard navigation (Enter to search, Escape to close)

- **Article Reading View**:
  - Sidebar with navigation and language toggle
  - Clean typography with Crimson Pro headings
  - Responsive design for mobile

- **Domain Filtering**:
  - New `/domain/:domain` route
  - Click domains in sidebar to filter articles
  - Updated breadcrumb showing current filter

- **Design System**:
  - Warm cream palette (#faf8f5 background)
  - Orange accent (#e07a3a) + Blue accent (#3a6e8e)
  - Custom fonts: Crimson Pro (headings), IBM Plex Sans (body), JetBrains Mono (code)
  - Soft shadows and smooth animations

---

## 10. Git History

| Commit | Description |
|--------|-------------|
| Latest | feat: Redesign web UI with documentation portal layout |
| Previous | refactor: Code quality refactoring - extract helper functions |
| Previous | feat: integrate glossary and search engine into pipeline |
| Previous | Initial commit |

---

*This file is updated according to implementation progress.*
