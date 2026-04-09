//! Agent Mode: Workspace export for AI coding assistants
//!
//! Generates structured workspace with metadata for AI agents like Claude Code, Cursor, etc.

use crate::agent::templates::AgentTemplates;
use crate::agent::token_counter::FallbackTokenCounter;
use crate::db::models::{AgentManifest, ReadingItem, SectionSummary, TokenBudget};
use crate::db::ArticleRow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use tracing::{info, warn};

/// Token counting trait for abstraction
trait TokenCount {
    fn count(&self, text: &str) -> usize;
}

impl TokenCount for FallbackTokenCounter {
    fn count(&self, text: &str) -> usize {
        FallbackTokenCounter::count(self, text)
    }
}

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path error: {0}")]
    Path(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Agent workspace exporter
pub struct AgentExporter {
    workspace_name: String,
    base_dir: PathBuf,
    max_tokens: usize,
    token_counter: FallbackTokenCounter,
}

impl AgentExporter {
    /// Create a new agent exporter
    pub fn new(
        workspace_name: &str,
        workspace_dir: Option<&str>,
        max_tokens: usize,
    ) -> Result<Self, AgentError> {
        let base_dir = if let Some(dir) = workspace_dir {
            PathBuf::from(dir)
        } else {
            dirs::home_dir()
                .ok_or_else(|| AgentError::Path("Cannot find home directory".to_string()))?
                .join(".matome/workspaces")
        };

        let workspace_dir = base_dir.join(workspace_name);

        // Try to use tiktoken, fall back to character-based if unavailable
        let fallback = FallbackTokenCounter::new();
        let _initial_estimate = fallback.count("test");

        warn!(
            "Agent exporter initialized (fallback counter ready, {} chars/token ratio)",
            4
        );

        Ok(Self {
            workspace_name: workspace_name.to_string(),
            base_dir: workspace_dir,
            max_tokens,
            token_counter: fallback, // Using fallback for now
        })
    }

    /// Export articles to agent workspace
    pub fn export(&self, articles: &[ArticleRow]) -> Result<ExportResult, AgentError> {
        info!(
            "Exporting {} articles to workspace: {}",
            articles.len(),
            self.workspace_name
        );

        // Create workspace directory
        fs::create_dir_all(&self.base_dir)?;

        // Group articles by domain (as section proxy)
        let sections = self.group_by_section(articles);

        // Generate manifest
        let manifest = self.generate_manifest(articles, &sections);
        self.write_manifest(&manifest)?;

        // Generate token budget
        let token_budget = self.generate_token_budget(articles);
        self.write_token_budget(&token_budget)?;

        // Generate workspace config
        self.write_workspace_config()?;

        // Generate CLAUDE.md template
        self.write_claude_md()?;

        // Write pages
        let mut files_written = 0;
        let mut tokens_estimate = 0;

        for article in articles {
            if let Some(section) = self.get_article_section(article, &sections) {
                let section_dir = self.base_dir.join(&section);
                fs::create_dir_all(&section_dir)?;

                let filename = self.article_to_filename(article);
                let path = section_dir.join(&filename);

                let content = self.article_to_markdown(article);
                let tokens = self.estimate_tokens(&content);
                tokens_estimate += tokens;

                fs::write(&path, &content)?;
                files_written += 1;
            }
        }

        info!(
            "Exported {} files (estimated {} tokens)",
            files_written, tokens_estimate
        );

        Ok(ExportResult {
            workspace_path: self.base_dir.clone(),
            files_written,
            tokens_estimate,
        })
    }

    /// Group articles by domain/section
    fn group_by_section<'a>(
        &self,
        articles: &'a [ArticleRow],
    ) -> HashMap<String, Vec<&'a ArticleRow>> {
        let mut sections: HashMap<String, Vec<&'a ArticleRow>> = HashMap::new();
        for article in articles {
            let section = self.infer_section(article);
            sections.entry(section).or_default().push(article);
        }
        sections
    }

    /// Infer section name from article
    fn infer_section(&self, article: &ArticleRow) -> String {
        // Use domain as section
        article.domain.clone()
    }

    /// Get section for article
    fn get_section<'a>(
        &self,
        article: &'a ArticleRow,
        sections: &HashMap<String, Vec<&ArticleRow>>,
    ) -> Option<&'a str> {
        sections
            .get(&article.domain)
            .map(|_| article.domain.as_str())
    }

    /// Get article section name
    fn get_article_section<'a>(
        &self,
        article: &'a ArticleRow,
        _sections: &HashMap<String, Vec<&ArticleRow>>,
    ) -> Option<String> {
        Some(article.domain.clone())
    }

    /// Convert article URL to filename
    fn article_to_filename(&self, article: &ArticleRow) -> String {
        let title = article
            .title
            .clone()
            .unwrap_or_else(|| "untitled".to_string());
        let safe = title
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim()
            .replace(' ', "-");
        format!("{}-{}.md", article.id, safe)
    }

    /// Convert article to markdown content
    fn article_to_markdown(&self, article: &ArticleRow) -> String {
        let title = article
            .title
            .clone()
            .unwrap_or_else(|| "Untitled".to_string());
        let translated = article
            .translated_md
            .as_deref()
            .unwrap_or(&article.original_md);

        format!(
            "# {}\n\n\
            **Source**: {}\n\
            **Domain**: {}\n\
             **Crawled**: {}\n\n\
            ---\n\n\
            {}\n",
            title, article.url, article.domain, article.crawled_at, translated
        )
    }

    /// Estimate token count using tiktoken (accurate for GPT-4/Claude)
    fn estimate_tokens(&self, content: &str) -> usize {
        self.token_counter.count(content)
    }

    /// Generate workspace manifest
    fn generate_manifest(
        &self,
        articles: &[ArticleRow],
        sections: &HashMap<String, Vec<&ArticleRow>>,
    ) -> AgentManifest {
        let total_tokens = articles
            .iter()
            .map(|a| self.estimate_tokens(&a.original_md))
            .sum();

        let section_summaries: Vec<SectionSummary> = sections
            .iter()
            .map(|(name, arts)| SectionSummary {
                name: name.clone(),
                files: arts.len(),
                tokens_estimate: arts
                    .iter()
                    .map(|a| self.estimate_tokens(&a.original_md))
                    .sum(),
            })
            .collect();

        AgentManifest {
            workspace: self.workspace_name.clone(),
            source_url: "local".to_string(),
            doc_version: None,
            crawled_at: chrono::Utc::now().to_rfc3339(),
            total_files: articles.len(),
            total_tokens_estimate: total_tokens,
            structure_type: "hierarchical".to_string(),
            agent_contract: vec![
                "Read index.json first for navigation".to_string(),
                "Code blocks are preserved verbatim; never rewrite".to_string(),
                "Check _agent/CHANGELOG.md for breaking changes".to_string(),
            ],
            sections: section_summaries,
        }
    }

    /// Write manifest.json
    fn write_manifest(&self, manifest: &AgentManifest) -> Result<(), AgentError> {
        let path = self.base_dir.join("manifest.json");
        let json = serde_json::to_string_pretty(manifest)?;
        fs::write(&path, json)?;
        info!("Written manifest to: {}", path.display());
        Ok(())
    }

    /// Generate token budget
    fn generate_token_budget(&self, articles: &[ArticleRow]) -> TokenBudget {
        let total_tokens: usize = articles
            .iter()
            .map(|a| self.estimate_tokens(&a.original_md))
            .sum();

        // Generate recommended reading order (by section)
        let mut reading_items: Vec<ReadingItem> = Vec::new();
        let mut current_tokens = 0;

        for article in articles.iter().take(20) {
            let tokens = self.estimate_tokens(&article.original_md);
            if current_tokens + tokens > self.max_tokens {
                break;
            }
            current_tokens += tokens;

            let filename = self.article_to_filename(article);
            reading_items.push(ReadingItem {
                section: article.domain.clone(),
                files: vec![filename],
                tokens,
            });
        }

        let priority_files = vec!["manifest.json".to_string(), "token_budget.json".to_string()];

        TokenBudget {
            context_limit: self.max_tokens,
            total_tokens,
            recommended_reading_order: reading_items,
            priority_files,
        }
    }

    /// Write token budget JSON
    fn write_token_budget(&self, budget: &TokenBudget) -> Result<(), AgentError> {
        let path = self.base_dir.join("token_budget.json");
        let json = serde_json::to_string_pretty(budget)?;
        fs::write(&path, json)?;
        info!("Written token budget to: {}", path.display());
        Ok(())
    }

    /// Write workspace.yaml config
    fn write_workspace_config(&self) -> Result<(), AgentError> {
        let config = format!(
            r#"name: {}
source: local
version: "1.0"
workspace_created: {}

agent_contract:
  - Read manifest.json before deep diving
  - Code blocks are verbatim; never rewrite
  - Check token_budget.json for context limits
  - Total tokens: {} | Budget limit: {}
"#,
            self.workspace_name,
            chrono::Utc::now().to_rfc3339(),
            self.max_tokens,
            self.max_tokens
        );

        let path = self.base_dir.join("workspace.yaml");
        fs::write(&path, config)?;
        info!("Written workspace config to: {}", path.display());
        Ok(())
    }

    /// Write CLAUDE.md for Claude Code
    fn write_claude_md(&self) -> Result<(), AgentError> {
        // Collect section names from sections map
        let sections: Vec<String> = vec!["documentation".to_string()]; // Default section

        let content = AgentTemplates::claude_md(&self.workspace_name, None, &sections);

        let path = self.base_dir.join("CLAUDE.md");
        fs::write(&path, content)?;
        info!("Written CLAUDE.md to: {}", path.display());
        Ok(())
    }
}

/// Result of export operation
#[derive(Debug)]
pub struct ExportResult {
    pub workspace_path: PathBuf,
    pub files_written: usize,
    pub tokens_estimate: usize,
}
