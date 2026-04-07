//! Translation module
//!
//! Handles Markdown translation using various providers (Ollama, DeepL, etc.)

use crate::config::Config;
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranslatorError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Translator trait for abstraction
#[async_trait]
pub trait TranslatorTrait: Send + Sync {
    async fn translate(&self, text: &str) -> Result<String, TranslatorError>;
}

/// Ollama translator
pub struct OllamaTranslator {
    client: reqwest::Client,
    endpoint: String,
    model: String,
    target_lang: String,
}

impl OllamaTranslator {
    pub fn new(config: &Config) -> Result<Self, TranslatorError> {
        let endpoint = std::env::var("OLLAMA_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        Ok(Self {
            client: reqwest::Client::new(),
            endpoint,
            model: config.translate.model.clone(),
            target_lang: config.translate.target_lang.clone(),
        })
    }

    async fn call_api(&self, prompt: &str) -> Result<String, TranslatorError> {
        let url = format!("{}/api/generate", self.endpoint);

        #[derive(serde::Serialize)]
        struct Request {
            model: String,
            prompt: String,
            stream: bool,
        }

        #[derive(serde::Deserialize)]
        struct Response {
            response: String,
        }

        let request = Request {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(TranslatorError::Api(format!(
                "Ollama API returned status: {}",
                response.status()
            )));
        }

        let result: Response = response
            .json()
            .await
            .map_err(|e| TranslatorError::Parse(e.to_string()))?;

        Ok(result.response)
    }

    fn get_language_name(&self) -> &'static str {
        match self.target_lang.as_str() {
            "ja" => "Japanese",
            "ko" => "Korean",
            "zh" => "Chinese",
            "es" => "Spanish",
            "fr" => "French",
            "de" => "German",
            _ => "Japanese",
        }
    }

    fn split_by_code_blocks(&self, text: &str) -> Vec<CodePart> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_code_block = false;
        let mut code_fence = String::new();

        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '`' {
                let mut fence = String::from(c);
                for _ in 0..2 {
                    if chars.peek() == Some(&'`') {
                        fence.push(chars.next().unwrap());
                    }
                }

                if in_code_block {
                    if fence == code_fence {
                        current.push_str(&fence);
                        current.push('\n');
                        parts.push(CodePart {
                            is_code: true,
                            content: current.clone(),
                        });
                        current.clear();
                        in_code_block = false;
                        code_fence.clear();
                    } else {
                        current.push_str(&fence);
                    }
                } else if fence.len() == 3 {
                    current.push_str(&fence);
                    code_fence = fence.clone();
                    parts.push(CodePart {
                        is_code: false,
                        content: current.clone(),
                    });
                    current.clear();
                    in_code_block = true;
                } else {
                    current.push_str(&fence);
                }
            } else {
                current.push(c);
            }
        }

        if !current.is_empty() {
            parts.push(CodePart {
                is_code: in_code_block,
                content: current,
            });
        }

        if parts.is_empty() {
            parts.push(CodePart {
                is_code: false,
                content: text.to_string(),
            });
        }

        parts
    }
}

struct CodePart {
    is_code: bool,
    content: String,
}

#[async_trait]
impl TranslatorTrait for OllamaTranslator {
    async fn translate(&self, text: &str) -> Result<String, TranslatorError> {
        // Detect code blocks and preserve them
        let parts = self.split_by_code_blocks(text);
        let mut results = Vec::new();

        for part in parts {
            if part.is_code {
                results.push(part.content);
            } else {
                let prompt = format!(
                    "Translate the following Markdown text to {}. \
                     Preserve all Markdown formatting (headers, lists, code blocks, links, etc.). \
                     Do not translate code, technical terms in backticks, or links. \
                     Only translate the prose content.\n\n{}",
                    self.get_language_name(),
                    part.content
                );

                let translated = self.call_api(&prompt).await?;
                results.push(translated.trim().to_string());
            }
        }

        Ok(results.join("\n"))
    }
}

/// DeepL translator
pub struct DeepLTranslator {
    client: reqwest::Client,
    api_key: String,
    target_lang: String,
}

impl DeepLTranslator {
    pub fn new(config: &Config) -> Result<Self, TranslatorError> {
        let api_key = std::env::var("DEEPL_API_KEY")
            .map_err(|_| TranslatorError::Config("DEEPL_API_KEY not set".to_string()))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            target_lang: config.translate.target_lang.clone(),
        })
    }
}

#[async_trait]
impl TranslatorTrait for DeepLTranslator {
    async fn translate(&self, text: &str) -> Result<String, TranslatorError> {
        #[derive(serde::Serialize)]
        struct Request<'a> {
            auth_key: &'a str,
            text: &'a str,
            target_lang: &'a str,
        }

        #[derive(serde::Deserialize)]
        struct Response {
            translations: Vec<Translation>,
        }

        #[derive(serde::Deserialize)]
        struct Translation {
            text: String,
        }

        let request = Request {
            auth_key: &self.api_key,
            text,
            target_lang: &self.target_lang,
        };

        let response = self
            .client
            .post("https://api-free.deepl.com/v2/translate")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(TranslatorError::Api(format!(
                "DeepL API returned status: {}",
                response.status()
            )));
        }

        let result: Response = response
            .json()
            .await
            .map_err(|e| TranslatorError::Parse(e.to_string()))?;

        Ok(result
            .translations
            .first()
            .map(|t| t.text.clone())
            .unwrap_or_default())
    }
}

/// Main Translator wrapper
#[derive(Clone)]
pub struct Translator {
    inner: Arc<Box<dyn TranslatorTrait>>,
}

impl Translator {
    pub fn new(config: &Config) -> Result<Self, TranslatorError> {
        let inner: Box<dyn TranslatorTrait> = match config.translate.provider.as_str() {
            "ollama" => Box::new(OllamaTranslator::new(config)?),
            "deepl" => Box::new(DeepLTranslator::new(config)?),
            _ => {
                return Err(TranslatorError::Config(format!(
                    "Unknown provider: {}",
                    config.translate.provider
                )))
            }
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub async fn translate(&self, text: &str) -> Result<String, TranslatorError> {
        self.inner.translate(text).await
    }
}

impl std::fmt::Debug for Translator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Translator").finish()
    }
}
