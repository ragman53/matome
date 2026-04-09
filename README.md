# matome

**Your personal technical documentation infrastructure.** (v0.2.0)

> ✅ **Production Ready** - v0.2.0 feature complete with high-performance parallel crawling.

matome automatically crawls documentation from specified websites, organizes them into hierarchical structures, tracks version changes, and provides a local web portal for browsing—all in one unified experience.

**New in v0.2.0**: 
- ⚡ **High-performance parallel crawling** (up to 17x faster)
- 🤖 Agent-Ready workspace export for AI coding assistants
- 🔄 Hierarchical document structure with tree navigation

## ✨ Features

### 🎯 Three-Mode Architecture

| Mode | Description | Use Case |
|------|-------------|----------|
| **📚 Library** | Local web portal with hierarchical navigation | Browse docs offline |
| **🔄 Diff** | Automatic change detection & alerts | Track API breaking changes |
| **🤖 Agent** | Structured workspace export for AI agents | Context-aware code generation |

### Core Features

- ⚡ **High-performance crawling** - Parallel HTTP fetching with connection pooling (17x speedup)
- 🌐 **Multi-domain crawling** - Collect docs from multiple sources simultaneously
- 🔄 **Hierarchical structure** - Preserve document tree (Site → Section → Page)
- 📊 **Version tracking** - Content hash-based change detection
- 🌐 **Automatic translation** - Translate to Japanese via Ollama or DeepL (optional)
- 📚 **Local web portal** - Browse in a clean, sidebar-based interface
- 🔍 **Full-text search** - Search across all documentation (Tantivy)
- 🤖 **Agent-ready export** - Workspace format for AI coding assistants
- 📖 **Glossary support** - Maintain consistent terminology
- ⚡ **Incremental updates** - Only crawl new or changed pages

## 🚀 Quick Start

### Library Mode (Browse Locally)

```bash
# 1. Initialize configuration
matome init

# 2. Add documentation sources
matome add https://docs.python.org/
matome add https://developer.mozilla.org/

# 3. Crawl documentation (parallel - up to 17x faster!)
matome crawl --concurrency 16

# 4. Start the web server
matome serve
```

Then open [http://127.0.0.1:8080](http://127.0.0.1:8080) in your browser.

### Agent Mode (AI Coding Assistants)

```bash
# Export workspace for AI agent
matome export --agent --workspace tokio-docs

# AI agent can now read:
# ~/.matome/workspaces/tokio-docs/
# ├── index.json              # Table of contents
# ├── runtime/
# │   └── runtime.md          # Clean Markdown
# └── _agent/
#     ├── manifest.json       # Agent contract
#     ├── CHANGELOG.md        # Recent changes
#     └── token_budget.json  # Context optimization
```

## 📦 Installation

### From Source (Recommended)

```bash
cargo install --git https://github.com/ragman53/matome.git
```

### Build from Source

```bash
git clone https://github.com/ragman53/matome.git
cd matome
cargo build --release
./target/release/matome --version
```

### Prerequisites

- **Rust** (for building from source)
- **Ollama** or **DeepL API key** (for translation, optional)
- **SQLite** (bundled with rusqlite, no separate installation needed)

## 🎯 Choose Your Workflow

matome adapts to how you work with technical documentation:

### 🔍 Library Mode (Default)

> "Save your favorite docs locally, search instantly."

```bash
matome add https://docs.example.com
matome serve  # Open http://localhost:8080
```

### 🔄 Diff Mode

> "Get notified when docs change. Focus on what matters."

```bash
matome mode diff
matome add --track https://lib.io/docs  # Enable version tracking
matome status  # See what changed since last crawl
```

### 🤖 Agent Mode

> "Export clean, chunked data for your AI coding assistant."

```bash
matome mode agent
matome export --agent --workspace tokio-docs --max-tokens 80000
```

## ⚙️ Configuration

### Initialize

```bash
matome init
```

This creates:
- `matome.toml` - Main configuration file
- `glossary.example.toml` - Terminology glossary template

### matome.toml

```toml
[core]
data-dir = ".matome"
default-mode = "library"  # library | diff | agent

[translate]
provider = "ollama"           # or "deepl", "none"
model = "gemma3:4b"           # Ollama model name
target-lang = "ja"            # Target language code
glossary-file = "glossary.toml"

[crawl]
concurrency = 16              # Parallel crawling threads (adjust based on site)
respect-robots = true         # Follow robots.txt
timeout = 60                 # Request timeout (seconds)
max-pages = 0                 # 0 = unlimited, N = max pages

[diff]
track-changes = true
alert-on-breaking = true

[agent]
workspace-dir = "~/.matome/workspaces"
token-budget = 128000
auto-generate-rules = true

# Documentation sources
[[domains]]
url = "https://docs.python.org/"
name = "python-docs"
include = ["/**"]

[[domains]]
url = "https://developer.mozilla.org/"
name = "mdn"
include = ["/**"]
```

### Glossary

Create `glossary.toml` to maintain consistent terminology:

```toml
# Simple format
[[terms]]
en = "compiler"
ja = "コンパイラ"
priority = "high"      # Changes to priority terms trigger alerts

[[terms]]
en = "runtime"
ja = "ランタイム"

# Multi-language format
[[terms]]
en = "API"
translations = { ja = "API", zh = "API", ko = "API" }
```

## 📖 Commands

### Core Commands

| Command | Description |
|---------|-------------|
| `matome init` | Generate configuration templates |
| `matome add <url>` | Add a domain to configuration |
| `matome crawl` | Crawl and process documentation |
| `matome serve` | Start the local web server |
| `matome search <query>` | Full-text search |
| `matome status` | Display statistics |
| `matome clean` | Manage and clean the database |

### Mode-Specific Commands

#### Library Mode
```bash
matome serve                        # Start web UI
matome search "async runtime"       # Full-text search
```

#### Diff Mode
```bash
matome mode diff                    # Switch to diff mode
matome diff --since 2024-01-01      # Show changes since date
matome status                       # Show change summary
```

#### Agent Mode
```bash
matome mode agent                   # Switch to agent mode
matome export --agent --workspace tokio-docs
matome bundle --topics "runtime,sync" --max-tokens 80000
```

### Admin Commands

```bash
matome mode <library|diff|agent>    # Switch operation mode
matome config show                  # Show current config
matome config edit                  # Edit configuration
matome migrate                      # Run database migrations
```

## 🤖 AI Agent Integration

### Supported Agents

| Agent | Integration Method |
|-------|-------------------|
| **Claude Code** | `~/.matome/workspaces/` in CLAUDE.md |
| **Cursor** | `.cursorrules` auto-generation |
| **Aider** | `matome bundle` for context injection |
| **VS Code Copilot** | `.copilot.rules` templates |

### Workspace Structure

```
~/.matome/workspaces/{workspace_name}/
├── index.json                  # TOC, paths, version, token estimate
├── runtime/
│   ├── _index.md              # Section overview
│   ├── runtime.md             # Clean Markdown
│   └── _agent/
│       └── page_meta.json     # Token count, dependencies
└── _agent/
    ├── manifest.json           # Agent contract
    ├── CHANGELOG.md           # Changes since last crawl
    ├── token_budget.json      # Optimized reading order
    ├── workspace.yaml         # Configuration
    ├── claude.md              # Claude/Cursor rules
    └── cursor.rules           # VS Code Copilot rules
```

### Agent Contract Example

```json
{
  "workspace": "tokio-docs",
  "source_url": "https://docs.rs/tokio/latest/tokio/",
  "doc_version": "1.38.0",
  "total_tokens_estimate": 185000,
  "agent_contract": [
    "Read index.json first for navigation",
    "Code blocks are preserved verbatim; never rewrite",
    "Check _agent/CHANGELOG.md for breaking changes"
  ]
}
```

## 🌐 Web Interface

### Library Mode UI

- **Sidebar** - Hierarchical document tree navigation
- **Breadcrumb** - Current location (Section → Page)
- **⌘K** - Quick search modal
- **Domain/Section filtering** - Click to filter

### Reading View

- Toggle between **翻訳 (Translated)** and **原文 (Original)**
- Clean typography optimized for documentation
- Code blocks with syntax highlighting
- Links to original articles

### Diff Mode UI

- **Change Summary** - Breaking/Major/Minor classification
- **Glossary Alerts** - ⚠️ for priority term changes
- **Diff View** - Side-by-side or inline comparison

## 🔧 Translation Providers

### Ollama (Local, Recommended)

```toml
[translate]
provider = "ollama"
model = "gemma3:4b"  # or "llama3", "mistral", etc.
target-lang = "ja"
```

Install Ollama: https://ollama.ai/

```bash
# Pull a model
ollama pull gemma3:4b

# Start the server
ollama serve
```

### DeepL (Cloud API)

```toml
[translate]
provider = "deepl"
api_key = "your-api-key"
target-lang = "JA"
```

Get API key: https://www.deepl.com/pro-api

### No Translation

```toml
[translate]
provider = "none"
```

## 🗂️ Data Structure

```
.matome/
├── matome.db              # SQLite database (WAL mode)
│                         # Tables: documents, sections, pages, page_versions
│
├── search_index/          # Tantivy full-text search index
│
└── .gitkeep             # Preserves directory structure

~/.matome/workspaces/     # Agent mode exports
└── {workspace_name}/
    ├── index.json
    ├── {sections}/
    └── _agent/
        └── manifest.json
```

## 📋 Documentation Sites Supported

matome is optimized for these documentation formats:

- **Docusaurus** - GitHub Docs, React Native, Meta, etc.
- **MkDocs** - Python documentation ecosystem
- **Standard HTML** - Most documentation sites

Code blocks with nested elements (syntax highlighting `<span>` tags) are properly extracted.

## 🛠️ Troubleshooting

### "Failed to acquire index lock"

The search index is locked by another process:

```bash
pkill matome
# Or delete the lock file:
rm .matome/search_index/write.lock
```

### Translation failing

1. Check Ollama is running: `ollama list`
2. Pull the model if missing: `ollama pull gemma3:4b`
3. Test with translation disabled: `provider = "none"`

### Empty search results

Run a full crawl first, then restart the server:

```bash
matome crawl
matome serve
```

### Slow crawling

Reduce concurrency if rate-limited:

```bash
matome crawl --concurrency 2
```

## 📋 Examples

### Full Library Workflow

```bash
# 1. Initialize
matome init

# 2. Add domains
matome add https://docs.rust-lang.org/
matome add https://developer.mozilla.org/

# 3. Create glossary
cat > glossary.toml << 'EOF'
[[terms]]
en = "ownership"
ja = "所有権"
[[terms]]
en = "borrowing"
ja = "借用"
EOF

# 4. Crawl (with 4 parallel threads)
matome crawl --concurrency 4

# 5. Start server
matome serve
```

### Change Tracking Workflow

```bash
# Enable version tracking
matome mode diff
matome add https://docs.rust-lang.org/

# First crawl (establish baseline)
matome crawl

# Later - check for changes
matome diff --since 2024-01-15
```

### Agent Export Workflow

```bash
# Export workspace for AI agent
matome mode agent
matome add https://docs.rs/tokio/latest/tokio/
matome crawl

# Export with token budget
matome export --agent --workspace tokio-docs --max-tokens 80000

# In Claude Code, add to CLAUDE.md:
# "Always read ~/.matome/workspaces/tokio-docs/index.json before Tokio questions."
```

### Incremental Update

```bash
# Run daily to get new documentation
matome crawl --incremental
```

## 📄 License

MIT License

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum) for the web server
- Full-text search powered by [Tantivy](https://github.com/quickwit-oss/tantivy)
- Translation via [Ollama](https://ollama.ai/) or [DeepL](https://www.deepl.com/)
- HTML extraction with [scraper](https://github.com/programming钦thatcher/scraper)

---

*matome* (まとめ) - Japanese for "summary" or "collection"
