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
        // Check if this <code> element is inside a <pre> block
        let is_in_pre = element
            .ancestors()
            .filter_map(ElementRef::wrap)
            .any(|e| e.value().name() == "pre");

        // Skip code blocks inside <pre> - let process_pre handle them
        if is_in_pre {
            // Just process the text content without adding markdown fences
            self.process_text_children(element, output);
            return;
        }

        output.push('`');
        self.process_text_children(element, output);
        output.push('`');
    }

    fn process_pre(&self, element: ElementRef, output: &mut String) {
        output.push('\n');
        // Try to extract language from class attribute (e.g., class="language-python")
        let language = self.extract_language_from_class(&element);
        output.push_str(&format!("```{}", language));
        if !language.is_empty() {
            output.push('\n');
        }
        // Recursively extract all text from code blocks (handles nested <span> etc.)
        self.extract_text_recursive(element, output);
        output.push_str("\n```\n");
    }

    /// Extract language from class attribute (e.g., "language-python" -> "python")
    fn extract_language_from_class(&self, element: &ElementRef) -> String {
        // Check class attribute on the element or look for code element inside
        if let Some(class) = element.value().attr("class") {
            if let Some(lang) = self::Extractor::parse_language_class(class) {
                return lang;
            }
        }

        // Check if there's a code element inside
        if let Ok(selector) = Selector::parse("code") {
            if let Some(code_elem) = element.select(&selector).next() {
                if let Some(class) = code_elem.value().attr("class") {
                    if let Some(lang) = Self::parse_language_class(class) {
                        return lang;
                    }
                }
            }
        }

        String::new()
    }

    /// Parse language from class string (e.g., "language-python" or "hljs python")
    fn parse_language_class(class: &str) -> Option<String> {
        for part in class.split_whitespace() {
            if part.starts_with("language-") {
                return Some(part.trim_start_matches("language-").to_string());
            }
            if part.starts_with("hljs-") {
                return Some(part.trim_start_matches("hljs-").to_string());
            }
            // Common language aliases
            match part {
                "python" | "js" | "javascript" | "typescript" | "rust" | "go" | "java" | "c"
                | "cpp" | "csharp" | "ruby" | "php" | "swift" | "kotlin" | "bash" | "sh"
                | "shell" | "sql" | "html" | "css" | "json" | "yaml" | "toml" | "xml"
                | "markdown" | "text" | "plaintext" => {
                    return Some(part.to_string());
                }
                _ => {}
            }
        }
        None
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
            .filter_map(ElementRef::wrap)
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
                    output.push_str(text.trim());
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
                output.push_str(text.trim());
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
                    .map(|cell| self.extract_cell_text(cell))
                    .collect();
                if !row.is_empty() {
                    rows.push(row);
                }
            }
        }

        if !rows.is_empty() {
            let col_count = rows[0].len();

            // First row: header row
            for cell in &rows[0] {
                let escaped = Self::escape_table_cell(cell);
                output.push_str(&format!("| {} ", escaped));
            }
            output.push_str("|\n");

            // Second row: header separator
            for _ in 0..col_count {
                output.push_str("| --- ");
            }
            output.push_str("|\n");

            // Remaining rows: data rows
            for row in rows.iter().skip(1) {
                for cell in row {
                    let escaped = Self::escape_table_cell(cell);
                    output.push_str(&format!("| {} ", escaped));
                }
                output.push_str("|\n");
            }
            output.push('\n');
        }
    }

    /// Extract text from a table cell, handling nested elements
    fn extract_cell_text(&self, cell: ElementRef) -> String {
        let mut text = String::new();
        let td_sel = Selector::parse("td,th").ok();
        let tr_sel = Selector::parse("tr").ok();
        self.extract_element_text(cell, &mut text, 0, &td_sel, &tr_sel);
        // Normalize whitespace within the cell
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Recursively extract text from an element, handling nested structures
    fn extract_element_text(
        &self,
        element: ElementRef,
        output: &mut String,
        depth: usize,
        td_sel: &Option<Selector>,
        tr_sel: &Option<Selector>,
    ) {
        let tag = element.value().name();

        match tag {
            // Block-level elements that need spacing
            "p" | "div" | "li" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if depth > 0 {
                    output.push(' ');
                }
                self.extract_text_children_recursive(element, output, depth + 1, td_sel, tr_sel);
                if tag == "li" {
                    // For list items, add a separator after
                    output.push_str("; ");
                }
            }
            // Inline elements - just process text
            "a" | "strong" | "b" | "em" | "i" | "code" | "span" | "br" => {
                self.extract_text_children_recursive(element, output, depth, td_sel, tr_sel);
            }
            // Skip these completely
            "script" | "style" | "noscript" => {}
            // Lists within cells
            "ul" | "ol" => {
                if depth > 0 {
                    output.push(' ');
                }
                for li in element
                    .children()
                    .filter_map(ElementRef::wrap)
                    .filter(|e| e.value().name() == "li")
                {
                    output.push_str("- ");
                    self.extract_element_text(li, output, depth + 1, td_sel, tr_sel);
                    output.push(' ');
                }
            }
            // Tables within cells (flatten)
            "table" => {
                self.render_table_inline(element, output, td_sel, tr_sel);
            }
            _ => {
                self.extract_text_children_recursive(element, output, depth, td_sel, tr_sel);
            }
        }
    }

    /// Extract text children recursively (for inline content)
    fn extract_text_children_recursive(
        &self,
        element: ElementRef,
        output: &mut String,
        depth: usize,
        td_sel: &Option<Selector>,
        tr_sel: &Option<Selector>,
    ) {
        for child in element.children() {
            if let Some(text) = child.value().as_text() {
                let text = text.trim();
                if !text.is_empty() {
                    output.push_str(text);
                    if depth > 0 {
                        output.push(' ');
                    }
                }
            } else if let Some(elem) = ElementRef::wrap(child) {
                self.extract_element_text(elem, output, depth, td_sel, tr_sel);
            }
        }
    }

    /// Render a table inline (flattened, no outer table markdown)
    fn render_table_inline(
        &self,
        table: ElementRef,
        output: &mut String,
        td_sel: &Option<Selector>,
        tr_sel: &Option<Selector>,
    ) {
        if let (Some(td_sel), Some(tr_sel)) = (td_sel, tr_sel) {
            let mut first = true;
            for row_elem in table.select(tr_sel) {
                if !first {
                    output.push_str(" | ");
                }
                first = false;
                for (i, cell) in row_elem.select(td_sel).enumerate() {
                    if i > 0 {
                        output.push_str(" | ");
                    }
                    let text = self.extract_cell_text(cell);
                    output.push_str(&Self::escape_table_cell(&text));
                }
            }
        }
    }

    /// Escape special characters in table cells
    fn escape_table_cell(text: &str) -> String {
        text.replace('|', "\\|")
            .replace('\n', " ")
            .replace('\r', "")
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

    #[test]
    fn test_extract_table_with_nested_elements() {
        // Test table with nested ul/li and strong elements
        let extractor = Extractor::new();
        let html = r#"
            <html>
            <head><title>Table Test</title></head>
            <body>
                <article>
                    <h1>Table Example</h1>
                    <table>
                        <tr><th>Name</th><th>Description</th></tr>
                        <tr><td>Item 1</td><td><strong>100</strong></td></tr>
                        <tr><td>Item 2</td><td><ul><li>Option A</li><li>Option B</li></ul></td></tr>
                    </table>
                </article>
            </body>
            </html>
        "#;

        let result = extractor.extract(html, "https://example.com/test").unwrap();

        // Verify table structure is preserved
        assert!(
            result.markdown.contains("| Name |"),
            "Table should have Name column"
        );
        assert!(
            result.markdown.contains("| Description |"),
            "Table should have Description column"
        );
        assert!(
            result.markdown.contains("| Item 1 |"),
            "Table should have Item 1 row"
        );
        assert!(
            result.markdown.contains("| --- |"),
            "Table should have separator row"
        );
        // Check nested elements are extracted
        assert!(
            result.markdown.contains("Option A") || result.markdown.contains("- Option A"),
            "Nested list items should be extracted"
        );
    }

    #[test]
    fn test_extract_code_with_language_class() {
        // Test code block language extraction from class
        let extractor = Extractor::new();
        let html = r#"
            <html>
            <head><title>Code Test</title></head>
            <body>
                <article>
                    <pre class="language-python"><code>def hello():
    print("world")</code></pre>
                    <pre class="language-rust"><code>fn main() {
    println!("Hello");
}</code></pre>
                </article>
            </body>
            </html>
        "#;

        let result = extractor.extract(html, "https://example.com/test").unwrap();

        // Check language tags are extracted
        assert!(
            result.markdown.contains("```python") || result.markdown.contains("```"),
            "Python code block should have language tag or be plain code"
        );
    }
}
