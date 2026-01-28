//! TurndownService - the main entry point for Node to Markdown conversion.

use crate::convert::convert;
use crate::node::Node;
use crate::Result;

// Re-export options from core
pub use turndown_core::{
    CodeBlockStyle, HeadingStyle, LinkReferenceStyle, LinkStyle, Options as TurndownOptions,
};

/// The main service for converting DOM nodes to Markdown
pub struct TurndownService {
    options: TurndownOptions,
    keep_filters: Vec<String>,
    remove_filters: Vec<String>,
}

impl TurndownService {
    /// Create a new TurndownService with default options
    pub fn new() -> Self {
        Self {
            options: TurndownOptions::default(),
            keep_filters: Vec::new(),
            remove_filters: Vec::new(),
        }
    }

    /// Create a TurndownService with custom options
    pub fn with_options(options: TurndownOptions) -> Self {
        Self {
            options,
            keep_filters: Vec::new(),
            remove_filters: Vec::new(),
        }
    }

    /// Convert a DOM Node tree to Markdown
    pub fn turndown(&self, node: &Node) -> Result<String> {
        // Convert CDP Node to Markdown AST
        let ast = convert(node, &self.options);

        // Serialize AST to string
        let result = turndown_core::serialize(&ast, &self.options);

        Ok(result)
    }

    /// Get the current options
    pub fn options(&self) -> &TurndownOptions {
        &self.options
    }

    /// Get mutable access to options
    pub fn options_mut(&mut self) -> &mut TurndownOptions {
        &mut self.options
    }

    /// Keep elements matching the filter as HTML
    pub fn keep(&mut self, tag: &str) -> &mut Self {
        self.keep_filters.push(tag.to_lowercase());
        self
    }

    /// Remove elements matching the filter
    pub fn remove(&mut self, tag: &str) -> &mut Self {
        self.remove_filters.push(tag.to_lowercase());
        self
    }

    /// Escape markdown special characters in a string
    pub fn escape(&self, text: &str) -> String {
        escape_markdown(text)
    }
}

impl Default for TurndownService {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape markdown special characters
fn escape_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for c in text.chars() {
        match c {
            '\\' | '*' | '_' | '[' | ']' | '#' | '+' | '-' | '!' | '`' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_p(text: &str) -> Node {
        let mut p = Node::element("p");
        p.add_child(Node::text(text));
        p
    }

    #[test]
    fn test_simple_paragraph() {
        let service = TurndownService::new();
        let node = make_p("Hello World");
        let result = service.turndown(&node).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_heading_setext() {
        let service = TurndownService::new();
        let mut h1 = Node::element("h1");
        h1.add_child(Node::text("Title"));
        let result = service.turndown(&h1).unwrap();
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
        let mut h1 = Node::element("h1");
        h1.add_child(Node::text("Title"));
        let result = service.turndown(&h1).unwrap();
        assert!(result.contains("# Title"));
    }

    #[test]
    fn test_emphasis() {
        let service = TurndownService::new();
        let mut em = Node::element("em");
        em.add_child(Node::text("emphasized"));
        let result = service.turndown(&em).unwrap();
        assert_eq!(result, "_emphasized_");
    }

    #[test]
    fn test_strong() {
        let service = TurndownService::new();
        let mut strong = Node::element("strong");
        strong.add_child(Node::text("bold"));
        let result = service.turndown(&strong).unwrap();
        assert_eq!(result, "**bold**");
    }

    #[test]
    fn test_inline_link() {
        let service = TurndownService::new();
        let mut a = Node::element_with_attrs("a", vec![("href", "https://example.com")]);
        a.add_child(Node::text("Link"));
        let result = service.turndown(&a).unwrap();
        assert_eq!(result, "[Link](https://example.com)");
    }

    #[test]
    fn test_image() {
        let service = TurndownService::new();
        let img = Node::element_with_attrs("img", vec![("src", "test.png"), ("alt", "Alt")]);
        let result = service.turndown(&img).unwrap();
        assert_eq!(result, "![Alt](test.png)");
    }

    #[test]
    fn test_inline_code() {
        let service = TurndownService::new();
        let mut code = Node::element("code");
        code.add_child(Node::text("code"));
        let result = service.turndown(&code).unwrap();
        assert_eq!(result, "`code`");
    }

    #[test]
    fn test_horizontal_rule() {
        let service = TurndownService::new();
        let hr = Node::element("hr");
        let result = service.turndown(&hr).unwrap();
        assert!(result.contains("* * *"));
    }

    #[test]
    fn test_blockquote() {
        let service = TurndownService::new();
        let mut blockquote = Node::element("blockquote");
        let mut p = Node::element("p");
        p.add_child(Node::text("Quote"));
        blockquote.add_child(p);
        let result = service.turndown(&blockquote).unwrap();
        assert!(result.contains(">"));
    }

    #[test]
    fn test_indented_code_block() {
        let service = TurndownService::new();
        let mut pre = Node::element("pre");
        let mut code = Node::element("code");
        code.add_child(Node::text("function() {}"));
        pre.add_child(code);
        let result = service.turndown(&pre).unwrap();
        assert_eq!(result, "    function() {}");
    }

    #[test]
    fn test_ordered_list() {
        let service = TurndownService::new();
        let mut ol = Node::element("ol");
        let mut li1 = Node::element("li");
        li1.add_child(Node::text("One"));
        let mut li2 = Node::element("li");
        li2.add_child(Node::text("Two"));
        ol.add_child(li1);
        ol.add_child(li2);
        let result = service.turndown(&ol).unwrap();
        assert!(result.contains("1.  One"));
        assert!(result.contains("2.  Two"));
    }

    #[test]
    fn test_unordered_list() {
        let service = TurndownService::new();
        let mut ul = Node::element("ul");
        let mut li1 = Node::element("li");
        li1.add_child(Node::text("One"));
        let mut li2 = Node::element("li");
        li2.add_child(Node::text("Two"));
        ul.add_child(li1);
        ul.add_child(li2);
        let result = service.turndown(&ul).unwrap();
        assert!(result.contains("*   One"));
        assert!(result.contains("*   Two"));
    }
}
