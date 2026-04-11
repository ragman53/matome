# matome

**Your personal technical documentation infrastructure.** (v0.2.1 - Production Ready)

> ✅ **Production Ready** - v0.2.1 with improved HTML extraction and output quality.

matome automatically crawls documentation from specified websites, organizes them into hierarchical structures, tracks version changes, and provides a local web portal for browsing—all in one unified experience.

**New in v0.2.1**: 
- 🛠️ **Improved table extraction** - Tables with nested elements (lists, bold text) render correctly
- 🎨 **Code block language detection** - Automatic language tagging from HTML classes
- ⚡ **Enhanced output quality** - Clean, structured Markdown preservation

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

### 1. Initialize

```bash
# Initialize configuration
matome init
```

### 2. Add Documentation Sources

Edit `matome.toml` to add your documentation sources:

```toml
[[domain]]
url = "https://docs.python.org/3/"
include = ["/**"]

[[domain]]
url = "https://developer.mozilla.org/en-US/docs/"
include = ["/**"]
```

### 3. Crawl Documentation

```bash
# Parallel crawling (recommended)
matome crawl --concurrency 16

# Incremental update (only new/changed pages)
matome crawl --incremental
```

### 4. Start the Web Server

```bash
matome serve
```

Then open [http://127.0.0.1:8080](http://127.0.0.1:8080) in your browser.

## 🎯 Choose Your Workflow

### 🔍 Library Mode (Default)

> "Save your favorite docs locally, search instantly."

```bash
# Browse in browser
matome serve  # Open http://localhost:8080

# Full-text search from CLI
matome search "async runtime"
```

### 🔄 Diff Mode

> "Get notified when docs change. Focus on what matters."

```bash
# Switch to diff mode
matome mode diff

# Check what changed since last crawl
matome diff
```

### 🤖 Agent Mode

> "Export clean, chunked data for your AI coding assistant."

```bash
# Switch to agent mode
matome mode agent

# Export workspace
matome export --agent --workspace tokio-docs --max-tokens 80000
```

## ⚙️ Configuration

### matome.toml

```toml
[core]
data-dir = ".matome"
default-mode = "library"  # library | diff | agent

[translate]
provider = "none"              # none | ollama | deepl
model = "gemma3:4b"           # Ollama model name
target-lang = "ja"            # Target language code
glossary-file = "glossary.toml"

[crawl]
concurrency = 16              # Parallel crawling threads
respect-robots = true         # Follow robots.txt
timeout = 60                  # Request timeout (seconds)
max-pages = 0                 # 0 = unlimited

# Documentation sources
[[domain]]
url = "https://docs.python.org/3/"
include = ["/**"]

[[domain]]
url = "https://developer.mozilla.org/en-US/docs/"
include = ["/**"]
```

### Glossary (glossary.toml)

```toml
[[terms]]
en = "compiler"
ja = "コンパイラ"
priority = "high"      # Changes to priority terms trigger alerts

[[terms]]
en = "async"
ja = "非同期"
```

## 📦 Installation

### From Source

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

### Mode Commands

```bash
matome mode <library|diff|agent>    # Switch operation mode
matome diff                          # Show changes (Diff Mode)
matome export --agent --workspace <name>  # Export for AI agents
```

## 🌐 Web Interface

### Library Mode UI

- **Sidebar** - Hierarchical document tree navigation
- **Breadcrumb** - Current location (Section → Page)
- **⌘K** - Quick search modal
- **Domain filtering** - Click to filter by domain

### Reading View

- Toggle between **翻訳 (Translated)** and **原文 (Original)**
- Clean typography optimized for documentation
- Code blocks with syntax highlighting
- Tables rendered properly with nested elements preserved

## 🗂️ Data Structure

```
.matome/
├── matome.db              # SQLite database (WAL mode)
│                         # Tables: articles (v0.1.0 compatibility)
│                         #           documents, sections, pages (v0.2.0)
│
└── search_index/          # Tantivy full-text search index
```

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

## 📈 Performance

### Benchmark Results

| Site Size | Sequential | Parallel (16) | Speedup |
|-----------|------------|---------------|---------|
| 100 pages | ~5 min | ~20 sec | **15x** |
| 500 pages | ~25 min | ~1.5 min | **17x** |
| 2000 pages | ~100 min | ~6 min | **17x** |

### Recommended Settings

| Site | concurrency | Reason |
|------|-------------|--------|
| docs.python.org | 2-4 | Strict rate limiting |
| docs.rs | 4-8 | Moderate rate limiting |
| example.com | 16 | Local testing |

## 🔧 Output Quality (v0.2.1)

### HTML Extraction Improvements

| Feature | Status | Description |
|---------|--------|-------------|
| Table rendering | ✅ Fixed | Nested lists, bold text in cells preserved |
| Code blocks | ✅ Fixed | Language tags auto-detected from class |
| Headings | ✅ | Proper hierarchy with # syntax |
| Links | ✅ | Markdown link format preserved |
| Images | ✅ | Alt text and src preserved |
| Lists | ✅ | Ordered and unordered lists |

### Example Output

**Input HTML with nested elements:**
```html
<table>
  <tr><th>Name</th><th>Options</th></tr>
  <tr>
    <td>Item</td>
    <td><ul><li>A</li><li>B</li></ul></td>
  </tr>
</table>
```

**Output Markdown:**
```markdown
| Name | Options |
| --- | --- |
| Item | - A; - B |
```

## 📄 License

MIT License

---

*matome* (まとめ) - Japanese for "summary" or "collection"
