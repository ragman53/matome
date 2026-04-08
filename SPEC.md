# matome - Specification

> A Rust CLI tool that collects articles from specified document domains, applies automatic Japanese translation, and builds a local integrated web portal.

---

## 1. Project Overview

### Core Concept

"Automatic English document collection, translation, and local integration" in a single binary.

### Design Principles

| Principle | Description |
|-----------|-------------|
| **Strict Separation of Concerns** | Crawl, Extract, Translate, Store, Serve are independently testable |
| **Unidirectional Data Flow** | `URL → HTML → Markdown → Translated MD → DB` |
| **Stateless Pipeline Design** | Side effects are consolidated to DB writes only |

---

## 2. Scope

### 2.1 In Scope (v1)

- [x] CLI for configuration and execution (Init, Add, Crawl, Serve)
- [x] Crawling via sitemap.xml and same-domain link following
- [x] Clean Markdown extraction from HTML
- [x] Per-Markdown translation to Japanese via local/API (code block protection)
- [x] Storage and Japanese full-text search with SQLite + Tantivy
- [x] Lightweight, fast local browsing UI with Axum + HTMX

### 2.2 Out of Scope (v1)

- **Complex JS Rendering**: Full SPA support is deferred, prioritizing static HTML and sitemap (reduces dependency on browser automation, prevents bloat)
- **Real-time Processing**: Daemon-style constant monitoring and auto-update features (delegated to cron or external schedulers)
- **Multi-user Features**: Authentication, user management, sharing features
- **Excessive Frontend**: React/Vue usage (limited to HTMX + Tailwind to keep builds simple)

---

## 3. System Architecture

### 3.1 Data Flow

```
[ Sources ]
      │
      ▼
┌─────────────┐
│ 1. Crawler │ ────► [ Raw HTML ]
└─────────────┘
      │
      ▼
┌─────────────┐
│ 2. Extract  │ ────► [ Markdown ]
└─────────────┘
      │
      ▼
┌─────────────┐
│ 3. Translate│ ──► (API / Local LLM)
└─────────────┘
      │
      ▼
┌─────────────┐
│ 4. Storage  │ ────► [ Translated Markdown ]
└─────────────┘
      │
      ▼
┌─────────────────────┐
│ SQLite (Data)       │ ◄───► [ Tantivy (Index) ]
└─────────────────────┘
      │
      ▼
┌─────────────┐
│ 5. Server   │ ────► [ Web UI (Axum/HTMX) ]
└─────────────┘
```

### 3.2 Layer Separation

| Layer | Responsibility | Technology |
|-------|---------------|------------|
| **Pipeline Layer** | Data acquisition, transformation, storage | Crawler, Extractor, Translator, Storage |
| **Presentation Layer** | Web UI delivery | Axum + HTMX |

---

## 4. CLI Commands

| Command | Description |
|---------|-------------|
| `matome init` | Generate templates for `matome.toml` and `glossary.toml` |
| `matome add <url>` | Add target domain to `matome.toml` |
| `matome crawl [--incremental]` | Execute pipeline (Crawl → Extract → Translate → Store) |
| `matome serve [--port 8080]` | Start local web server |
| `matome status` | Display DB and index statistics |

---

## 5. Configuration Files

### 5.1 matome.toml (Example)

```toml
[core]
data_dir = "./.matome"      # DB and index storage location

[[domains]]
url = "https://docs.rust-lang.org"
include = ["/**"]

[[domains]]
url = "https://developer.mozilla.org"
include = ["/**"]

[translate]
provider = "ollama"         # "deepl" | "ollama" | "none"
model = "gemma3:12b"
target_lang = "ja"
glossary_file = "glossary.toml"

[crawl]
concurrency = 8
respect_robots = true
```

### 5.2 glossary.toml (Example)

```toml
[[terms]]
en = "compiler"
ja = "コンパイラ"

[[terms]]
en = "runtime"
ja = "ランタイム"
```

---

## 6. Directory Structure

```
matome/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point (CLI routing)
│   ├── cli.rs               # Argument parsing with clap
│   ├── config.rs            # TOML loading and type definitions
│   │
│   ├── pipeline/            # Data pipeline (core)
│   │   ├── mod.rs           # Pipeline orchestration
│   │   ├── crawler.rs       # HTTP GET, Sitemap parsing (reqwest)
│   │   ├── extractor.rs     # HTML -> MD conversion (scraper, readability-rs)
│   │   └── translator.rs    # MD translation, caching, term replacement
│   │
│   ├── db/                  # Data persistence layer
│   │   ├── mod.rs
│   │   ├── sqlite.rs         # SQLite (metadata, MD text)
│   │   └── search.rs          # Tantivy (full-text search)
│   │
│   └── web/               # Presentation layer
│       ├── mod.rs           # Axum router
│       ├── handlers.rs      # Endpoints and template rendering
│       └── templates.rs     # Template utilities
│
├── templates/              # HTML templates (sidebar + grid layout)
│   ├── index.html          # Main portal view
│   ├── article.html        # Article reading view
│   └── search.html         # Search results view
├── assets/                 # Static files (if any)
├── glossary.toml            # Terminology glossary
└── matome.toml             # Configuration file
```

---

## 7. Database Design

### 7.1 articles Table

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

### 7.2 Full-Text Search Index

- **Engine**: Tantivy
- **Japanese Support**: Lindera (morphological analysis)
- **Indexed Fields**: title, description, translated_md

---

## 8. Web UI Architecture

### 8.1 Design Overview

**Documentation Portal Layout** with sidebar navigation:
- Fixed left sidebar (300px) with logo, search, navigation
- Article grid in main content area
- Stats bar showing article/domain counts
- Search modal with ⌘K keyboard shortcut

**Article Reading View**:
- Sidebar with navigation + language toggle
- Main content with rendered Markdown
- Clean typography with Crimson Pro headings

### 8.2 Web API Endpoints

| Path | Method | Description |
|------|--------|-------------|
| `/` | GET | Article list with sidebar navigation |
| `/article/:id` | GET | Translated article view |
| `/article/:id/original` | GET | Original language article view |
| `/search` | GET | Full-text search results |
| `/search` | POST | HTMX search (returns cards only) |
| `/domains` | GET | Domain overview page |
| `/domain/:domain` | GET | Articles filtered by domain |
| `/api/articles` | GET | JSON API |

### 8.3 Template System

Templates use a simple `{placeholder}` substitution system:
- `templates/index.html` - Main portal view
- `templates/article.html` - Article reading view
- `templates/search.html` - Search results view

Fallback inline templates are included in `handlers.rs` for reliability.

---

## 9. Milestones

### M0: Foundation

```
Input: None
Output: CLI Help display
```

- CLI foundation with clap and serde
- matome.toml parser implementation

### M1: Crawler Core

```
Input: URL
Output: Raw HTML string
```

- Async fetching with reqwest
- robots.txt/sitemap.xml parsing

### M2: Extraction

```
Input: Raw HTML
Output: Clean Markdown string
```

- Content extraction with readability-rs and scraper
- HTML to Markdown conversion

### M3: Translation

```
Input: Markdown string
Output: Translated Markdown string
```

- Ollama/DeepL API client implementation
- Fixed term replacement based on glossary.toml
- Code block protection

### M4: Storage & Search

```
Input: Translated MD and metadata
Output: SQLite storage and Tantivy indexing
```

- Metadata storage in SQLite
- Full-text search index building with Tantivy

### M5: Web Server & UI

```
Input: DB queries
Output: HTML/HTMX responses
```

- Data reading from SQLite
- HTML rendering with comrak
- UI construction with Axum + Askama + HTMX

### M6: Polish

- Incremental Crawl implementation
- Robust error handling

---

## 10. Non-Goals (v1)

| Item | Reason |
|------|--------|
| ~~Real-time monitoring / auto recrawl~~ | Delegated to cron or external schedulers |
| ~~Headless browser (fantoccini, etc.)~~ | Async runtime management becomes complex, binary size bloat |
| ~~Complex JS rendering support~~ | Limited to static HTML and sitemap.xml |
| ~~Authentication / user management~~ | Simplicity as a local-only tool |

---

## 11. Design Decisions

### 11.1 Trait Design

> **Q**: Should pipeline components be abstracted with traits?

**Decision**: **Start simple**

- Trait abstraction is over-engineering for v1
- Refactor to extract when necessary
- Prioritize unified function signatures; consider traits from M3 (Translation) onward

### 11.2 Headless Browser

> **Q**: Should we introduce browser automation like fantoccini?

**Decision**: **Do not introduce**

- Async runtime and browser process management drastically complicates code
- Binary size bloat
- Focus on HTTP direct + HTML parsing only

### 11.3 Scheduling

> **Q**: Should we embed auto-update functionality?

**Decision**: **Do not embed (external delegation)**

- Unix philosophy as a CLI tool (do one thing well)
- Delegate to cron or external schedulers

---

## 12. Glossary

| Term | Definition |
|------|------------|
| **Crawl** | The process of fetching HTML from specified URLs |
| **Extract** | The process of extracting main content and converting to Markdown |
| **Translate** | The process of translating Markdown to Japanese |
| **Store** | The process of saving data to SQLite and Tantivy |
| **Serve** | The process of providing Web UI |
| **Incremental Crawl** | Crawling only the differences since the last run |
| **glossary** | Definition file for translation mappings of technical terms |

---

*This document is updated according to project progress.*
