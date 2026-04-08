# matome TODO List

**Project**: matome - Rust CLI for documentation collection and translation  
**Last Updated**: 2026-04-08  
**Phase**: Phase 0 - Emergency Fixes  
**Build Status**: ✅ Compiles | **Test Status**: ✅ 11/11 passing

---

## 🚨 Phase 0: Emergency Fixes (IN PROGRESS)

**目的**: 新規ユーザーが最初のコマンドで詰む箇所を最優先で修正

### 🔴 Critical Issues

#### [ ] 0-1. 設定ファイルキー名統一

**ファイル**: `examples/matome.toml.example`  
**問題**: snake_caseだが、本体はkebab-case

```toml
# 変更前 (snake_case)
data_dir = "./.matome"
target_lang = "ja"

# 変更後 (kebab-case)
data-dir = ".matome"
target-lang = "ja"
```

**対応**:
- [ ] `examples/matome.toml.example` のキーをkebab-caseに変更
- [ ] コメント追加で説明明記

#### [ ] 0-2. 翻訳失敗時のログ出力

**ファイル**: `src/pipeline/mod.rs`  
**問題**: 翻訳失敗をサイレントに握りつぶし

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

**対応**:
- [ ] `warn!` ログのインポート確認
- [ ] 翻訳失敗時にURLとエラー内容を出力

---

## 🎯 Phase 1: Code Quality Foundation (PLANNED)

### 🟠 High Priority

#### [ ] 1-1. Extractor Clone設計修正

**ファイル**: `src/pipeline/extractor.rs`, `src/pipeline/mod.rs`  
**問題**: `#[derive(Clone)]` を使わず独自メソッドで応急処置

**対応**:
- [ ] Extractorに `#[derive(Clone)]` を付与
- [ ] `src/pipeline/mod.rs` の独自cloneメソッド削除

#### [ ] 1-2. テンプレート管理統一

**ファイル**: `src/web/handlers.rs`, `templates/`  
**問題**: handlers.rsに500行以上のインラインHTML + templates/ファイル二重管理

**対応**:
- [ ] 外部テンプレートファイル使用に一本化
- [ ] handlers.rsのインラインHTML削除
- [ ] Askama使用の場合はAskamaに完全移行

---

## ⚙️ Phase 2: Scalability (PLANNED)

### 🟠 High Priority

#### [ ] 2-1. SQLite コネクションプール化

**ファイル**: `src/db/sqlite.rs`  
**問題**: `Mutex<Connection>` で全操作を直列化、serve時並列リクエストが律速

**対応**:
- [ ] `r2d2` または `deadpool-sqlite` 導入
- [ ] `Database` 構造体をコネクションプール使用にリファクタ

### 🟡 Medium Priority

#### [ ] 2-2. SearchResult ID設計整理

**ファイル**: `src/db/search.rs`  
**問題**: Tantivy ID と SQLite AUTOINCREMENT ID が別物

**対応**:
- [ ] `SearchResult.id` フィールドを削除
- [ ] コメント追加でURL→DBマッピングを明示

---

## 🔧 Phase 3: Feature Fixes (PLANNED)

### 🟡 Medium Priority

#### [ ] 3-1. max_pages機能実装

**ファイル**: `src/pipeline/crawler.rs`  
**問題**: 設定できるが効かない

**対応**:
- [ ] crawler.rsに上限チェックロジック追加
- [ ] 設定`max_pages = 0`で無制限、`max_pages = N`でN件上限

#### [ ] 3-2. incremental crawl改善

**ファイル**: `src/pipeline/crawler.rs`  
**問題**: `docs.example.com` と `example.com` が別ドメイン判定

**対応**:
- [ ] 設定に `treat_subdomains_same` オプション追加
- [ ] domain抽出ロジック改善

---

## 🧹 Phase 4: Technical Debt (PLANNED)

### 🟡 Medium Priority

#### [ ] 4-1. glossary.rs unwrap()撲滅

**ファイル**: `src/pipeline/glossary.rs`  
**問題**: 二重unwrap

```rust
// 変更前
.unwrap_or_else(|_| regex_lite::Regex::new("").unwrap())

// 変更後
const EMPTY_PATTERN: &str = "";
```

#### [ ] 4-2. コードスニペット取得対応

**ファイル**: `src/pipeline/extractor.rs`  
**問題**: ページ内のコードブロックが取得できていない

**対応**:
- [ ] 現在の取得スコープ調査
- [ ] extractor усиление или scoped拡大

---

## 📦 Backlog: Future Enhancements

### 🟢 P3: Nice to Have

#### Article Management
- [ ] `matome delete <id>` CLI コマンド
- [ ] `DELETE /api/articles/:id` エンドポイント
- [ ] 記事ビューに削除ボタン

#### UI Enhancements
- [ ] Reading Progress 表示
- [ ] Dark Mode対応
- [ ] Bookmarks/Favorites
- [ ] Reading History
- [ ] Tags/Categories

#### Performance
- [ ] 翻訳結果キャッシュ
- [ ] 再クロール時の翻訳スキップ
- [ ] ページネーション
- [ ] Infinite scroll

#### Statistics Dashboard
- [ ] `/stats` エンドポイント
- [ ] ストレージ使用量表示

---

## 📊 Progress Matrix

| Component | Code | Tests | Docs | Priority Fix |
|-----------|:----:|:-----:|:----:|:------------:|
| Foundation | ✅ | ✅ | ✅ | - |
| Crawler | ✅ | ⚠️ | ⚠️ | Phase 3 |
| Extractor | ✅ | ✅ | ✅ | Phase 1 |
| Translator | ✅ | ⚠️ | ✅ | Phase 0 |
| Storage | ✅ | ⚠️ | ✅ | Phase 2 |
| Search | ✅ | ⚠️ | ✅ | Phase 2 |
| Web UI | ✅ | ⚠️ | ⚠️ | Phase 1 |

**Legend**: ✅ Done | ⚠️ Partial | ❌ Missing

---

## 📝 Notes

### 設定ファイルキー命名規則

Phase 0で**kebab-case**に統一:

```toml
# ✅ 正
data-dir = ".matome"
target-lang = "ja"
glossary-file = "glossary.toml"

# ❌ 誤 (修正対象)
data_dir = ".matome"
target_lang = "ja"
```

### エラーハンドリング方針

Phase 0で翻訳失敗時のログ出力を追加:
- パイプライン全体の停止を避ける
- 原文へのフォールバックを継続
- ユーザーに問題発生を通知

---

## 📅 スケジュール

```
Week 1: Phase 0
├── 設定ファイルキー名統一
├── 翻訳失敗ログ出力
└── README 初期セットアップ注意事項追記

Week 2-3: Phase 1
├── Extractor #[derive(Clone)] 化
└── テンプレート管理一本化

Week 4-5: Phase 2
├── SQLite コネクションプール
└── SearchResult ID整理

Week 6+: Phase 3
├── max_pages 実装
└── incremental crawl 改善
```

---

*This file is updated after each session.*
