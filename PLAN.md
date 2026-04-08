# matome Implementation Plan

**Purpose**: Rust CLI tool that collects articles from specified URLs, translates to Japanese, and provides a local web portal for browsing.

**Last Updated**: 2026-04-08  
**Phase**: Phase 0 (Emergency Fixes) - In Progress

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

## 2. Implementation Phases

### ✅ Phase 0: Emergency Fixes (IN PROGRESS)

**Priority**: 🔴 Critical - 新規ユーザーが詰む箇所

| # | Task | Status | Files | Notes |
|---|------|--------|-------|-------|
| 0-1 | 設定ファイルのキー名統一 | 🔄 In Progress | `examples/matome.toml.example` | snake_case → kebab-case |
| 0-2 | 翻訳失敗時のログ出力追加 | 🔄 In Progress | `src/pipeline/mod.rs` | warn!ログで翻訳失敗を通知 |

### ✅ Phase 1: Code Quality Foundation (COMPLETE)

| # | Task | Status | Files | Notes |
|---|------|--------|-------|-------|
| 1-1 | Extractor #[derive(Clone)] 化 | 🔄 In Progress | `src/pipeline/extractor.rs`, `src/pipeline/mod.rs` | 独自cloneメソッド削除 |
| 1-2 | テンプレート管理統一 | 🔄 In Progress | `src/web/handlers.rs`, `templates/` | インラインHTML削減 |

### ⏳ Phase 2: Scalability (PLANNED)

| # | Task | Priority | Files | Notes |
|---|------|----------|-------|-------|
| 2-1 | SQLite コネクションプール化 | 🟠 High | `src/db/sqlite.rs` | r2d2/deadpool導入 |
| 2-2 | SearchResult ID設計整理 | 🟡 Medium | `src/db/search.rs` | url-basedに一本化 |

### ⏳ Phase 3: Feature Fixes (PLANNED)

| # | Task | Priority | Files | Notes |
|---|------|----------|-------|-------|
| 3-1 | max_pages機能実装 | 🟡 Medium | `src/pipeline/crawler.rs` | 上限チェック追加 |
| 3-2 | incremental crawl改善 | 🟡 Medium | `src/pipeline/crawler.rs` | サブドメイン同一看待オプション |

### ⏳ Phase 4: Technical Debt (PLANNED)

| # | Task | Priority | Files | Notes |
|---|------|----------|-------|-------|
| 4-1 | unwrap()撲滅 | 🟡 Medium | `src/pipeline/glossary.rs` | const定数使用 |
| 4-2 | コードスニペット取得対応 | 🟡 Medium | `src/pipeline/extractor.rs` | スコープ調査 |

---

## 3. Phase 0 詳細

### 0-1. 設定ファイルキー名統一 🔴

**問題**: `matome.toml` (kebab-case) と `examples/matome.toml.example` (snake_case) の不統一

```toml
# matome.toml (現在 - kebab-case) ✅
data-dir = ".matome"
target-lang = "ja"

# examples/matome.toml.example (現在 - snake_case) ❌
data_dir = "./.matome"
target_lang = "ja"
```

**対応**: exampleファイルをkebab-caseに統一

### 0-2. 翻訳失敗時のログ出力 🔴

**問題**: `src/pipeline/mod.rs` で翻訳失敗をサイレントに握りつぶし

```rust
// 変更前
Err(_e) => {
    extracted.markdown.clone()
}

// 変更後
Err(e) => {
    warn!("Translation failed for {}: {}", url, e);
    extracted.markdown.clone()
}
```

---

## 4. Core Features

### 4.1 Data Pipeline Flow

```
Crawl → Extract → Translate → Apply Glossary → Store → Index
   ↓         ↓          ↓            ↓          ↓        ↓
 Raw    Markdown    Japanese     Terminology   SQLite   Tantivy
 HTML             Translation   Replacement
```

### 4.2 Web UI Design

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

### 4.3 CLI Commands

| Command | Description | Status |
|---------|-------------|--------|
| `matome init` | Generate config templates | ✅ |
| `matome add <url>` | Add domain to config | ✅ |
| `matome crawl [--incremental]` | Execute full pipeline | ✅ |
| `matome serve [--port <port>] [--host <host>]` | Start web server | ✅ |
| `matome status [--verbose]` | Display statistics | ✅ |
| `matome clean --all\|--domain\|--orphaned\|--id` | Clean database | ✅ |

---

## 5. Current File Structure

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
│   │   ├── translator.rs    # Ollama/DeepL API client
│   │   └── glossary.rs      # Multi-language glossary
│   ├── db/
│   │   ├── mod.rs           # DB module exports
│   │   ├── sqlite.rs        # SQLite operations
│   │   ├── search.rs        # Tantivy full-text search engine
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

## 6. Recent Changes

### 2026-04-08: Phase 0 - Emergency Fixes Started

- **Review Findings Incorporated**: Code review from REVIEW.md
- **Priority Focus**: Issues that block new users on first use
  - Config file key name consistency (snake_case vs kebab-case)
  - Translation failure logging

---

*This file is updated according to implementation progress.*
