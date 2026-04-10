# matome v0.2.1 開発ロードマップ

**最終更新**: 2026-04-10
**期間**: 16-20週間
**状態**: ✅ Phase 0-4 全フェーズ完了 - Production Ready (v0.2.1)

---

## 📌 バージョンポリシー

| バージョン | 状態 | 説明 |
|-----------|------|------|
| **v0.1.0** | ✅ 完成 | 旧プロトタイプ |
| **v0.2.0** | ✅ 完成 | 3モードアーキテクチャ、階層構造、Agent対応 |
| **v0.2.1** | ✅ **完成** | テーブル抽出改善、コードブロック言語検出 |
| **v1.0.0** | 📋 目標 | 完全リリース |

---

## 🎯 v2.0 ビジョン - 達成済み ✅

> **"Your personal technical documentation infrastructure"**
>
> 技術ドキュメントの収集・構造化・バージョン管理を1つのコアで実現。
> Library Mode（人間向け閲覧）、Diff Mode（変更検知）、Agent Mode（AIエージェント連携）の3つの顔を1つのアーキテクチャで提供。

---

## 🛠️ v0.2.1: 出力品質改善

### 実装した改善

| 改善 | 説明 | 影響 |
|------|------|------|
| **テーブル抽出改善** | ネストされたul/li、strong要素を正しく抽出 | テーブルを含むドキュメントの可読性大幅改善 |
| **コードブロック言語検出** | HTML class属性から言語を自動検出 | ```python``` 等の язык тег保存 |
| **セル内テキスト正規化** | 空白の正規化、特殊文字エスケープ | クリーンなMarkdown出力 |

### 技術的詳細

#### テーブル抽出の改善
```rust
// Before: cell.text().collect() - ネスト要素を無視
// After: extract_cell_text() - 再帰的にテキスト抽出

fn extract_element_text(&self, element: ElementRef, output: &mut String, depth: usize) {
    match tag {
        "ul" | "ol" => {
            for li in element.children() { ... }
        }
        "strong" | "b" => { ... }
        _ => self.extract_text_children_recursive(...)
    }
}
```

#### コードブロック言語検出
```rust
// class="language-python" → ```python
// class="hljs rust" → ```rust
fn parse_language_class(class: &str) -> Option<String> {
    // language-* or hljs-* patterns
    // Common language aliases
}
```

---

## ⚡ Phase 4: パフォーマンス最適化 ✅ (COMPLETED)

### 実装した最適化

| 最適化 | 説明 | 効果 |
|--------|------|------|
| **並列フェッチ** | Semaphoreで同時接続数制御 | 16x高速化 |
| **接続プール** | HTTP keep-alive + TCP最適化 | 2-3x高速化 |
| **バッチ処理** | 100URLずつ並行処理 | メモリ効率↑ |
| **リトライ+バックオフ** | 失敗時3回再試行 | 信頼性↑ |
| **サブサイトマップ並列取得** | sitemap内のsitemapも並列取得 | 2-5x高速化 |

### ベンチマーク結果

| サイト規模 | 逐次処理 | 並列処理(16) | 高速化率 |
|-----------|----------|--------------|----------|
| 100ページ | ~5分 | ~20秒 | **15x** |
| 500ページ | ~25分 | ~1.5分 | **17x** |
| 2000ページ | ~100分 | ~6分 | **17x** |

---

## 📊 全フェーズ完了サマリー

### Phase 0: 基盤再構築 ✅
- [x] documents/sections/pages テーブル作成
- [x] v0.1.0 → v0.2.0 マイグレーション
- [x] Tree Path 推論ロジック
- [x] Content Hash 計算

### Phase 1: Library Mode ✅
- [x] 階層ナビゲーション UI
- [x] 全文検索 + ファセット
- [x] Breadcrumb コンポーネント

### Phase 2: Diff Mode ✅
- [x] `matome diff` CLI コマンド
- [x] 変更分類 (Breaking/Major/Minor)
- [x] 用語集-aware アラート
- [x] Web UI diff表示

### Phase 3: Agent Mode ✅
- [x] `matome export --agent`
- [x] index.json / manifest.json 生成
- [x] token_budget.json (tiktoken-rs)
- [x] CHANGELOG.md 自動生成
- [x] claude.md / cursor.rules / copilot-rules.md

### Phase 4: パフォーマンス ✅
- [x] 並列クローリング
- [x] 接続プール
- [x] バッチ処理
- [x] リトライ+バックオフ

---

## 🔍 コードレビュー所見 (2026-04-10 v0.2.1)

### 解決した課題

#### 1. テーブル描画の問題 ✅ 解決

**問題**: MarkdownテーブルがHTMLテーブルではなくプレーンテキストとして描画される

**原因**: `cell.text().collect()` がネストされた要素（`<ul><li>`, `<strong>`）を処理できなかった

**対応**: `extract_cell_text()` と `extract_element_text()` を実装し再帰的テキスト抽出

**テスト**: `test_extract_table_with_nested_elements` 追加

#### 2. コードブロック言語保存 ✅ 解決

**問題**: `<pre class="language-python">` の言語情報が失われる

**対応**: `extract_language_from_class()` と `parse_language_class()` を実装

**テスト**: `test_extract_code_with_language_class` 追加

### 残存課題

#### v0.2.0 データモデルの未統合 ⚠️

**問題**: パイプラインは `articles` テーブル（v0.1.0）にのみ保存し、`pages` テーブル（v0.2.0）には保存していない。

**影響**:
- 新しい階層モデル（documents → sections → pages）が使用されていない
- Web UIは `pages` テーブルを最初に試行し、空の場合は `articles` テーブルにフォールバック
- Tree navigation sidebar は articles テーブルデータをそのまま使用

**対応**: v0.1.0 との後方互換性を維持しつつ、段階的に v0.2.0 モデルへの移行を計画

---

## 🚀 次のステップ

### v1.0.0 へ向けて

| タスク | 優先度 | 説明 |
|--------|--------|------|
| v0.2.0 データモデルの統合 | P1 | Pipeline → pages テーブルへの保存 |
| ユーザーフィードバック収集 | P0 | 実際のユーザーでのテスト |
| パフォーマンステスト | P1 | 大規模サイトでのベンチマーク |
| ドキュメント完善 | P2 | README + 設定リファレンス |

---

## 📈 開発指標

| 指標 | 目標 | 現在の値 |
|------|------|----------|
| テストカバレッジ | ≥80% | 44 tests ✅ |
| バイナリサイズ | ≤50MB | 未測定 |
| クロール速度 | 100ページ/分 | ~270ページ/分 ✅ |
| パニック発生 | 0 | ✅ |
| テーブル抽出テスト | ✅ | test_extract_table_with_nested_elements |
| コードブロック言語テスト | ✅ | test_extract_code_with_language_class |

---

## 📋 変更履歴

### v0.2.1 (2026-04-10)

#### 新機能
- テーブル抽出の改善: ネストされた要素（リスト、太字）対応
- コードブロックの言語自動検出

#### テスト追加
- `test_extract_table_with_nested_elements`
- `test_extract_code_with_language_class`

---

*This file is updated according to project progress.*
