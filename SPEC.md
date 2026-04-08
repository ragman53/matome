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
| **First-Use Friendly** | 新規ユーザーが最初のコマンドで詰まない設計 |

### Target Users

- **Primary**: 英語ドキュメントを日本語で読みたい日本語圏エンジニア
- **Use Case**: 完全オフライン環境、接続不安定な環境、Ollama翻訳 желающих
- **Competitive Advantage**: 「完全オフライン + Ollama翻訳 + 全文検索」の組み合わせ

---

## 2. Scope

### 2.1 In Scope (v1)

- [x] CLI for configuration and execution (Init, Add, Crawl, Serve, Status, Clean)
- [x] Crawling via sitemap.xml and same-domain link following
- [x] Clean Markdown extraction from HTML (Docusaurus/MkDocs対応)
- [x] Per-Markdown translation to Japanese via local/API (code block protection)
- [x] Storage and Japanese full-text search with SQLite + Tantivy
- [x] Lightweight, fast local browsing UI with Axum + HTMX
- [x] Incremental crawl support with subdomain normalization

### 2.2 Out of Scope (v1)

| 機能 | 理由 |
|------|------|
| ~~Complex JS Rendering~~ | Static HTML/sitemapに限定 |
| ~~Real-time Processing~~ | cron委譲 |
| ~~Multi-user Features~~ | ローカルツールのシンプルさ維持 |
| ~~Headless Browser~~ | バイナリサイズ・複雑性増加防止 |

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

---

## 4. Configuration Files

### 4.1 matome.toml

```toml
[core]
data-dir = ".matome"

[[domain]]
url = "https://docs.rust-lang.org"
include = ["/**"]

[translate]
provider = "ollama"
model = "gemma3:12b"
target-lang = "ja"
glossary-file = "glossary.toml"

[crawl]
concurrency = 8
respect-robots = true
timeout = 30
max-pages = 0
# treat-subdomains-same = true  # Optional: docs.example.com = example.com
```

### 4.2 glossary.toml

```toml
[[terms]]
en = "compiler"
ja = "コンパイラ"

[[terms]]
en = "API"
translations = { ja = "API", zh = "API", ko = "API" }
```

---

## 5. Directory Structure

```
matome/
├── Cargo.toml
├── matome.toml              # 設定ファイル
├── glossary.example.toml     # 用語集テンプレート
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # Argument parsing
│   ├── config.rs            # TOML parsing
│   ├── pipeline/
│   │   ├── mod.rs           # Pipeline orchestration
│   │   ├── crawler.rs       # HTTP fetch, sitemap parsing
│   │   ├── extractor.rs     # HTML → Markdown (Docusaurus/MkDocs対応)
│   │   ├── translator.rs    # MD translation
│   │   └── glossary.rs      # Term replacement
│   ├── db/
│   │   ├── mod.rs
│   │   ├── sqlite.rs         # SQLite (WAL mode)
│   │   └── search.rs          # Tantivy (full-text search)
│   └── web/
│       ├── mod.rs           # Axum router
│       └── handlers.rs      # Handlers (301 lines, embedded templates)
├── templates/               # HTML templates
└── examples/
    └── matome.toml.example  # 設定テンプレート
```

---

## 6. Database Design

### 6.1 articles Table

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

### 6.2 Full-Text Search Index

- **Engine**: Tantivy (WAL mode enabled)
- **Indexed Fields**: title, description, translated_md
- **Search ID**: URL hash (documented distinction from SQLite AUTOINCREMENT)

---

## 7. Web UI Architecture

### 7.1 Design Overview

**Documentation Portal Layout**:
- Fixed left sidebar (300px) with logo, search, navigation
- Article grid in main content area
- Stats bar showing article/domain counts
- Search modal with ⌘K keyboard shortcut

### 7.2 Web API Endpoints

| Path | Method | Description |
|------|--------|-------------|
| `/` | GET | Article list with sidebar navigation |
| `/article/:id` | GET | Translated article view |
| `/article/:id/original` | GET | Original language article view |
| `/search` | GET | Full-text search results |
| `/search` | POST | HTMX live search |
| `/domains` | GET | Domain overview page |
| `/domain/:domain` | GET | Articles filtered by domain |
| `/api/articles` | GET | JSON API |

### 7.3 Template System

Templates are embedded at compile time using `include_str!()`:
- `templates/index.html` - Main portal view
- `templates/article.html` - Article reading view
- `templates/search.html` - Search results view

---

## 8. CLI Commands

| Command | Description |
|---------|-------------|
| `matome init` | Generate templates |
| `matome add <url>` | Add domain to config |
| `matome crawl [--incremental]` | Execute pipeline |
| `matome serve [--port 8080]` | Start web server |
| `matome status` | Display statistics |
| `matome clean` | Clean database |

---

## 9. Design Decisions

### 9.1 Configuration Key Naming

**kebab-case** (e.g., `data-dir`, `target-lang`)

### 9.2 Error Handling

- Translation failures: warn log + fallback to original
- Search failures: fallback to LIKE query
- No silent error swallowing

### 9.3 Template Management

Compile-time embedding via `include_str!()` - no runtime file loading

### 9.4 SQLite Concurrency

WAL mode enabled for concurrent reads during write operations

---

## 10. Non-Goals (v1)

| Item | Reason |
|------|------|
| ~~Real-time monitoring / auto recrawl~~ | cron委譲 |
| ~~Headless browser~~ | 複雑性・サイズ増加防止 |
| ~~Complex JS rendering support~~ | Static HTML/sitemap限定 |
| ~~Authentication / user management~~ | ローカルツール |

---

*This document is updated according to project progress.*
