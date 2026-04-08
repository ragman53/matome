# matome TODO List

**Project**: matome - Rust CLI for documentation collection and translation
**Last Updated**: 2026-04-08

---

## ✅ Completed Tasks

### 🔧 Bug Fixes

- [x] **2026-04-08**: Web UI Toggle Bug - Fixed Original button link to `/article/{id}/original`

### 🚨 P0 Critical Issues (COMPLETED)

- [x] **2026-04-08**: Glossary Integration
  - Added `Glossary` field to `Pipeline` struct
  - Loads glossary from `config.translate.glossary_file`
  - Applies terminology replacements after translation via `apply_for_lang()`

- [x] **2026-04-08**: Search Engine Integration
  - Added `SearchEngine` to `Pipeline` struct (wrapped in `Arc`)
  - Indexes documents after saving to database
  - Web handlers now use `SearchEngine::search()` for full-text search
  - Added `get_articles_by_urls()` to fetch articles by URL list

---

## 🟡 Medium Priority (P1)

### 🟡 P1: Clean Up Dead Code (22 Build Warnings)

The following types and functions are defined but never used. These are candidates for removal:

#### src/config.rs - Unused Types
| Item | Action |
|------|--------|
| `ConfigError` enum | **Remove** - redundant with other errors |
| `html_lang` function | **Remove** |
| `Glossary` struct | **Keep** - used by pipeline |
| `Article` struct | **Keep** - may be useful later |

#### src/pipeline/glossary.rs - Unused Code
| Item | Action |
|------|--------|
| `HashMap` import | **Remove** |
| `from_terms`, `apply`, `get` methods | **Keep** - useful API |

#### src/db/search.rs - Unused Fields/Methods
| Item | Action |
|------|--------|
| `generate_id` method | **Remove** - inlined |
| `SearchResult` struct | **Keep** - used by handlers |
| `search`, `clear`, `doc_count` methods | **Keep** - used by handlers |

#### src/web/ - Minor Cleanup
| Item | Action |
|------|--------|
| `ServerError::Template` variant | **Remove** |
| `HandlerError::Render` variant | **Remove** |
| `load_template` function | **Remove** |
| `ExtractedPage::url` field | **Remove** |
| `Database::path` field | **Remove** |
| `ArticleRow::updated_at` field | **Remove** or use |
| `AppState::data_dir` field | **Remove** |

---

## 🟢 Low Priority (P2)

### 🟢 P2: Testing

- [ ] **Integration test**: Run `matome crawl` with configured domain
  - [ ] Verify HTML fetching works
  - [ ] Verify sitemap parsing works
  - [ ] Verify extraction works
  - [ ] Verify translation works
  - [ ] Verify storage works

- [ ] **Glossary test**: Create test glossary and verify replacement
  - [ ] Configure `glossary.toml` in `matome.toml`
  - [ ] Run crawl with glossary enabled
  - [ ] Verify terms are replaced correctly

- [ ] **Search test**: Verify full-text search returns relevant results
  - [ ] Index some articles
  - [ ] Search in Japanese
  - [ ] Verify results are ranked correctly

- [ ] **Web UI test**: Verify all endpoints work
  - [ ] `/` - Article list
  - [ ] `/article/:id` - View translated article
  - [ ] `/article/:id/original` - View original English ✅ (fixed)
  - [ ] `/search?q=...` - Full-text search

- [ ] **Incremental crawl test**:
  - [ ] Run initial crawl
  - [ ] Run `matome crawl --incremental`
  - [ ] Verify existing articles not re-fetched

---

### 🟢 P2: Documentation

- [ ] Add README.md with:
  - Installation instructions
  - Basic usage (`matome init`, `matome add`, `matome crawl`, `matome serve`)
  - Configuration options
  - Glossary format
  - Troubleshooting

- [ ] Document the multilanguage translation system
  - How to configure target language
  - How to use glossary for terminology
  - Supported languages list

---

### 🟢 P2: Performance

- [ ] Add progress indicators during crawl
  - Show pages crawled / total
  - Show pages translated / total
  - ETA calculation

- [ ] Add cancellation support (Ctrl+C)

- [ ] Tune concurrency settings
  - Default of 8 may be too high for some APIs
  - Add `--max-concurrency` flag

---

## 📋 Missing Features (Not Implemented)

### Feature: Domain Filtering in Web UI

**Problem**: Web UI shows all articles. No way to filter by domain.

**Needed**:
- [ ] Add `?domain=<domain>` query parameter
- [ ] Filter articles in handlers
- [ ] Update UI to show domain filter dropdown

### Feature: Article Deletion

**Problem**: No way to remove articles via CLI or web UI.

**Needed**:
- [ ] Add `matome delete <id>` CLI command
- [ ] Add delete button in web UI
- [ ] Remove from search index when deleted

### Feature: Statistics/Dashboard

**Problem**: `status` command is basic. No visual dashboard.

**Nice to have**:
- [ ] Web-based dashboard at `/stats`
- [ ] Show crawl history
- [ ] Show error rates
- [ ] Show translation quality metrics

---

## 📊 Progress Tracking

| Component | Status | Notes |
|-----------|--------|-------|
| **Translation System** | ✅ 100% | Glossary integrated |
| **Glossary Module** | ✅ 100% | Now integrated into pipeline |
| **Search Engine** | ✅ 100% | Now integrated, used in handlers |
| **Web UI** | ✅ 100% | Toggle bug fixed |
| **Code Quality** | ⚠️ 75% | 22 warnings (P1) |
| **Tests** | ❌ 0% | No integration tests |
| **Documentation** | ❌ 0% | No README |

---

## 🗂️ Files Changed (2026-04-08)

### P0-1: Glossary Integration
- `src/pipeline/mod.rs` - Added `Glossary` field, loading, and application
- `src/pipeline/glossary.rs` - Added `has_terms()` and `term_count()` methods

### P0-2: Search Engine Integration
- `src/db/mod.rs` - Exported `SearchEngine` and `SearchResult`
- `src/db/search.rs` - Fixed `index_document()` signature, added `Clone`
- `src/db/sqlite.rs` - Added `get_articles_by_urls()` method
- `src/pipeline/mod.rs` - Added `SearchEngine` field, indexing call
- `src/web/mod.rs` - Added `search_engine` to `AppState`
- `src/web/handlers.rs` - Updated search handlers to use `SearchEngine`

---

## 📅 Session Log

### 2026-04-08

**Completed**:
- ✅ Fixed Web UI Toggle Bug (Original button now links to `/article/{id}/original`)
- ✅ Integrated Glossary into translation pipeline
  - Loads glossary from config
  - Applies term replacements after translation
- ✅ Integrated Search Engine into pipeline and web handlers
  - Documents indexed during crawl
  - Web search uses full-text search with fallback to SQLite LIKE

---

*This file is updated after each session. Check for new items and mark completed ones.*