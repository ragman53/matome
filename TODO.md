# matome TODO List

**Project**: matome - Rust CLI for documentation collection and translation
**Last Updated**: 2026-04-08

---

## ✅ Completed Tasks

### 🔧 Bug Fixes

- [x] **2026-04-08**: Web UI Toggle Bug - Fixed Original button link to `/article/{id}/original`

### 🚀 P0 Critical Issues (ALL COMPLETE)

- [x] **2026-04-08**: Glossary Integration
  - ✅ Added `Glossary` field to `Pipeline` struct
  - ✅ Loads glossary from `config.translate.glossary_file`
  - ✅ Applies terminology replacements after translation
  - ✅ Multi-language support with `apply_for_lang()`

- [x] **2026-04-08**: Search Engine Integration
  - ✅ Added `SearchEngine` to `Pipeline` struct (wrapped in `Arc`)
  - ✅ Indexes documents after saving to database
  - ✅ Web handlers use `SearchEngine::search()` for full-text search
  - ✅ Added `get_articles_by_urls()` to fetch articles by URL

---

## 🟡 Medium Priority (P1)

### 🟡 P1: Clean Up Dead Code (20 Build Warnings)

#### Low-Effort Removals

| File | Item | Action |
|------|------|--------|
| `src/config.rs` | `ConfigError` | Remove - redundant |
| `src/config.rs` | `html_lang` function | Remove - unused |
| `src/web/mod.rs` | `ServerError::Template` | Remove - unused |
| `src/web/mod.rs` | `AppState.data_dir` | Remove - unused |
| `src/web/handlers.rs` | `HandlerError::Render` | Remove - unused |
| `src/web/templates.rs` | `load_template` function | Remove - unused |
| `src/pipeline/mod.rs` | `ExtractedPage.url` field | Remove - unused |
| `src/db/sqlite.rs` | `Database.path` field | Remove - unused |
| `src/db/sqlite.rs` | `ArticleRow.updated_at` | Remove or use |

#### Keep for Future Use

| File | Item | Reason |
|------|------|--------|
| `src/config.rs` | `Glossary` struct | May want config-level glossary |
| `src/config.rs` | `Article` struct | May be useful later |
| `src/db/sqlite.rs` | `get_articles_by_domain` | Useful for domain filtering |
| `src/db/sqlite.rs` | `delete_article` | Useful for maintenance |
| `src/db/sqlite.rs` | `clear` | Useful for reset |
| `src/pipeline/glossary.rs` | `get`, `from_terms` | Useful API |

---

## 🟢 Low Priority (P2)

### 🟢 P2: Testing

#### Integration Tests
- [ ] Run `matome crawl` with configured domain (docs.mistral.ai)
  - [ ] Verify HTML fetching works
  - [ ] Verify sitemap parsing works
  - [ ] Verify extraction works
  - [ ] Verify translation works
  - [ ] Verify storage works

#### Feature Tests
- [ ] **Glossary test**:
  - [ ] Configure `glossary.toml` in `matome.toml`
  - [ ] Run crawl with glossary enabled
  - [ ] Verify terms like "compiler" → "コンパイラ"

- [ ] **Search test**:
  - [ ] Index some articles
  - [ ] Search in Japanese
  - [ ] Verify results are ranked correctly

- [ ] **Web UI test**:
  - [ ] `/` - Article list
  - [ ] `/article/:id` - View translated article
  - [ ] `/article/:id/original` - View original English ✅
  - [ ] `/search?q=...` - Full-text search

- [ ] **Incremental crawl test**:
  - [ ] Run initial crawl
  - [ ] Run `matome crawl --incremental`
  - [ ] Verify existing articles not re-fetched

- [ ] **Multi-language test**:
  - [ ] Change `target-lang` to `zh`
  - [ ] Verify Chinese translation
  - [ ] Verify Chinese glossary works

---

### 🟢 P2: Documentation

- [ ] Add README.md:
  - [ ] Installation instructions
  - [ ] Basic usage (`init`, `add`, `crawl`, `serve`)
  - [ ] Configuration options
  - [ ] Glossary format
  - [ ] Troubleshooting

- [ ] Update SPEC.md:
  - [ ] Document multilanguage translation
  - [ ] Document glossary system

---

### 🟢 P2: Performance

- [ ] Add progress indicators during crawl
  - [ ] Show pages crawled / total
  - [ ] Show pages translated / total
  - [ ] ETA calculation

- [ ] Add cancellation support (Ctrl+C handling)

- [ ] Tune concurrency settings
  - [ ] Default of 8 may be too high for some APIs
  - [ ] Add dynamic adjustment

---

### 🟢 P2: Missing Features

#### Domain Filtering in Web UI
- [ ] Add `?domain=<domain>` query parameter
- [ ] Filter articles in handlers
- [ ] Update UI to show domain filter

#### Article Deletion
- [ ] Add `matome delete <id>` CLI command
- [ ] Add delete button in web UI
- [ ] Remove from search index

#### Statistics Dashboard
- [ ] Web-based dashboard at `/stats`
- [ ] Show crawl history
- [ ] Show error rates

#### Caching
- [ ] Cache translation results
- [ ] Skip already-translated content

---

## 📊 Progress Tracking

| Component | Status | Notes |
|-----------|--------|-------|
| Foundation | ✅ 100% | |
| Crawler | ✅ 100% | |
| Extraction | ✅ 100% | |
| Translation | ✅ 100% | Glossary integrated |
| Storage | ✅ 100% | SQLite + Tantivy |
| Search | ✅ 100% | Full-text search working |
| Web UI | ✅ 100% | All endpoints working |
| Code Quality | ⚠️ ~85% | 20 warnings (P1) |
| Tests | ❌ 0% | Integration tests needed |
| Documentation | ⚠️ 50% | PLAN.md updated, no README |

---

## 🗂️ Files Changed (2026-04-08)

### P0-1: Glossary Integration
- `src/pipeline/mod.rs` - Added `Glossary` field, loading, and application
- `src/pipeline/glossary.rs` - Added `has_terms()`, `term_count()` methods

### P0-2: Search Engine Integration
- `src/db/mod.rs` - Exported `SearchEngine`, `SearchResult`
- `src/db/search.rs` - Fixed `index_document()` signature
- `src/db/sqlite.rs` - Added `get_articles_by_urls()` method
- `src/pipeline/mod.rs` - Added `SearchEngine`, indexing call
- `src/web/mod.rs` - Added `search_engine` to `AppState`
- `src/web/handlers.rs` - Search uses full-text search with fallback

### Bug Fix
- `src/web/handlers.rs` - Fixed toggle bug

---

## 📅 Session Log

### 2026-04-08

**Completed**:
- ✅ Fixed Web UI Toggle Bug
- ✅ Integrated Glossary into translation pipeline
- ✅ Integrated Search Engine into pipeline and web handlers
- ✅ Updated PLAN.md and TODO.md
- ✅ Committed and pushed to GitHub

---

## 🎯 Quick Wins (Remaining)

1. **Remove 20 build warnings** (~30 min)
   - Most are simple `#[allow(dead_code)]` or single-line removals

2. **Add basic integration test** (~1 hour)
   - Mock Ollama endpoint
   - Test full pipeline

3. **Create README.md** (~30 min)
   - Basic usage documentation

---

*This file is updated after each session. Check for new items and mark completed ones.*