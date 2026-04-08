# matome TODO List

**Project**: matome - Rust CLI for documentation collection and translation  
**Last Updated**: 2026-04-08  
**Status**: ✅ ALL TASKS COMPLETED  
**Build Status**: ✅ Compiles | **Test Status**: ✅ 15/15 passing

---

## ✅ ALL PHASES COMPLETED

### Phase 0: Emergency Fixes ✅

- [x] 設定ファイルキー名統一 (snake_case → kebab-case)
- [x] 翻訳失敗時のログ出力 (warn!実装済み)

### Phase 1: Code Quality Foundation ✅

- [x] Extractor Clone修正 (#[derive(Clone)]化)
- [x] テンプレート管理統一 (include_str!(), 711行→301行)

### Phase 2: Scalability ✅

- [x] SQLite WALモード有効化
- [x] SearchResult ID設計整理 (ドキュメント追加)

### Phase 3: Feature Fixes ✅

- [x] max_pages機能 (実装済み確認)
- [x] incremental crawl改善 (treat_subdomains_sameオプション追加)

### Phase 4: Technical Debt ✅

- [x] glossary.rs unwrap()撲滅
- [x] コードスニペット取得対応 (Docusaurus/MkDocs対応)

---

## 📦 Future Enhancements (Backlog)

### 🟢 P3: Nice to Have

#### Article Management
- [ ] `matome delete <id>` CLI コマンド
- [ ] Web UIに削除ボタン

#### UI Enhancements
- [ ] Reading Progress 表示
- [ ] Dark Mode対応
- [ ] Bookmarks/Favorites
- [ ] Reading History
- [ ] Tags/Categories

#### Performance
- [ ] 翻訳結果キャッシュ
- [ ] Pagination / Infinite scroll

#### Statistics
- [ ] `/stats` エンドポイント

---

## 📊 Progress Matrix

| Component | Code | Tests | Docs | Status |
|-----------|:----:|:-----:|:----:|:------:|
| Foundation | ✅ | ✅ | ✅ | Complete |
| Crawler | ✅ | ✅ | ✅ | Complete |
| Extractor | ✅ | ✅ | ✅ | Complete |
| Translator | ✅ | ✅ | ✅ | Complete |
| Storage | ✅ | ✅ | ✅ | Complete |
| Search | ✅ | ✅ | ✅ | Complete |
| Web UI | ✅ | ✅ | ✅ | Complete |

**Legend**: ✅ Done

---

## 📝 Implementation Notes

### Code Quality

```
handlers.rs:     711 lines → 301 lines (58% reduction)
Tests:           11 → 15 passed
unwrap():       All eliminated
WAL mode:       Enabled
Code extraction: Docusaurus/MkDocs supported
```

### Configuration

```toml
# kebab-case命名規則
data-dir = ".matome"
target-lang = "ja"
treat-subdomains-same = true  # Optional
```

### Dependencies

- SQLite: rusqlite 0.32 (WAL mode)
- Search: tantivy 0.25
- Web: axum 0.7 + htmx
- HTML: scraper 0.26

---

## 📅 Development Timeline

| Period | Phase | Status |
|--------|-------|--------|
| Week 1 | Phase 0 | ✅ Complete |
| Week 2-3 | Phase 1 | ✅ Complete |
| Week 4-5 | Phase 2 | ✅ Complete |
| Week 6-7 | Phase 3 | ✅ Complete |
| Week 8 | Phase 4 | ✅ Complete |

---

*This file is updated after each session.*
