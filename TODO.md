# matome TODO List

**Project**: matome - Rust CLI for documentation collection and translation  
**Last Updated**: 2026-04-08  
**Test Status**: ✅ 11/11 passing  
**Build Status**: ✅ Compiles

---

## ✅ Completed This Session

The Web UI has been completely redesigned with a documentation portal layout:

- [x] **Sidebar Navigation** - Fixed left panel with logo, search, overview, domain links
- [x] **Article Grid** - Responsive card layout on main content area
- [x] **Search Modal** - ⌘K shortcut with HTMX live search
- [x] **Domain Filtering** - `/domain/:domain` route with sidebar navigation
- [x] **Article Reading View** - Sidebar with nav + language toggle (翻訳/原文)
- [x] **Responsive Design** - Sidebar collapses on mobile (<900px)
- [x] **Design System** - Warm palette, custom fonts, soft shadows

---

## 📋 Remaining TODO List

### 🟢 P3: Missing Features (Nice to Have)

#### Article Management

- [ ] **Delete functionality**
  - Add `matome delete <id>` CLI command
  - Add delete endpoint `DELETE /api/articles/:id`
  - Add delete button in article view
  - Remove from search index on delete

- [ ] **Edit functionality**
  - Edit article title/description
  - Re-translate individual article

#### UI Enhancements

- [ ] **Reading Progress** - Show scroll position in article
- [ ] **Dark Mode** - Toggle between light/dark themes
- [ ] **Bookmarks/Favorites** - Save important articles
- [ ] **Reading History** - Track recently viewed articles
- [ ] **Tags/Categories** - User-defined tagging system

#### Performance

- [ ] **Caching**
  - Cache translation results
  - Skip already-translated content on re-crawl
  - TTL-based cache invalidation

- [ ] **Pagination**
  - Add pagination for large article lists
  - Infinite scroll option

#### Statistics Dashboard

- [ ] Web-based dashboard at `/stats`
- [ ] Total articles, domains
- [ ] Storage usage
- [ ] Last crawl time

---

### 🟡 P2: Documentation

#### README.md

- [ ] Installation (cargo install, from source)
- [ ] Quick start guide (init → add → crawl → serve)
- [ ] Configuration options
- [ ] Glossary format
- [ ] Troubleshooting (common errors)
- [ ] Screenshots of new UI

#### Update SPEC.md

- [ ] Document new web UI architecture
- [ ] Document search features (⌘K, HTMX)
- [ ] Document domain filtering

---

### 🟠 P1: Testing

#### Integration Testing

- [ ] Full pipeline test with real domain
- [ ] Incremental crawl test
- [ ] Search functionality test
- [ ] UI interaction test (sidebar, search modal)

---

## 📊 Progress Matrix

| Component | Code | Tests | Docs | Status |
|-----------|:----:|:-----:|:----:|:------:|
| Foundation | ✅ | ✅ | ✅ | Complete |
| Crawler | ✅ | ⚠️ | ⚠️ | Integration tests needed |
| Extractor | ✅ | ✅ | ✅ | Complete |
| Translator | ✅ | ⚠️ | ✅ | Complete |
| Storage | ✅ | ⚠️ | ✅ | Complete |
| Search | ✅ | ⚠️ | ✅ | Complete |
| **Web UI** | ✅ | ⚠️ | ⚠️ | **New: Redesigned** |

**Legend**: ✅ Done | ⚠️ Partial | ❌ Missing

---

## 🗂️ File Change Log

### 2026-04-08: Web UI Redesign

**New Files Created**:

| File | Description |
|------|-------------|
| `templates/index.html` | Main portal with sidebar navigation |
| `templates/article.html` | Article reading view with sidebar |
| `templates/search.html` | Search results page |

**Files Modified**:

| File | Changes |
|------|---------|
| `src/web/handlers.rs` | Added `domain_articles`, `get_domain_nav`, `get_domain_count`, template rendering with variables |
| `src/web/mod.rs` | Added `/domain/:domain` route |
| `src/web/templates.rs` | Added `render_template` helper |
| `PLAN.md` | Updated with new UI architecture |
| `TODO.md` | Updated with completed tasks |

**Design Features Added**:
- Fixed sidebar with logo, search button, navigation
- Domain filtering via sidebar links
- Quick search modal (⌘K) with HTMX live search
- Responsive design with mobile menu toggle
- Warm cream palette with orange/blue accents
- Custom Google Fonts (Crimson Pro, IBM Plex Sans, JetBrains Mono)

---

## 🏃 Recommended Next Steps

1. **Add README.md** - User documentation with screenshots
2. **Integration Testing** - Test full pipeline with real domain
3. **Delete Functionality** - Article management
4. **Dark Mode** - Theme toggle

---

## 📝 Notes

### Current Architecture

The Web UI follows a documentation portal pattern inspired by Stripe Docs, Vercel Docs:
- Fixed sidebar for navigation
- Main content area with article grid or reading view
- Quick search modal overlay
- Responsive for mobile devices

### Test Coverage

Current: 11 unit tests  
Target: 20-30 tests (unit + integration)

---

*This file is updated after each session.*
