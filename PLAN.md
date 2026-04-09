# matome v0.2.0 開発ロードマップ

**最終更新**: 2026-04-09
**期間**: 16-20週間
**状態**: 🔄 開発中 (Direction Finding Phase)

---

## 📌 バージョンポリシー

| バージョン | 状態 | 説明 |
|-----------|------|------|
| **v0.1.0** | ✅ 完成 | 現行プロトタイプ |
| **v0.2.0** | 🔄 開発中 | このロードマップの方向性 |
| **v1.0.0** | 📋 目標 | 完全リリース |

---

## 🎯 v2.0 ビジョン

> **"Your personal technical documentation infrastructure"**
>
> 技術ドキュメントの収集・構造化・バージョン管理を1つのコアで実現。
> Library Mode（人間向け閲覧）、Diff Mode（変更検知）、Agent Mode（AIエージェント連携）の3つの顔を1つのアーキテクチャで提供。
>
> **📌 現在の方向性を探る段階** - ユーザーのフィードバックを収集しながら実装を進めます。

---

## 📊 Phase 0: 基盤再構築 (1-2週間)

### 目標
既存のプロトタイプ基盤を「階層構造・バージョン管理可能な設計」に再構築

### タスク詳細

#### 0.1 データモデル移行 (7日)
```
新しいテーブル:
- documents: クロール対象サイト単位
- sections: 論理的大分類(URLパターン/TOCから生成)
- pages: 実コンテンツ(階層情報付き)
- page_versions: 変更履歴
```

**マイグレーション SQL**:
```sql
-- 新規テーブル作成
CREATE TABLE documents (...);
CREATE TABLE sections (...);
CREATE TABLE pages (...);
CREATE TABLE page_versions (...);

-- 既存 articles データフォールバック
UPDATE pages SET tree_path = '/page/' || id WHERE tree_path IS NULL;
UPDATE pages SET breadcrumbs = json('["' || tree_path || '"]') WHERE breadcrumbs IS NULL;
```

#### 0.2 Tree Path 推論ロジック (5日)
```rust
pub fn infer_tree_path(url: &str, base_url: &str) -> String {
    url.strip_prefix(base_url)
       .unwrap_or("/")
       .trim_end_matches('/')
       .to_string()
}

// 例: "https://docs.example.com/api/v2/auth" → "/api/v2/auth"
```

**テストケース**:
- [ ] `/` のみのパス
- [ ] 深いネスト (4段階以上)
- [ ] クエリパラメータ除去
- [ ] index.html除去
- [ ] 末尾スラッシュ正規化

#### 0.3 Content Hash 計算 (3日)
```rust
pub fn compute_hash(content: &str) -> String {
    let normalized = normalize_for_comparison(content);
    Sha256::digest(normalized.as_bytes())
}

pub fn normalize_for_comparison(content: &str) -> String {
    // 空白正規化、改行統一、ID/class除去
}
```

**テストケース**:
- [ ] 空白量の違いを同一判定
- [ ] 改行スタイルの違いを同一判定
- [ ] クラス名除去後の同一判定
- [ ] コードブロックは完全に除外

---

## 📚 Phase 1: Library Mode 完全対応 (2-3週間)

### 目標
Web UIで階層ナビゲーション・全文検索・パンくずを提供

### タスク詳細

#### 1.1 階層ナビゲーション UI (10日)
```
Left Sidebar:
├── Rust Book/
│   ├── Getting Started/
│   │   ├── Installation.md  ← クリックで本文表示
│   │   └── Hello World.md
│   └── Ownership/
│       ├── Borrowing.md
│       └── Lifetimes.md
└── Tokio Docs/
    └── ...
```

**技術スタック**:
- HTMX + Alpine.js
- daisyUI (またはTailwind手動)
- サーバーサイドレンダリング

#### 1.2 全文検索 + ファセット (7日)
```rust
// Tantivy クエリ例
let query = tantivy::query::QueryParser::for_index(index, vec!["title", "content"]);
let parsed = query.parse_query("runtime +section:api")?;
```

**対応フィールド**:
- `title`: 完全一致重視
- `content`: 全文検索
- `tree_path`: ファセットフィルタ
- `doc_version`: バージョン固定検索

#### 1.3 Breadcrumb  компонента (3日)
```
📚 Rust Book > 📖 Getting Started > Installation
         ↑ パンくずナビゲーション
```

---

## 🔄 Phase 2: Diff Mode (3-4週間)

### 目標
ドキュメント変更を自動的に検出し、破壊的変更をアラート

### タスク詳細

#### 2.1 変更検知エンジン (7日)
```rust
pub enum ChangeType {
    None,       // 同一
    Minor,      // typo/フォーマット
    Major,      // セクション書き直し
    Breaking,   // 用語集重要語彙変更
}

pub async fn detect_and_record(page: &Page, new_content: &str) -> Result<ChangeResult> {
    let new_hash = compute_hash(new_content);
    if page.content_hash == new_hash {
        return Ok(ChangeResult::Unchanged);
    }

    let change_type = classify_change(&page.content, new_content);
    let glossary_alerts = check_glossary_terms(&page.content, new_content);

    // page_versions に記録
    db::insert_version(page.id, new_hash, change_type).await?;

    Ok(ChangeResult::Changed { change_type, glossary_alerts })
}
```

#### 2.2 `matome diff` CLI (5日)
```bash
# 使用例
$ matome diff --since 2024-05-01

📄 変更検出: tokio-docs (v1.37.0 → v1.38.0)

🔴 Breaking Changes:
  • /api/runtime.md - "spawn" の定義が変更
  • /guides/async.md - "await" 構文の BREAKING CHANGE

🟠 Major Changes:
  • /api/task.md - set_join_handle シグネチャ変更

🟡 Minor Changes:
  • /api/rt.md - typo修正 (12箇所)

✅ Unchanged:
  • /api/sync/*.md (7 pages)
```

#### 2.3 用語集連携アラート (7日)
```toml
# glossary.toml
[[terms]]
en = "async"
ja = "非同期"
priority = "high"  # Breaking Change としてフラグ
```

**検出ロジック**:
1. 用語集の重要語彙(priority=high)を抽出
2. 旧・新コンテンツ双方で一致率を計算
3. 重要語彙が削除/変更 → Breaking フラグ

#### 2.4 Web UI diff 表示 (5日)
- text-diff crate で行単位差分ハイライト
- サイドバイサイド or インライン表示切替
- 変更部分へのジャンプ機能

---

## 🤖 Phase 3: Agent Mode (4-5週間)

### 目標
AIコーディングエージェント向けの構造化ワークスペースを自動生成

### タスク詳細

#### 3.1 ワークスペース生成基盤 (10日)
```rust
pub struct AgentExporter {
    workspace_name: String,
    base_dir: PathBuf,
    token_estimator: Tiktoken,
}

impl AgentExporter {
    pub async fn export(&self, pages: &[Page]) -> Result<()> {
        // 1. ディレクトリ構造生成
        self.create_directory_tree().await?;

        // 2. Markdown ファイル書き出し
        for page in pages {
            self.write_markdown(page).await?;
            self.write_page_meta(page).await?;
        }

        // 3. メタファイル生成
        self.generate_index_json().await?;
        self.generate_manifest().await?;
        self.generate_token_budget().await?;
    }
}
```

**出力構造**:
```
~/.matome/workspaces/tokio-docs/
├── index.json
├── runtime/
│   ├── _index.md
│   ├── runtime.md
│   └── _agent/
│       └── page_meta.json
├── sync/
│   └── ...
└── _agent/
    ├── manifest.json
    ├── CHANGELOG.md
    ├── token_budget.json
    ├── workspace.yaml
    ├── claude.md
    └── cursor.rules
```

#### 3.2 トークン見積もり (5日)
```rust
use tiktoken_rs::{cl100k_base, Encoding};

// tiktoken-rs で正確なトークンカウント
pub fn count_tokens(text: &str) -> usize {
    let encoding = cl100k_base().unwrap();
    encoding.encode(text).len()
}
```

**token_budget.json 出力例**:
```json
{
  "context_limit": 128000,
  "total_tokens": 185000,
  "recommended_reading_order": [
    { "section": "runtime", "files": ["_index.md", "runtime.md"], "tokens": 8200 }
  ],
  "priority_files": ["index.json", "runtime/_index.md"]
}
```

#### 3.3 CHANGELOG.md 自動生成 (5日)
```bash
# 出力例
## tokio-docs Changelog (2024-05-20)

### 🔴 Breaking Changes
- **/api/runtime.md**: `spawn` signature changed
  - Old: `spawn<T>(future: T) -> JoinHandle<T>`
  - New: `spawn<T>(future: T) -> JoinHandle<T, JoinError>`

### 🟠 Major Changes
- **/guides/async.md**: Async runtime architecture section rewritten
```

#### 3.4 エージェント設定テンプレート (5日)
```markdown
# claude.md (自動生成)
/ <!-- Auto-generated by matome -->
# Tokio Documentation Workspace

## Navigation
1. Start with `index.json` for module hierarchy
2. Code examples are authoritative - never rewrite
3. Check `_agent/CHANGELOG.md` for breaking changes

## Usage
```bash
# Import workspace context
cat ~/.matome/workspaces/tokio-docs/index.json
```
```

#### 3.5 Bundle コマンド (5日)
```bash
# トピックベースのコンテキストバンドル生成
matome bundle tokio-docs --topics "runtime,sync" --max-tokens 80000 > context.md

# 出力: 80,000トークン以下のマークダウンファイル
# 優先順位: _index.md → 重要度順ファイル
```

---

## 🔧 Phase 4: 統合・エコシステム (6-8週間)

### タスク

#### 4.1 Git 同期エクスポート (7日)
```bash
matome export --git --repo ~/docs/tokio-workspace
# → Git リポジトリ初期化・コミット・タグ付け
# → CI/CD エージェントが git pull で更新取得
```

#### 4.2 ローカル API サーバー (7日)
```rust
// /api/v1/tree - ツリー構造JSON
// /api/v1/changes - 変更一覧
// /api/v1/bundle - コンテキストバンドル
// /api/v1/export - ワークスペースzip
```

#### 4.3 DOM TOC 抽出 (10日)
```toml
[structure]
toc-selector = "nav.sidebar ul"
breadcrumb-selector = "nav.breadcrumb"
```

```rust
pub fn extract_toc_from_dom(html: &str, selector: &str) -> Vec<TocNode> {
    // scraper で nav 要素抽出
    // 再帰的に ul/li をツリー化
}
```

#### 4.4 サンプルワークスペース公開 (5日)
- tokio-docs ワークスペースサンプル
- kubernetes-docs ワークスペースサンプル
- 検証レポート公開

---

## 📈 マイルストーン

| マイルストーン | 達成時期 | 完了条件 |
|-------------|---------|---------|
| **M0: 基盤完成** | Week 2 | マイグレーション成功、tree_path 推論テスト通過 |
| **M1: Library 完成** | Week 5 | ツリーUI表示、全文検索動作確認 |
| **M2: Diff 完成** | Week 9 | `matome diff` 正常動作、アラート表示 |
| **M3: Agent MVP** | Week 14 | ワークスペース生成、Claude 連携テスト通過 |
| **M4: v2.0 リリース** | Week 20 | 全モード動作、テスト80%+、ドキュメント整備 |

---

## 🎯 次のアクション (Week 1)

### 即座に開始可能

1. **マイグレーションスクリプト作成** (2時間)
   ```bash
   sqlx migrate add add_tree_hierarchy
   ```

2. **infer_tree_path() 実装** (4時間)
   - ユニットテスト付きで実装
   - 境界ケース カバレッジ 90%+

3. **compute_content_hash() 実装** (2時間)
   - normalize ロジック定義
   - テストケース作成

4. **PR/Issue 作成** (1時間)
   - v2.0 Specification のフィードバック募集
   - GitHub Discussions で Agent Mode 需要調査

---

## 📋 リソース見積もり

| Phase | 開発期間 | 主要タスク数 | コード量見積もり |
|-------|---------|------------|----------------|
| Phase 0 | 2週間 | 6タスク | ~500行 |
| Phase 1 | 3週間 | 5タスク | ~800行 |
| Phase 2 | 4週間 | 5タスク | ~600行 |
| Phase 3 | 5週間 | 6タスク | ~1000行 |
| Phase 4 | 8週間 | 4タスク | ~700行 |
| **Total** | **22週間** | **26タスク** | **~3600行** |

---

## ⚠️ リスクと対策

| リスク | 確率 | 影響 | 対策 |
|-------|-----|-----|------|
| Tantivy スキーマ変更 | 中 | 中 | マイグレーションスクリプト準備 |
| エージェント互換性変化 | 高 | 低 | 出力フォーマットは標準JSON/MDに限定 |
| マイグレーションでデータ損失 | 低 | 高 | バックアップ取得・ロールバック手順整備 |
| トークン見積もり精度 | 中 | 中 | tiktoken 以外の簡易バックオフ実装 |

---

*This document is updated according to project progress.*
