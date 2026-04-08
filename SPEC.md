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
- **Competitive Advantage**: 「完全オフライン + Ollama翻訳 + 全文検索」の組み合わせが唯一の勝ち筋

---

## 2. Scope

### 2.1 In Scope (v1)

- [x] CLI for configuration and execution (Init, Add, Crawl, Serve, Status, Clean)
- [x] Crawling via sitemap.xml and same-domain link following
- [x] Clean Markdown extraction from HTML
- [x] Per-Markdown translation to Japanese via local/API (code block protection)
- [x] Storage and Japanese full-text search with SQLite + Tantivy
- [x] Lightweight, fast local browsing UI with Axum + HTMX
- [x] Incremental crawl support

### 2.2 Out of Scope (v1)

| 機能 | 理由 |
|------|------|
| ~~Complex JS Rendering~~ | Full SPA support - static HTML/sitemapに限定 |
| ~~Real-time Processing~~ | Daemon-style monitoring - cron委譲 |
| ~~Multi-user Features~~ | Authentication, user management |
| ~~React/Vue Usage~~ | HTMX + Tailwindでシンプル維持 |
| ~~Headless Browser~~ | Async runtime複雑化、バイナリサイズ増大 |

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

## 4. Configuration Files

### 4.1 matome.toml (Example)

```toml
[core]
data-dir = ".matome"

[[domains]]
url = "https://docs.rust-lang.org"
include = ["/**"]

[[domains]]
url = "https://developer.mozilla.org"
include = ["/**"]

[translate]
provider = "ollama"         # "deepl" | "ollama" | "none"
model = "gemma3:12b"
target-lang = "ja"
glossary-file = "glossary.toml"

[crawl]
concurrency = 8
respect-robots = true
timeout = 30
max-pages = 0               # 0 = unlimited
```

### 4.2 glossary.toml (Example)

```toml
[[terms]]
en = "compiler"
ja = "コンパイラ"

[[terms]]
en = "runtime"
ja = "ランタイム"

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
│   ├── main.rs              # Entry point (CLI routing)
│   ├── cli.rs               # Argument parsing with clap
│   ├── config.rs            # TOML loading and type definitions
│   │
│   ├── pipeline/            # Data pipeline (core)
│   │   ├── mod.rs           # Pipeline orchestration
│   │   ├── crawler.rs       # HTTP GET, Sitemap parsing
│   │   ├── extractor.rs     # HTML -> MD conversion
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
├── assets/                 # Static files
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

- **Engine**: Tantivy
- **Japanese Support**: Lindera (morphological analysis)
- **Indexed Fields**: title, description, translated_md
- **Search ID**: URL hash (note: separate from SQLite AUTOINCREMENT)

---

## 7. Web UI Architecture

### 7.1 Design Overview

**Documentation Portal Layout** with sidebar navigation:
- Fixed left sidebar (300px) with logo, search, navigation
- Article grid in main content area
- Stats bar showing article/domain counts
- Search modal with ⌘K keyboard shortcut

**Article Reading View**:
- Sidebar with navigation + language toggle
- Main content with rendered Markdown
- Clean typography with Crimson Pro headings

### 7.2 Web API Endpoints

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

### 7.3 Template System

Templates use a simple `{placeholder}` substitution system:
- `templates/index.html` - Main portal view
- `templates/article.html` - Article reading view
- `templates/search.html` - Search results view

---

## 8. CLI Commands

| Command | Description |
|---------|-------------|
| `matome init` | Generate templates for `matome.toml` and `glossary.toml` |
| `matome add <url>` | Add target domain to `matome.toml` |
| `matome crawl [--incremental]` | Execute pipeline (Crawl → Extract → Translate → Store) |
| `matome serve [--port 8080]` | Start local web server |
| `matome status` | Display DB and index statistics |
| `matome clean` | Clean database entries |

---

## 9. Design Decisions

### 9.1 Configuration Key Naming

**Decision**: **kebab-case統一** (Phase 0で修正)

- Rustのserdeは`kebab-case`を使用
- exampleファイルもkebab-caseに統一
- 新規ユーザーが混同しないよう一貫性を保つ

### 9.2 Error Handling Philosophy

**Decision**: **ログ出力 + フォールバック継続**

- 翻訳失敗時は警告ログ出力 + 原文フォールバック
- ユーザーが問題発生時に気づけるようにする
- パイプライン全体の停止を避ける

### 9.3 Trait Design

**Decision**: **Start simple**

- Trait abstraction is over-engineering for v1
- Refactor to extract when necessary
- Prioritize unified function signatures

### 9.4 Template Management

**Decision**: **External template files only** (Phase 1で修正)

- `templates/` ディレクトリ配下のファイルのみ使用
- handlers.rs内のインラインHTMLは削除
- Askama templateを使用する場合はAskamaに完全移行

### 9.5 SQLite Concurrency

**Decision**: **Connection pooling** (Phase 2で実装)

- serve時並列リクエスト対応
- r2d2/deadpool-sqlite導入
- 単一Mutexからの脱却

---

## 10. Known Issues (v1)

| Issue | Severity | Status | Fix Plan |
|-------|----------|--------|----------|
| 設定ファイルキー名不統一 | 🔴 Critical | Phase 0 | exampleファイル修正 |
| 翻訳失敗時ログなし | 🔴 Critical | Phase 0 | warn!ログ追加 |
| Extractor clone設計歪み | 🟠 High | Phase 1 | #[derive(Clone)]化 |
| handlers.rsインラインHTML | 🟠 High | Phase 1 | テンプレート一本化 |
| Mutex粒度粗すぎ | 🟠 High | Phase 2 | コネクションプール化 |
| max_pages機能未実装 | 🟡 Medium | Phase 3 | crawler.rsに上限チェック |
| incremental crawl domain判定 | 🟡 Medium | Phase 3 | サブドメイン同一看待 |
| glossary.rs 二重unwrap | 🟡 Medium | Phase 4 | const使用 |
| コードスニペット未取得 | 🟡 Medium | Phase 4 | 調査・対応 |

---

## 11. Non-Goals (v1)

| Item | Reason |
|------|------|
| ~~Real-time monitoring / auto recrawl~~ | Delegated to cron or external schedulers |
| ~~Headless browser (fantoccini, etc.)~~ | Async runtime management complexity, binary size bloat |
| ~~Complex JS rendering support~~ | Limited to static HTML and sitemap.xml |
| ~~Authentication / user management~~ | Simplicity as a local-only tool |

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
