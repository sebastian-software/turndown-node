//! TurndownService - the main entry point for Node to Markdown conversion.

use crate::node::{Node, NodeRef, NodeType};
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

/// The main service for converting DOM nodes to Markdown
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

    /// Convert a DOM Node tree to Markdown
    pub fn turndown(&self, node: &Node) -> Result<String> {
        // Process the node tree
        let result = self.process_node(node, None);

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

    /// Process a node and its children
    fn process_node(&self, node: &Node, parent_tag: Option<&str>) -> String {
        match node.node_type {
            NodeType::Text => {
                // Collapse whitespace for text nodes
                let text = node.node_value.as_deref().unwrap_or("");
                let collapsed = collapse_whitespace(text);
                // Escape markdown special characters in text
                self.escape_text(&collapsed)
            }
            NodeType::Element => {
                self.process_element(node, parent_tag)
            }
            NodeType::Document | NodeType::DocumentFragment => {
                self.process_children(node, parent_tag)
            }
            _ => String::new(),
        }
    }

    /// Process children of a node
    fn process_children(&self, node: &Node, parent_tag: Option<&str>) -> String {
        let tag = if node.is_element() {
            Some(node.tag_name())
        } else {
            None
        };
        let parent = tag.as_deref().or(parent_tag);

        // Special handling for ordered lists - track item index
        if node.is_element() && node.tag_name() == "ol" {
            return self.process_ordered_list(node, parent);
        }

        let mut result = String::new();

        for child in node.children() {
            result.push_str(&self.process_node(child, parent));
        }

        result
    }

    /// Process an ordered list with proper item numbering
    fn process_ordered_list(&self, node: &Node, parent_tag: Option<&str>) -> String {
        let mut result = String::new();
        let mut index = 1;

        for child in node.children() {
            if child.is_element() && child.tag_name() == "li" {
                let content = self.process_children(child, Some("ol"));
                let content = content
                    .trim()
                    .replace("\n\n\n", "\n\n")
                    .replace('\n', "\n    ");

                result.push_str(&format!("{}.  {}\n", index, content));
                index += 1;
            } else {
                result.push_str(&self.process_node(child, parent_tag));
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
    fn process_element(&self, node: &Node, parent_tag: Option<&str>) -> String {
        let node_ref = if let Some(parent) = parent_tag {
            NodeRef::with_parent(node, parent)
        } else {
            NodeRef::new(node)
        };

        // Check if should be removed
        if self.rules.should_remove(&node_ref, &self.options) {
            return String::new();
        }

        // Check if should be kept as HTML
        if self.rules.should_keep(&node_ref, &self.options) {
            return self.rules.keep_replacement(&node_ref);
        }

        // Special handling for ordered list items is done in process_ordered_list
        // For other elements, process children first
        let content = self.process_children(node, parent_tag);

        // Apply rule if one matches
        if let Some(rule) = self.rules.for_node(&node_ref, &self.options) {
            return rule.replace(&node_ref, &content, &self.options);
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
