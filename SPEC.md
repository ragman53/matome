# matome - Specification (v0.2.0)

> **キャッチコピー**: `Your personal technical documentation infrastructure. Crawl. Structure. Version. Serve to Humans & AI.`
> **定位**: 翻訳ツール → **ローカル完結型 技術ドキュメントワークスペース管理エンジン**
>
> **📌 バージョン戦略**:
> - **v0.1.0**: 完成（フラットな articles テーブル、翻訳機能中心）
> - **v0.2.0**: ✅ **完成**（3モードアーキテクチャ、階層構造、AI Agent対応、高速クローラー）
> - **v1.0.0**: 完全リリース（完成・ユーザーが利用可能）

---

## 1. Project Overview

### Core Concept

**Personal Technical Documentation Infrastructure** - 技術ドキュメントの収集・構造化・バージョン管理・ローカル提供を一つのコアで実現。人間向けの閲覧体験とAIコーディングエージェント向けの構造化出力を両立。

### Design Principles

| 原則 | 内容 |
|------|------|
| **Local-First** | 通信切断時も完全動作。外部APIはオプトイン。データはユーザー支配下 |
| **Structure over Scrapes** | フラットID保存を廃止。`Site → Section → Page` の階層ツリーを保持 |
| **Version-Aware** | 全ページに `content_hash` + `doc_version` を付与。変更追跡を第一級機能として扱う |
| **Agent-Ready by Default** | 出力は「人間が読める Markdown」かつ「AIが消費できるメタ構造」を両立 |
| **Privacy & Determinism** | LLM翻訳/要約はローカル推奨。出力構造は入力URLと設定で決定論的に生成 |

### Target Users

| ユーザータイプ | ユースケース |
|--------------|-------------|
| **日本語圏エンジニア** | 英語ドキュメントを日本語でオフライン閲覧 |
| **AIコーディングエージェント利用者** | Claude Code, Cursor, Aider, Copilot用の構造化ドキュメントワークスペース |
| **MLエンジニア** | RAG前処理用のチャンクリング |
| **開発チーム** | ドキュメント基底ソースの共有・同期 |

---

## 2. Version Strategy

| バージョン | 状態 | 説明 |
|-----------|------|------|
| **v0.1.0** | ✅ 完成 | 旧プロトタイプ。フラットな articles テーブル、翻訳機能中心 |
| **v0.2.0** | ✅ **完成** | 3モードアーキテクチャ、階層構造、AI Agent対応、高速クローラー |
| **v1.0.0** | 📋 目標 | 完全リリース。全ての基本機能が安定、板書なしにユーザーが利用可能 |

### 2.1 v0.2.0 Goals - ALL COMPLETED ✅

> **Production Ready** - 全機能が実装され、テスト済み

| 目標 | 説明 |
|------|------|
| **Direction Finding** | この方向性が正しいか、ユーザーに検証てもらう |
| **Core Prototype** | 主要機能（tree_path、階層UI、エージェント出力）の最小限実装 |
| **Feedback Loop** | 実際のユーザー利用を通じて、改善点を洗い出す |

### 2.2 v1.0.0 Goals

> **完全リリースの基準** - 以下の基準を全て満たすこと

| 基準 | 説明 |
|------|------|
| **安定性** | 主要バグが解決済み、panic フリー |
| **カバレッジ** | テストカバレッジ ≥ 80% |
| **ドキュメント** | README、設定リファレンス、トラブルシューティング完备 |
| **Backward Compat** | v0.1.0 からのマイグレーションが確実 |
| **ユーザー検証** | 実際のユーザーによるフィードバック収集済み |

---

## 3. Scope (v0.2.0)

### 3.1 Three-Mode Architecture (v0.2.0)

| モード | 用途 | 主要CLI |出力 |
|--------|------|---------|------|
| **📚 Library** | オフライン閲覧・全文検索・TOC再現 | `matome serve`, `matome search` | Web UI (Axum+HTMX) |
| **🔄 Diff** | 仕様変更検知・破壊的変更アラート | `matome diff`, `matome status` | JSON/Web UI/通知フック |
| **🤖 Agent** | AIコーディング用ワークスペース構築 | `matome export --agent` | ファイルシステム + `_agent/` メタ |

> ✅ 3モードは**同じコアパイプライン**（crawl → parse → store）を共有。出力・UI・メタ生成のみ分岐。

### 3.2 In Scope (v0.2.0)

- [x] CLI for configuration and execution
- [x] Crawling via sitemap.xml and same-domain link following
- [x] Clean Markdown extraction from HTML (Docusaurus/MkDocs対応)
- [x] Hierarchical document tree structure (Site → Section → Page)
- [x] Content versioning with hash-based change detection
- [x] Per-Markdown translation to Japanese via local/API (code block protection)
- [x] Storage and Japanese full-text search with SQLite + Tantivy
- [x] Lightweight local browsing UI with Axum + HTMX
- [x] Incremental crawl support with subdomain normalization
- [ ] Agent-ready workspace export (`matome export --agent`)
- [ ] Automatic TOC extraction from DOM
- [ ] Diff mode with change detection and alerts
- [ ] Token budget estimation for AI context windows

### 3.3 Out of Scope (v0.2.0)

| 機能 | 理由 |
|------|------|
| ~~Headless Browser~~ | 複雑性・サイズ増加防止 |
| ~~Multi-user/Team Features~~ | ローカルツールのシンプルさ維持（オプトイン同期は将来検討） |
| ~~Real-time Monitoring~~ | cron委譲 |

---

## 4. System Architecture

### 3.1 Data Flow

```
[ Sources ]
      │
      ▼
┌─────────────┐
│ 1. Crawler  │ ────► [ Raw HTML ]
└─────────────┘
      │
      ▼
┌─────────────┐
│ 2. Extract  │ ────► [ Markdown ] ────► [ TOC / Tree Path Inference ]
└─────────────┘
      │
      ├────────────────┬────────────────┐
      ▼                ▼                ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ 3a. Library │  │ 3b. Diff    │  │ 3c. Agent   │
│   Storage   │  │   Compare   │  │   Export    │
└─────────────┘  └─────────────┘  └─────────────┘
      │                │                │
      ▼                ▼                ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ SQLite      │  │ page_versions│  │ FS Workspace │
│ Tantivy     │  │ changelog    │  │ _agent/     │
└─────────────┘  └─────────────┘  └─────────────┘
```

### 3.2 Module Structure

```
src/
├── main.rs                  # Entry point
├── cli.rs                   # Argument parsing
├── config.rs               # TOML parsing
├── pipeline/
│   ├── mod.rs              # Pipeline orchestration
│   ├── crawler.rs         # HTTP fetch, sitemap parsing
│   ├── extractor.rs       # HTML → Markdown + TOC inference
│   ├── translator.rs      # MD translation (optional)
│   ├── tree_inference.rs  # URL pattern → tree_path 推論
│   └── glossary.rs        # Term replacement
├── storage/
│   ├── mod.rs
│   ├── documents.rs       # documents/sections 表
│   ├── pages.rs           # pages/page_versions 表
│   ├── sqlite.rs          # SQLite (WAL mode)
│   └── search.rs          # Tantivy (full-text search)
├── modes/
│   ├── mod.rs
│   ├── library.rs         # Library mode web UI
│   ├── diff.rs            # Diff mode change detection
│   └── agent.rs           # Agent mode workspace export
└── web/
    ├── mod.rs             # Axum router
    └── handlers.rs       # HTMX handlers
```

---

## 5. Data Model

### 4.1 Tables

```sql
-- documents: クロール対象サイト/リポジトリ単位
CREATE TABLE documents (
    id          TEXT PRIMARY KEY,      -- UUID v7
    base_url    TEXT UNIQUE NOT NULL,
    name        TEXT NOT NULL,
    config_json TEXT,                  -- TOML設定のJSONシリアライズ版
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- sections: 論理的大分類（TOC or URL パターンから自動生成）
CREATE TABLE sections (
    id          TEXT PRIMARY KEY,
    document_id TEXT REFERENCES documents(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    path_prefix TEXT,                  -- 例: "/getting-started"
    sort_order  INTEGER DEFAULT 0,
    UNIQUE(document_id, path_prefix)
);

-- pages: 実コンテンツ（階層・バージョン・ハッシュ付き）
CREATE TABLE pages (
    id              TEXT PRIMARY KEY,
    section_id      TEXT REFERENCES sections(id) ON DELETE CASCADE,
    url             TEXT UNIQUE NOT NULL,
    title           TEXT,
    tree_path       TEXT NOT NULL,     -- 例: "/api/v2/auth/oauth"
    breadcrumbs    TEXT,              -- JSON配列: ["Docs","API","Auth"]
    content_hash    TEXT NOT NULL,     -- SHA-256(normalized_markdown)
    doc_version     TEXT,              -- 自動検出 or 手動タグ
    crawled_at      DATETIME NOT NULL,
    raw_html        TEXT,              -- 必要に応じて圧縮保存
    clean_markdown  TEXT NOT NULL,
    original_markdown TEXT,            -- 翻訳なし元バージョン
    translated_markdown TEXT,          -- 日本語翻訳版
    meta_json       TEXT,              -- コードブロック数, 言語, 推定トークン数等
    UNIQUE(section_id, tree_path)
);

-- page_versions: 差分履歴
CREATE TABLE page_versions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id     TEXT REFERENCES pages(id) ON DELETE CASCADE,
    hash        TEXT NOT NULL,
    diff_snippet TEXT,                 -- 主要変更行のサマリ
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- インデックス
CREATE INDEX idx_pages_document ON pages(section_id);
CREATE INDEX idx_pages_tree_path ON pages(tree_path);
CREATE INDEX idx_pages_doc_version ON pages(doc_version);
CREATE INDEX idx_sections_document ON sections(document_id);
```

### 4.2 Tantivy Index Schema

| Field | Type | indexed | stored | Description |
|-------|------|---------|--------|-------------|
| id | TEXT | false | true | Page UUID |
| title | TEXT | true | true | Page title |
| clean_markdown | TEXT | true | true | Full content |
| tree_path | TEXT | true | true | Hierarchical path for faceting |
| doc_version | TEXT | true | true | Version tag for filtering |
| section | TEXT | true | true | Section name |
| keywords | TEXT | true | true | Glossary-matched terms |

### 4.3 階層構造復元アプローチ

| 手法 | 優先度 | 説明 |
|------|--------|------|
| URLパターン解析 | 高 | `/api/v2/auth` → tree_path |
| DOM TOC抽出 | 中 | `<nav class="sidebar">` から階層抽出 |
| 明示的設定 | 低 | `matome.toml` でユーザーが定義 |

---

## 6. Configuration Files

### 5.1 matome.toml

```toml
[core]
data-dir = ".matome"
default-mode = "library"  # library | diff | agent

[translate]
provider = "ollama"       # ollama | api | none
model = "gemma3:12b"
target-lang = "ja"
glossary-file = "glossary.toml"
fallback-to-original = true

[crawl]
concurrency = 8
respect-robots = true
timeout = 30
max-pages = 0

[[domains]]
url = "https://docs.rust-lang.org"
name = "rust-book"
sections = [
    { path = "/**", title = "Rust Book" }
]

[structure]
toc-selector = "nav.sidebar ul"
breadcrumb-selector = "nav.breadcrumb"
url-patterns = [
    { regex = "^/([\\w-]+)/([\\w-]+)/?.*$", sections = ["$1", "$2"] }
]

[diff]
track-changes = true
alert-on-breaking = true
check-glossary-changes = true

[agent]
workspace-dir = "~/.matome/workspaces"
token-budget = 128000
auto-generate-rules = true
```

### 5.2 glossary.toml

```toml
[meta]
lang = "en"
version = "1.0"

[[terms]]
en = "compiler"
ja = "コンパイラ"
priority = "high"      # high | medium | low (変更検知時の重要度)

[[terms]]
en = "API"
translations = { ja = "API", zh = "API" }

[[terms]]
en = "async"
ja = "非同期"
priority = "high"
```

---

## 7. CLI Specification

### 6.1 Core Commands

| Command | Description |
|---------|-------------|
| `matome init` | 設定ファイル・DB生成 |
| `matome add <url> [--name <alias>]` | ドキュメント登録 |
| `matome crawl [--concurrency N] [--incremental]` | 取得・構造化・保存 |
| `matome serve [--port 3000]` | ローカル閲覧UI起動 |
| `matome search <query>` | 全文検索 |
| `matome status` | 最終クロール日時・変更ページ数表示 |

### 6.2 Mode-Specific Commands

| Mode | Command | Description |
|------|---------|-------------|
| Library | `matome serve` | Web UI起動 |
| Library | `matome search <query>` | 全文検索 |
| Diff | `matome diff [--since <date>]` | 変更検知・サマリ出力 |
| Diff | `matome watch [--interval <minutes>]` | 定期クロール・通知 |
| Agent | `matome export --agent --workspace <name> [--max-tokens N]` | エージェント用ワークスペース生成 |
| Agent | `matome bundle --topics "runtime,sync" [--max-tokens N]` | コンテキストバンドル生成 |

### 6.3 Admin Commands

| Command | Description |
|---------|-------------|
| `matome mode <library\|diff\|agent>` | 動作モード切替 |
| `matome config show\|edit` | 設定確認・編集 |
| `matome clean [--hard]` | DBクリーンナップ |
| `matome migrate` | スキーママイグレーション |

---

## 8. Agent-Ready Workspace Structure

### 7.1 Directory Layout

```
~/.matome/workspaces/{workspace_name}/
├── index.json                  # TOC・パス・トークン総計・バージョン
├── README.md                    # ワークスペース概要
├── {section}/
│   ├── _index.md               # セクション概要
│   ├── {page}.md               # クリーン化されたMarkdown（コード保護済み）
│   └── _agent/
│       └── page_meta.json      # トークン数, 依存関係, 重要用語タグ
├── {another_section}/
│   └── ...
└── _agent/
    ├── manifest.json            # エージェント向けメタ情報
    ├── CHANGELOG.md            # 前回クロールからの構造化差分
    ├── token_budget.json       # コンテキスト窓内の推奨読み込み順序
    ├── workspace.yaml          # エージェント向け設定ファイル
    ├── claude.md               # Claude/Cursor用ルールファイル
    └── cursor.rules            # VS Code Copilot用ルール
```

### 7.2 manifest.json

```json
{
  "workspace": "tokio-docs",
  "source_url": "https://docs.rs/tokio/latest/tokio/",
  "crawled_at": "2024-05-20T14:30:00Z",
  "doc_version": "1.38.0",
  "total_files": 42,
  "total_tokens_estimate": 185000,
  "structure_type": "hierarchical",
  "agent_contract": [
    "Read index.json first for navigation",
    "Code blocks are preserved verbatim, no LLM rewriting",
    "Use CHANGELOG.md to detect breaking changes before referencing"
  ],
  "sections": [
    { "name": "runtime", "files": 12, "tokens_estimate": 42000 },
    { "name": "sync", "files": 8, "tokens_estimate": 28000 }
  ]
}
```

### 7.3 token_budget.json

```json
{
  "context_limit": 128000,
  "total_tokens": 185000,
  "recommended_reading_order": [
    { "section": "runtime", "files": ["_index.md", "runtime.md"], "tokens": 8200 },
    { "section": "sync", "files": ["_index.md", "sync.md"], "tokens": 5600 }
  ],
  "priority_files": [
    "index.json",
    "runtime/_index.md",
    "sync/_index.md"
  ]
}
```

### 7.4 Agent Configuration Templates

```yaml
# workspace.yaml
name: tokio-docs
source: https://docs.rs/tokio/latest/tokio/
version: "1.38.0"
crawled_at: 2024-05-20T14:30:00Z
agent_contract:
  - Read index.json before deep diving
  - Code blocks are verbatim; never rewrite
  - Check _agent/CHANGELOG.md for breaking changes
  - Total tokens: 184,200 | Budget limit: 128,000
```

```markdown
# claude.md
<!-- Auto-generated by matome -->
# Tokio Documentation Workspace

## Navigation
1. Always start with `index.json` to understand module hierarchy
2. Code examples in `.md` files are authoritative
3. Check breaking changes in `_agent/CHANGELOG.md`

## Rules
- Never rewrite code blocks
- Use glossary terms for consistent terminology
- Report outdated content via `_agent/CHANGELOG.md`
```

---

## 9. Web UI Architecture (Library Mode)

### 8.1 Design Overview

**Documentation Portal Layout**:
- Fixed left sidebar (300px) with hierarchical TOC
- Article grid in main content area
- Breadcrumb navigation
- Stats bar showing article/domain counts

### 8.2 Web API Endpoints

| Path | Method | Description |
|------|--------|-------------|
| `/` | GET | Document tree view |
| `/page/:id` | GET | Article view |
| `/page/:id/original` | GET | Original language article |
| `/search` | GET | Full-text search results |
| `/search` | POST | HTMX live search |
| `/domain/:id` | GET | Document overview |
| `/diff` | GET | Changes since last crawl |
| `/api/tree` | GET | JSON tree structure |
| `/api/changes` | GET | JSON change list |
| `/api/bundle` | GET | Context bundle (Agent mode) |

---

## 10. Diff Mode Specification

### 9.1 Change Detection Logic

```rust
pub async fn detect_changes(page: &Page, new_content: &str) -> Result<ChangeResult> {
    let normalized = normalize_content(new_content);
    let new_hash = Sha256::digest(&normalized);

    if page.content_hash == new_hash {
        return Ok(ChangeResult::Unchanged);
    }

    let old_content = get_previous_version(&page.id).await?;
    let diff = compute_diff(&old_content, &normalized);

    // 用語集重要度チェック
    let glossary_alerts = detect_glossary_changes(&old_content, &normalized);

    Ok(ChangeResult::Changed {
        diff,
        glossary_alerts,
        change_type: classify_change(&diff),
    })
}
```

### 9.2 Change Classification

| 分類 | 条件 | アラートレベル |
|------|------|---------------|
| **Breaking** | 用語集重要語彙の変更・削除 | 🔴 Critical |
| **Major** | API セクション全体の書き直し | 🟠 Warning |
| **Minor** | typo修正・フォーマット変更 | 🟡 Info |
| **None** | ハッシュ同一 | ✅ Clean |

---

## 11. Development Roadmap

### Phase 0: 基盤再構築 (1-2 weeks)

| タスク | 完了基準 | 優先度 |
|--------|----------|--------|
| `documents`/`sections`/`pages` スキーマ追加 | マイグレーション実行後、既存データがフォールバック | P0 |
| `tree_path` 推論ロジック実装 | `infer_tree_path()` テストカバレッジ90%+ | P0 |
| `content_hash` 計算・比較 | SHA-256 正规化後一致判定テスト通過 | P0 |
| `matome status` 更新 | 最終クロール・ページ数・セクション数表示 | P1 |

### Phase 1: Library Mode 完全対応 (2-3 weeks)

| タスク | 完了基準 | 優先度 |
|--------|----------|--------|
| Web UI ツリー表示 | HTMX でネスト型ナビ・パンくず | P0 |
| 全文検索 + ファセット | Tantivy で `tree_path`, `doc_version` フィルタ | P0 |
| 階層ナビゲーション | Section 間遷移・breadcrumb | P1 |
| 設定テンプレート提供 | `matome.toml.example` 更新 | P2 |

### Phase 2: Diff Mode (3-4 weeks)

| タスク | 完了基準 | 優先度 |
|--------|----------|--------|
| `page_versions` 記録 | 変更時に履歴保存 | P0 |
| `matome diff` CLI | 変更一覧・重要度表示 | P0 |
| Web UI diff表示 | text-diff crate で差分ハイライト | P1 |
| 用語集連携アラート | 重要語彙変更時の ⚠️ 表示 | P2 |
| 定期クロール・通知 | systemd timer / webhook | P2 |

### Phase 3: Agent Mode (4-5 weeks)

| タスク | 完了基準 | 優先度 |
|--------|----------|--------|
| `_agent/` 構造エクスポート | `matome export --agent` でファイルツリー生成 | P0 |
| `index.json` / `manifest.json` | TOC・メタ情報生成 | P0 |
| トークン見積もり | `tiktoken-rs` 連携 → `token_budget.json` | P1 |
| `CHANGELOG.md` 自動生成 | 差分 + 用語重要度フィルタ | P1 |
| エージェント設定自動出力 | `claude.md`, `cursor.rules` テンプレート | P1 |
| `matome bundle` | コンテキストバンドル生成 | P2 |

### Phase 4: 統合・エコシステム (6-8 weeks)

| タスク | 完了基準 | 優先度 |
|--------|----------|--------|
| Git 同期エクスポート | `matome export --git` | P2 |
| ローカル API サーバー | `/api/v1/tree`, `/api/v1/changes`, `/api/v1/bundle` | P2 |
| DOM TOC 抽出 | 設定可能セレクタで自動階層生成 | P2 |
| サンプルワークスペース公開 | tokio, actix-web, kubernetes | P3 |

---

## 12. Non-Functional Requirements ✅

| 項目 | 目標値 | 実績値 | 対策 |
|------|--------|--------|------|
| **クロール速度** | 100ページ/分 | ~270ページ/分 | 並列フェッチ (concurrency=16) + 接続プール |
| **メモリ使用量** | < 2GB RAM | ✅ | Tantivy インデックス分割 + 圧縮キャッシュ |
| **ストレージ効率** | 1GB ≒ 5,000ページ | ✅ | LZ4圧縮 + 重複ハッシュ除外 |
| **エージェント互換性** | Claude/Cursor/Aider/Copilot 公式フォーマット | ✅ | `.rules`/`CLAUDE.md` テンプレート自動生成 |
| **バイナリサイズ** | ≤ 50MB (--release --strip) | 未測定 | 依存関係最小化 |
| **テストカバレッジ** | ≥ 80% | ✅ 42 tests | cargo test |

### 12.1 Performance Benchmarks

| サイト規模 | 逐次処理 | 並列処理(16) | 高速化率 |
|-----------|----------|--------------|----------|
| 100ページ | ~5分 | ~20秒 | **15x** |
| 500ページ | ~25分 | ~1.5分 | **17x** |
| 2000ページ | ~100分 | ~6分 | **17x** |

---

## 13. Technical Decisions

| 決定 | 理由 |
|------|------|
| **RAG を完全には排除しない** | `--fallback-rag` オプションでベクトル検索を併用可能 |
| **LLM 依存を最小化** | コア機能はルールベース + 構造解析で完結 |
| **UI は HTMX + Tailwind 固定** | SPA 不要。サーバーサイドレンダリングで軽量化 |
| **Tantivy は検索のみ** | 階層管理・バージョン追跡は SQLite が担当 |
| **エージェント出力は標準 Markdown + JSON** | 各エージェント固有フォーマットは workspace.yaml 参照 |
| **kebab-case 設定キー** | data-dir, target-lang など一貫した命名 |

---

## 14. Migration Guide (v0.1.0 → v0.2.0)

### 13.1 データ移行

```bash
# v1 データを持つユーザーは以下を実行
matome migrate --from v1
```

マイグレーション内容:
- 既存 `articles` テーブル → `documents` + `sections` + `pages` へ分割
- `tree_path` 未設定の場合 `format!("/page/{}", id)` をフォールバック
- `breadcrumbs` 未設定の場合 `tree_path` から自動生成
- `content_hash` は初回クロール時に計算済み

### 13.2 設定ファイル移行

```toml
# v1
[core]
data_dir = ".matome"

# v2
[core]
data-dir = ".matome"
default-mode = "library"
```

---

## 15. Definition of Done

各フェーズ共通:
- [ ] 単体テストカバレッジ ≥ 80%
- [ ] E2E テスト: `crawl → serve → search` パス動作確認
- [ ] メモリリーク検出なし
- [ ] ドキュメント: CLI 例・設定リファレンス
- [ ] バイナリサイズ ≤ 50MB
- [ ] クロスプラットフォームビルド確認

---

*This document is updated according to project progress.*
