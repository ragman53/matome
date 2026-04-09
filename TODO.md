# matome TODO List (v0.2.0)

**Project**: matome - Rust CLI for documentation collection, structuring, versioning  
**Last Updated**: 2026-04-09  
**Status**: 🔄 Phase 0: 基盤再構築 ✅  
**Build Status**: ✅ Compiles | **Test Status**: ✅ 15/15 passing

---

## 📌 Version Strategy

| バージョン | 状態 | 説明 |
|-----------|------|------|
| **v0.1.0** | ✅ 完成 | 現行プロトタイプ。フラット articles、翻訳機能 |
| **v0.2.0** | 🔄 開発中 | このTODOの方向性。3モードアーキテクチャ、階層構造 |
| **v1.0.0** | 📋 目標 | 完全リリース。安定、板書なしでユーザーが利用可能 |

---

## 🚀 Phase 0: 基盤再構築 ✅ (v0.2.0 - Priority 0)

### Core Data Model Migration

| Task | Status | Notes |
|------|--------|-------|
| [x] Add `documents` table | ⬜ | UUID-based, base_url unique |
| [x] Add `sections` table | ⬜ | document_id FK, path_prefix |
| [x] Add `pages` table (replace articles) | ⬜ | tree_path, content_hash, breadcrumbs |
| [x] Add `page_versions` table | ⬜ | Change history tracking |
| [x] SQLx migration scripts | ⬜ | v0.1.0→v0.2.0 migration with fallback |
| [x] Fallback: `tree_path = "/page/{id}"` | ⬜ | For existing flat data |

### Core Logic

| Task | Status | Notes |
|------|--------|-------|
| [x] `infer_tree_path()` implementation | ⬜ | URL → hierarchical path |
| [x] `compute_content_hash()` | ⬜ | SHA-256 of normalized content |
| [x] `compare_and_update()` | ⬜ | Hash comparison + version recording |
| [x] Update `matome status` | ⬜ | Show section/page counts |

---

## 📚 Phase 1: Library Mode (Priority 1)

### Web UI Enhancements

| Task | Status | Notes |
|------|--------|-------|
| [x] Tree navigation sidebar | ⬜ | Hierarchical section/page display |
| [x] Breadcrumb component | ⬜ | Section → Page path |
| [x] Update Tantivy schema | ⬜ | Add tree_path, doc_version fields |
| [x] Faceted search | ⬜ | Filter by section/version |
| [x] Update config templates | ⬜ | matome.toml.example for v0.2.0 |

---

## 🔄 Phase 2: Diff Mode (Priority 2)

### Change Detection

| Task | Status | Notes |
|------|--------|-------|
| [x] `matome diff` CLI command | ⬜ | Changed pages summary |
| [x] Change classification | ⬜ | Breaking/Major/Minor |
| [x] Glossary-aware alerts | ⬜ | Priority terms tracking |
| [x] Web UI diff view | ⬜ | text-diff crate integration |
| [ ] Periodic crawl + webhook | ⬜ | Optional monitoring |

---

## 🤖 Phase 3: Agent Mode (Priority 3)

### Workspace Export

| Task | Status | Notes |
|------|--------|-------|
| [x] `matome export --agent` | ⬜ | File tree generation |
| [x] `index.json` generator | ⬜ | TOC + metadata |
| [x] `manifest.json` generator | ⬜ | Agent contract |
| [x] `workspace.yaml` generator | ⬜ | Configuration for agents |
| [ ] `token_budget.json` | ⬜ | tiktoken-rs integration |
| [x] `CHANGELOG.md` auto-generation | ⬜ | Diff + glossary importance |
| [x] `claude.md` template | ⬜ | Claude/Cursor rules |
| [ ] `cursor.rules` template | ⬜ | VS Code Copilot rules |
| [x] `matome bundle` command | ⬜ | Context bundle generation |

### Verification

| Task | Status | Notes |
|------|--------|-------|
| [ ] Sample workspace: tokio | ⬜ | Test export |
| [ ] Sample workspace: kubernetes | ⬜ | Test export |
| [ ] Claude Code integration test | ⬜ | End-to-end verification |

---

## 🔧 Technical Debt

| Task | Status | Notes |
|------|--------|-------|
| ~~unwrap() elimination~~ | ✅ | Completed in v0.1.0 |
| ~~WAL mode~~ | ✅ | Enabled |
| ~~Code extraction (Docusaurus/MkDocs)~~ | ✅ | Completed in v0.1.0 |

---

## 📊 Progress Matrix

| Component | Code | Tests | Docs | Status |
|-----------|:----:|:-----:|:----:|:------:|
| Foundation | ✅ | ⬜ | ⬜ | Phase 0 |
| Crawler | ✅ | ✅ | ✅ | Complete |
| Extractor | ✅ | ✅ | ✅ | Complete |
| Translator | ✅ | ✅ | ✅ | Complete |
| Storage | ⬜ | ⬜ | ✅ | In Progress |
| Search | ⬜ | ⬜ | ✅ | In Progress |
| Tree Inference | ✅ | ⬜ | ⬜ | Not Started |
| Version Control | ✅ | ⬜ | ⬜ | Not Started |
| Agent Export | ⬜ | ⬜ | ⬜ | Not Started |
| Web UI (v0.2.0) | ⬜ | ⬜ | ⬜ | Not Started |

**Legend**: ✅ Done | ⬜ Not Started | 🔄 In Progress

---

## 📅 Development Timeline

| Period | Phase | Target |
|--------|-------|--------|
| Week 1-2 | Phase 0 | Data model migration + tree_path inference |
| Week 3-5 | Phase 1 | Library mode with tree navigation |
| Week 6-9 | Phase 2 | Diff mode implementation |
| Week 10-14 | Phase 3 | Agent mode workspace export |
| Week 15-20 | Phase 4 | Integration + ecosystem |

---

## 🎯 v0.2.0 Goals (Direction Finding Phase)

### Primary Objectives

1. **Structure over Flat Storage**
   - Hierarchical `document → section → page` model
   - Tree path inference from URL patterns
   - Breadcrumb navigation

2. **Version-Aware**
   - Content hash-based change detection
   - Change history with `page_versions`
   - Breaking change classification

3. **Agent-Ready**
   - File system workspace export
   - Token budget estimation
   - Agent-specific metadata generation

### Three-Mode Architecture

| Mode | Focus | Key Feature |
|------|-------|-------------|
| Library | Human reading | Tree navigation + full-text search |
| Diff | Change tracking | Hash comparison + alerts |
| Agent | AI integration | Workspace export + context bundle |

---

## 📋 v1.0.0 Release Criteria (Future Goal)

| Criteria | Target | Current |
|----------|--------|---------|
| Stability | Zero panics, major bugs fixed | N/A |
| Test Coverage | ≥ 80% | N/A |
| Documentation | Complete README + Config ref | Partial |
| Migration | v0.1.0 → v1.0.0 smooth | N/A |
| User Feedback | Collected & incorporated | N/A |

---

*This file is updated according to project progress.*
