# matome

A Rust CLI tool for collecting, translating, and browsing documentation locally.

matome automatically crawls documentation from specified websites, translates content to Japanese (or your target language), and provides a beautiful local web portal for reading—all in one unified experience.

## ✨ Features

- 🌐 **Multi-domain crawling** - Collect docs from multiple sources simultaneously
- 🔄 **Automatic translation** - Translate English docs to Japanese via Ollama or DeepL
- 📚 **Local web portal** - Browse your collected docs in a clean, sidebar-based interface
- 🔍 **Full-text search** - Search across all your collected documentation (Tantivy)
- 📖 **Glossary support** - Maintain consistent terminology across translations
- ⚡ **Incremental updates** - Only crawl new or changed pages
- 🧹 **Database management** - Clean incomplete or unwanted articles
- 📦 **Docusaurus/MkDocs対応** - Code blocks with nested elements properly extracted

## 🚀 Quick Start

```bash
# 1. Initialize configuration
matome init

# 2. Edit matome.toml and add your domains
matome add https://docs.python.org/
matome add https://developer.mozilla.org/

# 3. Crawl and translate
matome crawl

# 4. Start the web server
matome serve
```

Then open [http://127.0.0.1:8080](http://127.0.0.1:8080) in your browser.

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
data-dir = ".matome"      # Where database and search index are stored

# Add your documentation sources
[[domain]]
url = "https://docs.python.org/"
include = ["/**"]

[[domain]]
url = "https://developer.mozilla.org/"
include = ["/**"]

# Translation settings
[translate]
provider = "ollama"           # or "deepl", "none"
model = "gemma3:4b"           # Ollama model name
target-lang = "ja"            # Target language code
glossary-file = "glossary.toml"

# Crawling settings
[crawl]
concurrency = 8               # Parallel crawling threads
respect-robots = true         # Follow robots.txt
timeout = 30                 # Request timeout (seconds)
max-pages = 0                 # 0 = unlimited, N = max pages
# treat-subdomains-same = true  # Optional: docs.example.com = example.com
```

### Glossary

Create `glossary.toml` to maintain consistent terminology:

```toml
# Simple format
[[terms]]
en = "compiler"
ja = "コンパイラ"

[[terms]]
en = "runtime"
ja = "ランタイム"

# Multi-language format
[[terms]]
en = "API"
translations = { ja = "API", zh = "API", ko = "API" }
```

## 📖 Commands

### `matome init`

Generate configuration templates in the current directory.

```bash
matome init                    # Create in current directory
matome init --output /path    # Create in specific directory
```

### `matome add`

Add a domain to your configuration.

```bash
matome add https://docs.example.com/
matome add https://docs.example.com/ --include "/docs/**"
```

### `matome crawl`

Crawl and translate documentation.

```bash
matome crawl                        # Full crawl
matome crawl --incremental          # Only new/changed pages
matome crawl --concurrency 4       # Override concurrency
```

### `matome serve`

Start the local web server.

```bash
matome serve                        # Default: 127.0.0.1:8080
matome serve --port 3000           # Custom port
matome serve --host 0.0.0.0        # Bind address
matome serve --data-dir .data      # Custom data directory
```

### `matome status`

Display database statistics.

```bash
matome status
matome status --verbose             # Detailed statistics
```

### `matome clean`

Manage and clean the database.

```bash
# Delete all articles
matome clean --all

# Delete articles from specific domain
matome clean --domain developer.mozilla.org

# Delete incomplete articles (missing title, translation, etc.)
matome clean --orphaned

# Delete specific article
matome clean --id 123
```

## 🌐 Web Interface

### Navigation

- **Sidebar** - Browse by domain, search, navigation
- **⌘K** - Open quick search modal
- **Domain filtering** - Click domains in sidebar to filter

### Reading View

- Toggle between **翻訳 (Translated)** and **原文 (Original)**
- Clean typography optimized for documentation
- Code blocks with syntax highlighting
- Links to original articles

### Search

- Full-text search across all articles (Tantivy)
- Live search with HTMX
- Domain-scoped searching via sidebar

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
│                         # Contains: id, url, title, description,
│                         #          original_md, translated_md,
│                         #          domain, crawled_at, updated_at
│
├── search_index/          # Tantivy full-text search index
│
└── .gitkeep             # Preserves directory structure
```

## 📋 Documentation Sites Supported

matome is optimized for these documentation formats:

- **Docusaurus** - GitHub Docs, React Native, Meta, etc.
- **MkDocs** - Python documentation ecosystem
- **Standard HTML** - Most documentation sites

Code blocks with nested elements (syntax highlighting `<span>` tags) are properly extracted.

## 🛠️ Troubleshooting

### "Failed to acquire index lock"

The search index is locked by another process. Stop any running `matome serve` instances:

```bash
pkill matome
```

Or delete the lock file:

```bash
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

### Full Workflow

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

### Incremental Update

```bash
# Run daily to get new documentation
matome crawl --incremental
```

### Clean and Rebuild

```bash
# Remove orphaned articles
matome clean --orphaned

# Remove all from a domain
matome clean --domain old-docs.example.com

# Start fresh
matome clean --all
matome crawl
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
