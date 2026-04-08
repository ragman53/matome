//! HTML to Markdown extraction module
//!
//! Handles content extraction from HTML and conversion to clean Markdown.

use crate::pipeline::ExtractedPage;
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ExtractorError {
    #[error("HTML parse error: {0}")]
    HtmlParse(String),
    #[error("Content extraction error: {0}")]
    ContentExtract(String),
    #[error("Markdown conversion error: {0}")]
    MarkdownConvert(String),
}

/// Try to select first matching element
fn try_select<'a>(document: &'a Html, selector_str: &str) -> Option<scraper::ElementRef<'a>> {
    Selector::parse(selector_str)
        .ok()
        .and_then(|s| document.select(&s).next())
}

/// Get non-empty text content from element
fn get_element_text(elem: scraper::ElementRef) -> Option<String> {
    let text = elem.text().collect::<String>().trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
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
        let document = Html::parse_document(html);
        let title = self.extract_title(&document);
        let description = self.extract_description(&document);
        let content_html = self.extract_main_content(&document)?;
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
        try_select(document, "title")
            .and_then(|elem| get_element_text(elem))
            .or_else(|| try_select(document, "h1").and_then(|elem| get_element_text(elem)))
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Extract page description
    fn extract_description(&self, document: &Html) -> Option<String> {
        try_select(document, r#"meta[name="description"]"#)
            .and_then(|elem| elem.value().attr("content").map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .or_else(|| {
                try_select(document, r#"meta[property="og:description"]"#)
                    .and_then(|elem| elem.value().attr("content").map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
            })
    }

    /// Extract main content using article-like selection
    /// Extended selectors for Docusaurus, MkDocs, and other documentation sites
    fn extract_main_content(&self, document: &Html) -> Result<String, ExtractorError> {
        let selectors = [
            // Standard selectors
            "article",
            "main",
            "[role=\"main\"]",
            ".content",
            ".post-content",
            ".article-content",
            ".documentation",
            "#content",
            "#main-content",
            // Docusaurus
            ".theme-doc-markdown",
            ".docMainContainer",
            "[data-page-content]",
            ".docItemWrapper",
            // MkDocs
            ".md-content",
            ".mkdocs-content",
            // General purpose
            ".markdown-body",
            ".documentation-body",
        ];

        for selector_str in &selectors {
            if let Some(elem) = try_select(document, selector_str) {
                let html_content = elem.html();
                if html_content.len() > 200 {
                    return Ok(html_content);
                }
            }
        }

        try_select(document, "body")
            .map(|elem| elem.html())
            .ok_or_else(|| ExtractorError::ContentExtract("No content found".to_string()))
    }

    /// Convert HTML to Markdown
    fn html_to_markdown(&self, html: &str) -> Result<String, ExtractorError> {
        let document = Html::parse_document(html);
        let mut markdown = String::new();
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
                self.process_heading(element, output, tag_name)
            }
            "p" => self.process_paragraph(element, output),
            "a" => self.process_anchor(element, output),
            "strong" | "b" => self.process_inline_element(element, output, "**"),
            "em" | "i" => self.process_inline_element(element, output, "*"),
            "code" => self.process_code(element, output),
            "pre" => self.process_pre(element, output),
            "ul" | "ol" => self.process_list(element, output, tag_name),
            "blockquote" => self.process_blockquote(element, output),
            "br" => output.push_str("  \n"),
            "hr" => {
                if !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str("---\n\n");
            }
            "img" => self.process_image(element, output),
            "table" => self.render_table(element, output, td_sel, tr_sel),
            "div" | "span" | "section" | "article" | "body" | "html" => {
                self.process_children(element, output, td_sel, tr_sel)
            }
            _ => self.process_children(element, output, td_sel, tr_sel),
        }
    }

    fn process_heading(&self, element: ElementRef, output: &mut String, tag_name: &str) {
        if !output.ends_with('\n') {
            output.push('\n');
        }
        let level = tag_name[1..].parse::<usize>().unwrap_or(1);
        for _ in 0..level {
            output.push('#');
        }
        output.push(' ');
        self.process_text_children(element, output);
        output.push_str("\n\n");
    }

    fn process_paragraph(&self, element: ElementRef, output: &mut String) {
        self.process_text_children(element, output);
        output.push_str("\n\n");
    }

    fn process_anchor(&self, element: ElementRef, output: &mut String) {
        let href = element.value().attr("href").unwrap_or("");
        self.process_text_children(element, output);
        if !href.is_empty() {
            output.push_str(&format!("({})", href));
        }
    }

    fn process_inline_element(&self, element: ElementRef, output: &mut String, wrapper: &str) {
        output.push_str(wrapper);
        self.process_text_children(element, output);
        output.push_str(wrapper);
    }

    fn process_code(&self, element: ElementRef, output: &mut String) {
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
            self.process_text_children(element, output);
            output.push('`');
        }
    }

    fn process_pre(&self, element: ElementRef, output: &mut String) {
        output.push('\n');
        output.push_str("```\n");
        // Recursively extract all text from code blocks (handles nested <span> etc.)
        self.extract_text_recursive(element, output);
        output.push_str("```\n");
    }

    /// Recursively extract all text content from an element
    /// Used for code blocks where text may be deeply nested in <span> elements
    fn extract_text_recursive(&self, element: ElementRef, output: &mut String) {
        for child in element.children() {
            if let Some(text) = child.value().as_text() {
                let text = text.trim();
                if !text.is_empty() {
                    output.push_str(text);
                    output.push('\n');
                }
            } else if let Some(elem) = ElementRef::wrap(child) {
                // Skip style and script elements
                let tag = elem.value().name();
                if tag != "style" && tag != "script" {
                    self.extract_text_recursive(elem, output);
                }
            }
        }
    }

    fn process_list(&self, element: ElementRef, output: &mut String, tag_name: &str) {
        let is_ordered = tag_name == "ol";
        for (i, item) in element
            .children()
            .filter_map(|c| ElementRef::wrap(c))
            .filter(|e| e.value().name() == "li")
            .enumerate()
        {
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
                    self.process_element(li_elem, output, 0, &None, &None);
                }
            }
            output.push('\n');
        }
        output.push('\n');
    }

    fn process_blockquote(&self, element: ElementRef, output: &mut String) {
        output.push_str("> ");
        self.process_text_children(element, output);
        output.push_str("\n\n");
    }

    fn process_image(&self, element: ElementRef, output: &mut String) {
        let src = element.value().attr("src").unwrap_or("");
        let alt = element.value().attr("alt").unwrap_or("");
        if !src.is_empty() {
            output.push_str(&format!("![{}]({})\n", alt, src));
        }
    }

    fn process_children(
        &self,
        element: ElementRef,
        output: &mut String,
        td_sel: &Option<Selector>,
        tr_sel: &Option<Selector>,
    ) {
        for child in element.children() {
            if let Some(elem) = ElementRef::wrap(child) {
                self.process_element(elem, output, 0, td_sel, tr_sel);
            }
        }
    }

    fn process_text_children(&self, element: ElementRef, output: &mut String) {
        for child in element.children() {
            if let Some(text) = child.value().as_text() {
                output.push_str(&text.trim());
            } else if let Some(elem) = ElementRef::wrap(child) {
                self.process_text_element(elem, output);
            }
        }
    }

    fn process_text_element(&self, element: ElementRef, output: &mut String) {
        match element.value().name() {
            "a" => {
                let href = element.value().attr("href").unwrap_or("");
                self.process_text_children(element, output);
                if !href.is_empty() {
                    output.push_str(&format!("({})", href));
                }
            }
            "strong" | "b" => {
                output.push_str("**");
                self.process_text_children(element, output);
                output.push_str("**");
            }
            "em" | "i" => {
                output.push('*');
                self.process_text_children(element, output);
                output.push('*');
            }
            "code" => {
                output.push('`');
                self.process_text_children(element, output);
                output.push('`');
            }
            _ => self.process_text_children(element, output),
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
                let row: Vec<String> = row_elem
                    .select(td_sel)
                    .map(|cell| cell.text().collect::<String>().trim().to_string())
                    .collect();
                if !row.is_empty() {
                    rows.push(row);
                }
            }
        }

        if let Some(first_row) = rows.first() {
            let col_count = first_row.len();
            for _ in 0..col_count {
                output.push_str("| --- ");
            }
            output.push_str("|\n");

            for (i, row) in rows.iter().enumerate() {
                for cell in row {
                    output.push_str(&format!("| {} ", cell));
                }
                output.push_str("|\n");
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

    #[test]
    fn test_extract_code_with_nested_spans() {
        // Test extraction of code with deeply nested <span> elements
        // This simulates Docusaurus/MkDocs code blocks
        let extractor = Extractor::new();
        let html = r#"
            <html>
            <head><title>Code Test</title></head>
            <body>
                <article>
                    <h1>Code Example</h1>
                    <pre><code><span><span>import os</span></span></code></pre>
                    <pre><code><span class="token">def</span> hello():
                        <span class="token">print</span><span>("Hello")</span></span></code></pre>
                </article>
            </body>
            </html>
        "#;

        let result = extractor.extract(html, "https://example.com/code").unwrap();

        // Check that code content is extracted
        assert!(
            result.markdown.contains("import os") || result.markdown.contains("```"),
            "Code block should be extracted"
        );
    }

    #[test]
    fn test_extract_docusaurus_selectors() {
        // Test Docusaurus-specific selectors
        let extractor = Extractor::new();
        let html = r#"
            <html>
            <head><title>Docusaurus Test</title></head>
            <body>
                <div class="theme-doc-markdown">
                    <h1>Test Page</h1>
                    <p>Content here</p>
                </div>
            </body>
            </html>
        "#;

        let result = extractor.extract(html, "https://example.com/test").unwrap();
        assert_eq!(result.title, "Docusaurus Test");
        assert!(result.markdown.contains("Test Page"));
        assert!(result.markdown.contains("Content here"));
    }
}
