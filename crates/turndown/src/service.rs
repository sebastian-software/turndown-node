//! TurndownService - the main entry point for HTML to Markdown conversion.

use scraper::{ElementRef, Html, Node};

use crate::rules::{Filter, Rule, Rules};
use crate::Result;

/// Heading style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadingStyle {
    /// Use setext-style headings (underlined with = or -)
    #[default]
    Setext,
    /// Use ATX-style headings (prefixed with #)
    Atx,
}

/// Code block style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeBlockStyle {
    /// Use indented code blocks (4 spaces)
    #[default]
    Indented,
    /// Use fenced code blocks (```)
    Fenced,
}

/// Link style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkStyle {
    /// Use inline links [text](url)
    #[default]
    Inlined,
    /// Use reference links [text][ref]
    Referenced,
}

/// Reference style for referenced links
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkReferenceStyle {
    #[default]
    Full,
    Collapsed,
    Shortcut,
}

/// Options for TurndownService
#[derive(Debug, Clone)]
pub struct TurndownOptions {
    /// Heading style (setext or atx)
    pub heading_style: HeadingStyle,

    /// Horizontal rule string
    pub hr: String,

    /// Bullet list marker
    pub bullet_list_marker: char,

    /// Code block style
    pub code_block_style: CodeBlockStyle,

    /// Fence string for fenced code blocks
    pub fence: String,

    /// Emphasis delimiter
    pub em_delimiter: char,

    /// Strong delimiter
    pub strong_delimiter: String,

    /// Link style
    pub link_style: LinkStyle,

    /// Reference style for referenced links
    pub link_reference_style: LinkReferenceStyle,
}

impl Default for TurndownOptions {
    fn default() -> Self {
        Self {
            heading_style: HeadingStyle::Setext,
            hr: "* * *".to_string(),
            bullet_list_marker: '*',
            code_block_style: CodeBlockStyle::Indented,
            fence: "```".to_string(),
            em_delimiter: '_',
            strong_delimiter: "**".to_string(),
            link_style: LinkStyle::Inlined,
            link_reference_style: LinkReferenceStyle::Full,
        }
    }
}

/// The main service for converting HTML to Markdown
pub struct TurndownService {
    options: TurndownOptions,
    rules: Rules,
}

impl TurndownService {
    /// Create a new TurndownService with default options
    pub fn new() -> Self {
        Self {
            options: TurndownOptions::default(),
            rules: Rules::new(),
        }
    }

    /// Create a TurndownService with custom options
    pub fn with_options(options: TurndownOptions) -> Self {
        Self {
            options,
            rules: Rules::new(),
        }
    }

    /// Convert HTML to Markdown
    pub fn turndown(&self, html: &str) -> Result<String> {
        let document = Html::parse_fragment(html);

        // Process the document
        let result = self.process_children(document.root_element());

        // Post-process
        Ok(self.post_process(&result))
    }

    /// Add a custom rule
    pub fn add_rule(&mut self, key: &str, rule: Rule) -> &mut Self {
        self.rules.add(key, rule);
        self
    }

    /// Keep elements matching the filter as HTML
    pub fn keep(&mut self, filter: Filter) -> &mut Self {
        self.rules.keep(filter);
        self
    }

    /// Remove elements matching the filter
    pub fn remove(&mut self, filter: Filter) -> &mut Self {
        self.rules.remove(filter);
        self
    }

    /// Apply a plugin
    pub fn use_plugin<F>(&mut self, plugin: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        plugin(self);
        self
    }

    /// Escape markdown special characters in a string
    pub fn escape(&self, text: &str) -> String {
        crate::utilities::escape_markdown(text)
    }

    /// Get the current options
    pub fn options(&self) -> &TurndownOptions {
        &self.options
    }

    /// Get mutable access to options
    pub fn options_mut(&mut self) -> &mut TurndownOptions {
        &mut self.options
    }

    /// Process children of an element
    fn process_children(&self, element: ElementRef) -> String {
        let mut result = String::new();

        for child in element.children() {
            match child.value() {
                Node::Text(text) => {
                    // Collapse whitespace for text nodes
                    let collapsed = collapse_whitespace(&text.text);
                    // Escape markdown special characters in text
                    let escaped = self.escape_text(&collapsed);
                    result.push_str(&escaped);
                }
                Node::Element(_) => {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        result.push_str(&self.process_element(child_element));
                    }
                }
                _ => {}
            }
        }

        result
    }

    /// Escape markdown special characters in text
    fn escape_text(&self, text: &str) -> String {
        // Escape characters that could be interpreted as markdown
        let mut result = String::with_capacity(text.len());

        for c in text.chars() {
            match c {
                '\\' => result.push_str("\\\\"),
                '*' => result.push_str("\\*"),
                '_' => result.push_str("\\_"),
                '[' => result.push_str("\\["),
                ']' => result.push_str("\\]"),
                '#' => result.push_str("\\#"),
                '+' => result.push_str("\\+"),
                '-' => result.push_str("\\-"),
                '!' => result.push_str("\\!"),
                '`' => result.push_str("\\`"),
                _ => result.push(c),
            }
        }

        result
    }

    /// Process a single element
    fn process_element(&self, element: ElementRef) -> String {
        // Check if should be removed
        if self.rules.should_remove(&element, &self.options) {
            return String::new();
        }

        // Check if should be kept as HTML
        if self.rules.should_keep(&element, &self.options) {
            return self.rules.keep_replacement(&element);
        }

        // Process children first
        let content = self.process_children(element);

        // Apply rule if one matches
        if let Some(rule) = self.rules.for_element(&element, &self.options) {
            return rule.replace(&element, &content, &self.options);
        }

        // Default: return content as-is
        content
    }

    /// Post-process the result
    fn post_process(&self, output: &str) -> String {
        // Trim only leading/trailing newlines, not all whitespace
        // (we need to preserve indentation for code blocks)
        let result = output.trim_matches('\n');

        // Replace multiple consecutive newlines with max 2
        let mut newline_count = 0;
        let mut processed = String::with_capacity(result.len());

        for c in result.chars() {
            if c == '\n' {
                newline_count += 1;
                if newline_count <= 2 {
                    processed.push(c);
                }
            } else {
                newline_count = 0;
                processed.push(c);
            }
        }

        processed
    }
}

impl Default for TurndownService {
    fn default() -> Self {
        Self::new()
    }
}

/// Collapse whitespace in text
fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_whitespace = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_was_whitespace {
                result.push(' ');
                prev_was_whitespace = true;
            }
        } else {
            result.push(c);
            prev_was_whitespace = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_paragraph() {
        let service = TurndownService::new();
        let result = service.turndown("<p>Hello World</p>").unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_heading_setext() {
        let service = TurndownService::new();
        let result = service.turndown("<h1>Title</h1>").unwrap();
        assert!(result.contains("Title"));
        assert!(result.contains("="));
    }

    #[test]
    fn test_heading_atx() {
        let options = TurndownOptions {
            heading_style: HeadingStyle::Atx,
            ..Default::default()
        };
        let service = TurndownService::with_options(options);
        let result = service.turndown("<h1>Title</h1>").unwrap();
        assert!(result.contains("# Title"));
    }

    #[test]
    fn test_emphasis() {
        let service = TurndownService::new();
        let result = service.turndown("<em>emphasized</em>").unwrap();
        assert_eq!(result, "_emphasized_");
    }

    #[test]
    fn test_strong() {
        let service = TurndownService::new();
        let result = service.turndown("<strong>bold</strong>").unwrap();
        assert_eq!(result, "**bold**");
    }

    #[test]
    fn test_inline_link() {
        let service = TurndownService::new();
        let result = service
            .turndown(r#"<a href="https://example.com">Link</a>"#)
            .unwrap();
        assert_eq!(result, "[Link](https://example.com)");
    }

    #[test]
    fn test_image() {
        let service = TurndownService::new();
        let result = service
            .turndown(r#"<img src="test.png" alt="Alt">"#)
            .unwrap();
        assert_eq!(result, "![Alt](test.png)");
    }

    #[test]
    fn test_inline_code() {
        let service = TurndownService::new();
        let result = service.turndown("<code>code</code>").unwrap();
        assert_eq!(result, "`code`");
    }

    #[test]
    fn test_horizontal_rule() {
        let service = TurndownService::new();
        let result = service.turndown("<hr>").unwrap();
        assert!(result.contains("* * *"));
    }

    #[test]
    fn test_blockquote() {
        let service = TurndownService::new();
        let result = service
            .turndown("<blockquote><p>Quote</p></blockquote>")
            .unwrap();
        assert!(result.contains(">"));
    }

    #[test]
    fn test_indented_code_block() {
        let service = TurndownService::new();
        let result = service
            .turndown("<pre><code>function() {}</code></pre>")
            .unwrap();
        assert_eq!(result, "    function() {}");
    }

    #[test]
    fn test_ordered_list() {
        let service = TurndownService::new();
        let result = service
            .turndown("<ol><li>One</li><li>Two</li></ol>")
            .unwrap();
        assert!(result.contains("1.  One"));
        assert!(result.contains("2.  Two"));
    }
}
