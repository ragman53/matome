//! HTML to Markdown extraction module
//!
//! Handles content extraction from HTML and conversion to clean Markdown.

use crate::pipeline::ExtractedPage;
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractorError {
    #[error("HTML parse error: {0}")]
    HtmlParse(String),
    #[error("Content extraction error: {0}")]
    ContentExtract(String),
    #[error("Markdown conversion error: {0}")]
    MarkdownConvert(String),
}

/// HTML to Markdown extractor
#[derive(Clone)]
pub struct Extractor;

impl Extractor {
    /// Create a new extractor instance
    pub fn new() -> Self {
        Self
    }

    /// Extract content from HTML and convert to Markdown
    pub fn extract(&self, html: &str, url: &str) -> Result<ExtractedPage, ExtractorError> {
        // Parse HTML
        let document = Html::parse_document(html);

        // Extract title
        let title = self.extract_title(&document);

        // Extract description
        let description = self.extract_description(&document);

        // Extract main content using readability-like algorithm
        let content_html = self.extract_main_content(&document)?;

        // Convert to Markdown
        let markdown = self.html_to_markdown(&content_html)?;

        Ok(ExtractedPage {
            url: url.to_string(),
            title,
            description,
            markdown,
        })
    }

    /// Extract page title
    fn extract_title(&self, document: &Html) -> String {
        // Try <title> first
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(title_elem) = document.select(&title_selector).next() {
                let title = title_elem.text().collect::<String>().trim().to_string();
                if !title.is_empty() {
                    return title;
                }
            }
        }

        // Fall back to <h1>
        if let Ok(h1_selector) = Selector::parse("h1") {
            if let Some(h1_elem) = document.select(&h1_selector).next() {
                let title = h1_elem.text().collect::<String>().trim().to_string();
                if !title.is_empty() {
                    return title;
                }
            }
        }

        "Untitled".to_string()
    }

    /// Extract page description
    fn extract_description(&self, document: &Html) -> Option<String> {
        // Look for meta description
        if let Ok(meta_selector) = Selector::parse(r#"meta[name="description"]"#) {
            if let Some(meta_elem) = document.select(&meta_selector).next() {
                if let Some(content) = meta_elem.value().attr("content") {
                    let desc = content.trim().to_string();
                    if !desc.is_empty() {
                        return Some(desc);
                    }
                }
            }
        }

        // Try og:description
        if let Ok(og_selector) = Selector::parse(r#"meta[property="og:description"]"#) {
            if let Some(og_elem) = document.select(&og_selector).next() {
                if let Some(content) = og_elem.value().attr("content") {
                    let desc = content.trim().to_string();
                    if !desc.is_empty() {
                        return Some(desc);
                    }
                }
            }
        }

        None
    }

    /// Extract main content using article-like selection
    fn extract_main_content(&self, document: &Html) -> Result<String, ExtractorError> {
        // Priority order for content extraction
        let selectors = [
            "article",
            "main",
            "[role=\"main\"]",
            ".content",
            ".post-content",
            ".article-content",
            ".documentation",
            "#content",
            "#main-content",
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(elem) = document.select(&selector).next() {
                    let html_content = elem.html();
                    if html_content.len() > 200 {
                        return Ok(html_content);
                    }
                }
            }
        }

        // Fall back to body
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body_elem) = document.select(&body_selector).next() {
                return Ok(body_elem.html());
            }
        }

        Err(ExtractorError::ContentExtract(
            "No content found".to_string(),
        ))
    }

    /// Convert HTML to Markdown
    fn html_to_markdown(&self, html: &str) -> Result<String, ExtractorError> {
        let document = Html::parse_document(html);
        let mut markdown = String::new();

        // Pre-parse table selectors
        let td_sel = Selector::parse("td,th").ok();
        let tr_sel = Selector::parse("tr").ok();

        self.process_element(document.root_element(), &mut markdown, 0, &td_sel, &tr_sel);
        Ok(markdown)
    }

    /// Process HTML element recursively
    fn process_element(
        &self,
        element: ElementRef,
        output: &mut String,
        _indent: usize,
        td_sel: &Option<Selector>,
        tr_sel: &Option<Selector>,
    ) {
        let tag_name = element.value().name();

        match tag_name {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if !output.ends_with('\n') {
                    output.push('\n');
                }
                let level = tag_name[1..].parse::<usize>().unwrap_or(1);
                for _ in 0..level {
                    output.push('#');
                }
                output.push(' ');
                for child in element.children() {
                    if let Some(text) = child.value().as_text() {
                        output.push_str(&text.trim());
                    } else if let Some(elem) = ElementRef::wrap(child) {
                        self.process_element(elem, output, _indent, td_sel, tr_sel);
                    }
                }
                output.push_str("\n\n");
            }
            "p" => {
                for child in element.children() {
                    if let Some(text) = child.value().as_text() {
                        output.push_str(&text.trim());
                    } else if let Some(elem) = ElementRef::wrap(child) {
                        self.process_element(elem, output, _indent, td_sel, tr_sel);
                    }
                }
                output.push_str("\n\n");
            }
            "a" => {
                let href = element.value().attr("href").unwrap_or("");
                for child in element.children() {
                    if let Some(text) = child.value().as_text() {
                        if href.is_empty() {
                            output.push_str(&text.trim());
                        } else {
                            output.push_str(&format!("[{}]({})", text.trim(), href));
                        }
                    }
                }
            }
            "strong" | "b" => {
                output.push_str("**");
                for child in element.children() {
                    if let Some(text) = child.value().as_text() {
                        output.push_str(&text.trim());
                    }
                }
                output.push_str("**");
            }
            "em" | "i" => {
                output.push_str("*");
                for child in element.children() {
                    if let Some(text) = child.value().as_text() {
                        output.push_str(&text.trim());
                    }
                }
                output.push_str("*");
            }
            "code" => {
                let is_block = element
                    .parent()
                    .and_then(|p| ElementRef::wrap(p))
                    .map(|e| e.value().name() == "pre")
                    .unwrap_or(false);

                if is_block {
                    output.push('\n');
                    output.push_str("```\n");
                    for child in element.children() {
                        if let Some(text) = child.value().as_text() {
                            output.push_str(text.trim());
                            output.push('\n');
                        }
                    }
                    output.push_str("```\n");
                } else {
                    output.push('`');
                    for child in element.children() {
                        if let Some(text) = child.value().as_text() {
                            output.push_str(&text.trim());
                        }
                    }
                    output.push('`');
                }
            }
            "pre" => {
                output.push('\n');
                output.push_str("```\n");
                for child in element.children() {
                    if let Some(text) = child.value().as_text() {
                        output.push_str(text.trim());
                        output.push('\n');
                    } else if let Some(inner) = ElementRef::wrap(child) {
                        if inner.value().name() == "code" {
                            for grandchild in child.children() {
                                if let Some(text) = grandchild.value().as_text() {
                                    output.push_str(text.trim());
                                    output.push('\n');
                                }
                            }
                        }
                    }
                }
                output.push_str("```\n");
            }
            "ul" | "ol" => {
                let is_ordered = tag_name == "ol";
                let mut items: Vec<ElementRef> = Vec::new();
                for child in element.children() {
                    if let Some(elem) = ElementRef::wrap(child) {
                        if elem.value().name() == "li" {
                            items.push(elem);
                        }
                    }
                }
                for (i, item) in items.iter().enumerate() {
                    let prefix = if is_ordered {
                        format!("{}. ", i + 1)
                    } else {
                        "- ".to_string()
                    };
                    output.push_str(&prefix);
                    for li_child in item.children() {
                        if let Some(text) = li_child.value().as_text() {
                            output.push_str(&text.trim());
                        } else if let Some(li_elem) = ElementRef::wrap(li_child) {
                            self.process_element(li_elem, output, _indent + 1, td_sel, tr_sel);
                        }
                    }
                    output.push('\n');
                }
                output.push('\n');
            }
            "blockquote" => {
                output.push_str("> ");
                for child in element.children() {
                    if let Some(text) = child.value().as_text() {
                        output.push_str(&text.trim());
                    }
                }
                output.push_str("\n\n");
            }
            "br" => {
                output.push_str("  \n");
            }
            "hr" => {
                if !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str("---\n\n");
            }
            "img" => {
                let src = element.value().attr("src").unwrap_or("");
                let alt = element.value().attr("alt").unwrap_or("");
                if !src.is_empty() {
                    output.push_str(&format!("![{}]({})\n", alt, src));
                }
            }
            "table" => {
                // Simple table conversion
                self.render_table(element, output, td_sel, tr_sel);
            }
            "div" | "span" | "section" | "article" | "body" | "html" => {
                for child in element.children() {
                    if let Some(elem) = ElementRef::wrap(child) {
                        self.process_element(elem, output, _indent, td_sel, tr_sel);
                    }
                }
            }
            _ => {
                for child in element.children() {
                    if let Some(elem) = ElementRef::wrap(child) {
                        self.process_element(elem, output, _indent, td_sel, tr_sel);
                    }
                }
            }
        }
    }

    /// Render table as Markdown
    fn render_table(
        &self,
        table: ElementRef,
        output: &mut String,
        td_sel: &Option<Selector>,
        tr_sel: &Option<Selector>,
    ) {
        let mut rows: Vec<Vec<String>> = Vec::new();

        if let (Some(td_sel), Some(tr_sel)) = (td_sel, tr_sel) {
            for row_elem in table.select(tr_sel) {
                let mut row: Vec<String> = Vec::new();

                for cell in row_elem.select(td_sel) {
                    let cell_text: String = cell.text().collect();
                    row.push(cell_text.trim().to_string());
                }

                if !row.is_empty() {
                    rows.push(row);
                }
            }
        }

        if let Some(first_row) = rows.first() {
            let col_count = first_row.len();

            // Header separator
            for _ in 0..col_count {
                output.push_str("| --- ");
            }
            output.push_str("|\n");

            for (i, row) in rows.iter().enumerate() {
                for cell in row {
                    output.push_str(&format!("| {} ", cell));
                }
                output.push_str("|\n");

                // Header separator after first row
                if i == 0 {
                    for _ in 0..col_count {
                        output.push_str("| --- ");
                    }
                    output.push_str("|\n");
                }
            }
            output.push('\n');
        }
    }
}

impl Default for Extractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let extractor = Extractor::new();
        let html = r#"<html><head><title>Test Page Title</title></head><body></body></html>"#;
        let result = extractor.extract(html, "https://example.com/test").unwrap();
        assert_eq!(result.title, "Test Page Title");
    }

    #[test]
    fn test_extract_simple_html() {
        let extractor = Extractor::new();
        let html = r#"
            <html>
            <head><title>Test</title></head>
            <body>
                <h1>Hello World</h1>
                <p>This is a test paragraph with <strong>bold</strong> text.</p>
                <pre><code>fn main() { println!("Hello"); }</code></pre>
            </body>
            </html>
        "#;

        let result = extractor.extract(html, "https://example.com/test").unwrap();
        assert_eq!(result.title, "Test");
        assert!(result.markdown.contains("Hello World"));
        assert!(result.markdown.contains("**bold**"));
        assert!(result.markdown.contains("fn main()"));
    }
}
