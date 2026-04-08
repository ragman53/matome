# matome Implementation Plan

**Purpose**: Rust CLI tool that collects articles from specified URLs, translates to Japanese, and provides a local web portal for browsing.

**Last Updated**: 2026-04-08  
**Status**: ✅ ALL PHASES COMPLETED

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

## 2. Implementation Phases (COMPLETED ✅)

### ✅ Phase 0: Emergency Fixes

| # | Task | Status |
|---|------|--------|
| 0-1 | 設定ファイルのキー名統一 (kebab-case) | ✅ Complete |
| 0-2 | 翻訳失敗時のログ出力追加 | ✅ Complete |

### ✅ Phase 1: Code Quality Foundation

| # | Task | Status |
|---|------|--------|
| 1-1 | Extractor #[derive(Clone)] 化 | ✅ Complete |
| 1-2 | テンプレート管理統一 (include_str!) | ✅ Complete (58% 削減) |

### ✅ Phase 2: Scalability

| # | Task | Status |
|---|------|--------|
| 2-1 | SQLite WALモード有効化 | ✅ Complete |
| 2-2 | SearchResult ID設計整理 | ✅ Complete |

### ✅ Phase 3: Feature Fixes

| # | Task | Status |
|---|------|--------|
| 3-1 | max_pages機能実装 | ✅ Complete |
| 3-2 | incremental crawl改善 (treat_subdomains_same) | ✅ Complete |

### ✅ Phase 4: Technical Debt

| # | Task | Status |
|---|------|--------|
| 4-1 | glossary.rs unwrap()撲滅 | ✅ Complete |
| 4-2 | コードスニペット取得対応 | ✅ Complete |

---

## 3. Code Quality Achievements

```
handlers.rs:     711 lines → 301 lines (58% 削減)
テスト数:        11 → 15 tests passed
unwrap()問題:    全て撲滅
WALモード:       有効化（読み取り並列化）
コード抽出:      Docusaurus/MkDocs対応
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
- Fixed left sidebar (300px) with logo, search, navigation
- Article grid in main content area
- Stats bar showing article/domain counts
- Search modal with ⌘K keyboard shortcut

### 4.3 CLI Commands

| Command | Description | Status |
|---------|-------------|--------|
| `matome init` | Generate config templates | ✅ |
| `matome add <url>` | Add domain to config | ✅ |
| `matome crawl [--incremental]` | Execute full pipeline | ✅ |
| `matome serve [--port <port>]` | Start web server | ✅ |
| `matome status [--verbose]` | Display statistics | ✅ |
| `matome clean` | Clean database | ✅ |

---

## 5. Current File Structure

```
matome/
├── Cargo.toml
├── matome.toml              # Configuration file
├── glossary.example.toml    # Terminology glossary template
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
│   │   ├── sqlite.rs        # SQLite operations (WAL mode)
│   │   ├── search.rs        # Tantivy full-text search engine
│   │   └── error.rs        # Error types
│   └── web/
│       ├── mod.rs           # Axum router + SearchEngine
│       └── handlers.rs      # All endpoints (301 lines, embedded templates)
├── templates/               # HTML templates (compiled into binary)
│   ├── index.html          # Main portal view
│   ├── article.html        # Article reading view
│   └── search.html         # Search results view
└── examples/
    └── matome.toml.example # Configuration template (kebab-case)
```

---

## 6. Git History (Recent Commits)

| Commit | Description |
|--------|-------------|
| d8fc733 | fix: Remove double unwrap in glossary.rs |
| 06f3b3a | docs: Update TODO.md - code snippet extraction fixed |
| 2626518 | fix: Improve code snippet extraction for Docusaurus/MkDocs sites |
| 38a7c72 | docs: Update TODO.md - Phase 3 completed |
| cfb21cb | feat: Phase 3 feature fixes |
| 030601f | docs: Update TODO.md - Phase 2 completed |
| c0bd61d | perf: Phase 2 scalability improvements |
| 7f65efd | docs: Update TODO.md with completed Phase 0 and Phase 1 tasks |
| 12f931c | refactor: Phase 1 code quality improvements |
| 7cb604e | docs: Phase 0 documentation updates |

---

*This file is updated according to implementation progress.*
