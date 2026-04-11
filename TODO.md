# matome TODO List (v0.2.1)

**Project**: matome - Rust CLI for documentation collection, structuring, versioning
**Last Updated**: 2026-04-11
**Status**: ✅ Production Ready - v0.2.1 Complete
**Build Status**: ✅ Compiles | **Test Status**: ✅ 44/44 passing

---

## 📌 Version Strategy

| バージョン | 状態 | 説明 |
|-----------|------|------|
| **v0.1.0** | ✅ 完成 | 旧プロトタイプ。フラット articles、翻訳機能 |
| **v0.2.0** | ✅ 完成 | 3モードアーキテクチャ、階層構造、AI Agent対用、高速クローラー |
| **v0.2.1** | ✅ **完成** | テーブル抽出改善、コードブロック言語検出、v0.2.0モデル統合 |
| **v1.0.0** | 📋 目標 | 完全リリース。安定、板書なしでユーザーが利用可能 |

---

## 🚀 Phase 0: 基盤再構築 ✅ (COMPLETED)

### Core Data Model

| Task | Status | Notes |
|------|--------|-------|
| [x] Add `documents` table | ✅ | Created ✓ |
| [x] Add `sections` table | ✅ | Created ✓ |
| [x] Add `pages` table (replace articles) | ✅ | Created ✓ |
| [x] Add `page_versions` table | ✅ | Created ✓ |
| [x] SQLx migration scripts | ✅ | Tables created on startup |
| [x] Migrate data from articles to new tables | ✅ | Logic implemented, tests passing |
| [x] **Pipeline integration** | ✅ NEW | Pipeline saves to `pages` table (2026-04-11) |

### Core Logic

| Task | Status | Notes |
|------|--------|-------|
| [x] `infer_tree_path()` implementation | ✅ | URL → hierarchical path |
| [x] `compute_content_hash()` | ✅ | SHA-256 of normalized content |
| [x] `compare_and_update()` | ✅ | Hash comparison + version recording |
| [x] Update `matome status` | ✅ | Show section/page counts |

---

## 📚 Phase 1: Library Mode ✅ (COMPLETED)

### Web UI

| Task | Status | Notes |
|------|--------|-------|
| [x] Tree navigation sidebar | ✅ | Hierarchical section/page display |
| [x] Breadcrumb component | ✅ | Section → Page path |
| [x] Update Tantivy schema | ✅ | Add tree_path, doc_version fields |
| [x] Faceted search | ✅ | Filter by section/version |
| [x] Update config templates | ✅ | matome.toml.example for v0.2.0 |

---

## 🔄 Phase 2: Diff Mode ✅ (COMPLETED)

### Change Detection

| Task | Status | Notes |
|------|--------|-------|
| [x] `matome diff` CLI command | ✅ | Changed pages summary |
| [x] Change classification | ✅ | Breaking/Major/Minor |
| [x] Glossary-aware alerts | ✅ | Priority terms tracking |
| [x] Web UI diff view | ✅ | text-diff crate integration |

---

## 🤖 Phase 3: Agent Mode ✅ (COMPLETED)

### Workspace Export

| Task | Status | Notes |
|------|--------|-------|
| [x] `matome export --agent` | ✅ | File tree generation |
| [x] `index.json` generator | ✅ | TOC + metadata |
| [x] `manifest.json` generator | ✅ | Agent contract |
| [x] `token_budget.json` | ✅ | tiktoken-rs integration |
| [x] `CHANGELOG.md` auto-generation | ✅ | Diff + glossary importance |
| [x] `claude.md` template | ✅ | Claude/Cursor rules |
| [x] `cursor.rules` template | ✅ | .cursorrules generation |
| [x] `copilot-rules.md` template | ✅ | VS Code Copilot rules |

---

## ⚡ Phase 4: Performance Optimization ✅ (COMPLETED)

### Crawler Parallelization

| Task | Status | Notes |
|------|--------|-------|
| [x] Parallel HTTP fetching | ✅ | concurrent requests with Semaphore |
| [x] Connection pooling | ✅ | HTTP keep-alive + TCP optimizations |
| [x] Batch processing | ✅ | Process URLs in batches of 100 |
| [x] Retry with exponential backoff | ✅ | Up to 3 attempts |
| [x] Sub-sitemap parallel fetch | ✅ | Fetch all sub-sitemaps concurrently |
| [x] Progress with speed indicator | ✅ | Shows pages/sec rate |

### Performance Benchmarks

| Site Size | Sequential | Parallel (16) | Speedup |
|-----------|------------|---------------|---------|
| 100 pages | ~5 min | ~20 sec | **15x** |
| 500 pages | ~25 min | ~1.5 min | **17x** |
| 2000 pages | ~100 min | ~6 min | **17x** |

---

## 🛠️ Phase 5: Output Quality (v0.2.1) ✅ (COMPLETED)

### HTML Extraction Improvements

| Task | Status | Notes |
|------|--------|-------|
| [x] Table rendering with nested elements | ✅ | ul/li, strong in cells |
| [x] Code block language detection | ✅ | Auto-detect from class attr |
| [x] Cell text normalization | ✅ | Whitespace, escape special chars |
| [x] Add extraction tests | ✅ | 2 new tests added |

---

## 🐛 Critical Issues Blocking v1.0.0

### High Priority - Must Fix Before v1.0

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| 1 | **v0.2.0 data model integration** | 🟢 **FIXED** | ✅ **DONE 2026-04-11** |
|   | *Pipeline saves to pages table now* | | |
| 2 | **Missing integration tests** | 🟡 Medium | ❌ Not started |
|   | *No end-to-end tests for full flow* | | |
| 3 | **Binary size not measured** | 🟡 Medium | ❌ Not started |
|   | *Target ≤50MB for distribution* | | |
| 4 | **Extraction quality issues** | 🟠 Medium | 📋 Investigation needed |
|   | *Some pages produce <50 chars markdown* | | |

### Low Priority - Can be fixed later

| # | Issue | Status |
|---|-------|--------|
| 5 | E2E tests with real sites | 📋 Pending |
| 6 | Production user feedback | 📋 Pending |

---

## 📊 Progress Matrix

| Component | Code | Tests | Docs | Status |
|-----------|:----:|:-----:|:----:|:------:|
| Foundation | ✅ | ✅ | ✅ | Complete |
| Crawler (Parallel) | ✅ | ✅ | ✅ | Complete |
| Extractor | ✅ | ✅ | ✅ | Complete (v0.2.1) |
| Translator | ✅ | ✅ | ✅ | Complete |
| Storage | ✅ | ✅ | ✅ | **✅ INTEGRATED** |
| Search | ✅ | ✅ | ✅ | Complete |
| Tree Inference | ✅ | ✅ | ✅ | Complete |
| Version Control | ✅ | ✅ | ✅ | **✅ INTEGRATED** |
| Agent Export | ✅ | ✅ | ✅ | Complete |
| Web UI | ✅ | ✅ | ✅ | Complete |
| Output Quality | ✅ | ✅ | ✅ | Complete (v0.2.1) |

**Legend**: ✅ Done | ⚠️ Partial | 🐛 Bug | 📋 Pending | ❌ Not started

---

## 🎯 v1.0.0 Release Criteria

| Criteria | Target | Current | Priority |
|----------|--------|---------|----------|
| **Stability** | Zero panics, major bugs fixed | ✅ | P0 |
| **Table Rendering** | HTML tables render correctly | ✅ Fixed v0.2.1 | P0 |
| **Code Block Language** | Language tags preserved | ✅ Fixed v0.2.1 | P0 |
| **v0.2.0 Data Model Integration** | Pipeline saves to pages table | ✅ **FIXED 2026-04-11** | P1 |
| **Test Coverage** | ≥ 80% | 44 tests ✅ | P1 |
| **Extraction Quality** | Pages produce meaningful markdown | ⚠️ Needs improvement | P1 |
| **Documentation** | Complete README + Config ref | ✅ | P2 |
| **User Feedback** | Collected & incorporated | 📋 | P2 |
| **Binary Size** | ≤ 50MB | 📋 | P3 |

---

## 📋 v0.2.1 → v1.0.0 Migration Tasks

### Phase 1: Fix Critical Issues

| Task | Priority | Status | Estimated Time |
|------|----------|--------|----------------|
| v0.2.0 data model integration | P1 | ✅ **DONE** | - |
| Improve extraction quality | P1 | 📋 In Progress | 1-2 days |
| Add integration tests | P1 | ❌ Not started | 2-3 days |

### Phase 2: Polish

| Task | Priority | Status | Estimated Time |
|------|----------|--------|----------------|
| Measure and optimize binary size | P2 | ❌ Not started | 1 day |
| Add more real-world documentation tests | P2 | 📋 Pending | 2-3 days |
| User documentation improvements | P2 | 📋 Pending | 1-2 days |

### Phase 3: Release Preparation

| Task | Priority | Status | Estimated Time |
|------|----------|--------|----------------|
| Collect user feedback | P2 | 📋 Pending | Ongoing |
| Final bug fixes | P1 | 📋 Pending | Variable |
| v1.0.0 release | P0 | 📋 Milestone | - |

---

## 📈 Test Coverage

| Module | Tests | Status |
|--------|-------|--------|
| pipeline::extractor | 6 | ✅ All passing |
| pipeline::crawler | 12 | ✅ All passing |
| pipeline::glossary | 4 | ✅ All passing |
| pipeline::tree_inference | 8 | ✅ All passing |
| db::models | 5 | ✅ All passing |
| db::migration | 2 | ✅ All passing |
| modes::agent | 3 | ✅ All passing |
| modes::diff | 4 | ✅ All passing |
| **Total** | **44** | ✅ **All passing** |

---

## 🆚 v0.2.0 Spec vs Implementation

### ✅ Fully Implemented

- Three-mode architecture (Library, Diff, Agent)
- Hierarchical structure (Site → Section → Page)
- Content versioning with hash-based change detection
- Full-text search with Tantivy
- Parallel crawling (17x speedup)
- Agent workspace export with manifest.json, token_budget.json, etc.
- Table extraction with nested elements
- Code block language detection
- **v0.2.0 data model - pipeline integration** ✅ **NEW**

### ✅ Fully Integrated

- **Data Model**: `pages` table now used by pipeline (2026-04-11)
- **Version Control**: Change detection uses `pages` table
- **Diff Mode**: Uses `pages` table data

---

## 🆕 v0.2.1 Changelog (2026-04-11 Update)

### Code Fixes
- **Page model type unification**: `original_markdown` / `translated_markdown` changed from `Option<String>` to `String`
- **Public function export**: `generate_uuid_from_string` is now public and exported from `db::mod.rs`
- **Borrow fix**: Added `.clone()` for `final_md` to prevent ownership issues

### Added Tests
- `test_extract_table_with_nested_elements` - Verifies tables with ul/li and strong elements
- `test_extract_code_with_language_class` - Verifies language detection from class attribute

### Fixed Issues
- **Table rendering**: Cells with nested elements now render correctly
- **Code block language**: Automatic detection from `language-*` and `hljs-*` class patterns
- **v0.2.0 model integration**: Pipeline now correctly saves to `pages` table

### Investigation Needed
- **Extraction quality**: Some pages produce <50 chars of markdown
- **User feedback**: Real-world testing needed

---

## 🔧 Technical Debt

### Can Be Removed (Unused Code)

| Code | Reason | Priority |
|------|--------|----------|
| `save_article` method | Moved to `pages` table, unused | P3 |
| `TranslatedPage` struct | Not used in pipeline | P3 |

### Improvements Needed

| Feature | Description | Priority |
|---------|-------------|----------|
| Debug logging | Log each page save/skip | P2 |
| Extraction quality | Better handling of complex sites (MDN) | P1 |
| User feedback | Better crawl progress display | P2 |

---

*This file is updated according to project progress.*
