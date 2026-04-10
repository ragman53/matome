# matome TODO List (v0.2.0)

**Project**: matome - Rust CLI for documentation collection, structuring, versioning  
**Last Updated**: 2026-04-10  
**Status**: ✅ Production Ready - v0.2.0 Complete  
**Build Status**: ✅ Compiles | **Test Status**: ✅ 42/42 passing  

---

## 📌 Version Strategy

| バージョン | 状態 | 説明 |
|-----------|------|------|
| **v0.1.0** | ✅ 完成 | 旧プロトタイプ。フラット articles、翻訳機能 |
| **v0.2.0** | ✅ 完成 | 3モードアーキテクチャ、階層構造、Agent対応、高速クローラー |
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

## 🐛 Known Issues (2026-04-10)

### High Priority

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| 1 | v0.2.0 data model not integrated | 🟡 Medium | ⚠️ Pipeline saves to articles table only, pages table unused |
| 2 | Table rendering issue | 🟡 Medium | 🐛 Tables render as plain text instead of HTML tables |

### Low Priority

| # | Issue | Status |
|---|-------|--------|
| 3 | Binary size not measured | 📋 Pending |
| 4 | E2E tests with real sites | 📋 Pending |

---

## 📊 Progress Matrix

| Component | Code | Tests | Docs | Status |
|-----------|:----:|:-----:|:----:|:------:|
| Foundation | ✅ | ✅ | ✅ | Complete |
| Crawler (Parallel) | ✅ | ✅ | ✅ | Complete |
| Extractor | ✅ | ✅ | ✅ | Complete |
| Translator | ✅ | ✅ | ✅ | Complete |
| Storage | ✅ | ✅ | ✅ | Complete |
| Search | ✅ | ✅ | ✅ | Complete |
| Tree Inference | ✅ | ✅ | ✅ | Complete |
| Version Control | ✅ | ✅ | ✅ | Complete |
| Agent Export | ✅ | ✅ | ✅ | Complete |
| Web UI | ✅ | ✅ | ✅ | Complete |

**Legend**: ✅ Done | ⚠️ Partial | 🐛 Bug | 📋 Pending

---

## 🎯 v1.0.0 Release Criteria

| Criteria | Target | Current | Priority |
|----------|--------|---------|----------|
| Stability | Zero panics, major bugs fixed | ✅ | P0 |
| v0.2.0 Data Model Integration | Pipeline saves to pages table | ⚠️ In Progress | P1 |
| Table Rendering Fix | HTML tables render correctly | 🐛 Broken | P1 |
| Test Coverage | ≥ 80% | 42 tests ✅ | P1 |
| Documentation | Complete README + Config ref | ✅ | P2 |
| User Feedback | Collected & incorporated | 📋 | P2 |
| Binary Size | ≤ 50MB | 📋 | P3 |

---

## 📋 v0.2.0 → v1.0.0 Migration Tasks

### Phase 1: Fix Critical Issues

| Task | Priority | Estimated Time |
|------|----------|----------------|
| Integrate v0.2.0 data model into pipeline | P1 | 1-2 days |
| Fix table rendering in extractor | P1 | 1 day |
| Add integration tests | P1 | 2-3 days |

### Phase 2: Polish

| Task | Priority | Estimated Time |
|------|----------|----------------|
| Measure and optimize binary size | P2 | 1 day |
| Add more real-world documentation tests | P2 | 2-3 days |
| User documentation improvements | P2 | 1-2 days |

### Phase 3: Release Preparation

| Task | Priority | Estimated Time |
|------|----------|----------------|
| Collect user feedback | P2 | Ongoing |
| Final bug fixes | P1 | Variable |
| v1.0.0 release | P0 | Milestone |

---

*This file is updated according to project progress.*
