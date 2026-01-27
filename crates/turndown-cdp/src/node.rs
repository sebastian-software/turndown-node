//! CDP-style DOM Node structure for HTML to Markdown conversion.
//!
//! This module provides a DOM node structure that matches the Chrome DevTools Protocol
//! DOM.Node structure. Any parser (html5ever, CDP, etc.) can convert their output to
//! this structure to use turndown.

/// Node types matching DOM nodeType values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Element node (nodeType = 1)
    Element = 1,
    /// Text node (nodeType = 3)
    Text = 3,
    /// Comment node (nodeType = 8)
    Comment = 8,
    /// Document node (nodeType = 9)
    Document = 9,
    /// Document fragment node (nodeType = 11)
    DocumentFragment = 11,
}

impl From<u32> for NodeType {
    fn from(value: u32) -> Self {
        match value {
            1 => NodeType::Element,
            3 => NodeType::Text,
            8 => NodeType::Comment,
            9 => NodeType::Document,
            11 => NodeType::DocumentFragment,
            _ => NodeType::Element, // Default fallback
        }
    }
}

/// A DOM node following the CDP DOM.Node structure.
///
/// This structure is designed to be compatible with Chrome DevTools Protocol
/// and can be used as a common interface for any HTML parser.
#[derive(Debug, Clone)]
pub struct Node {
    /// Node type (1 = Element, 3 = Text, etc.)
    pub node_type: NodeType,

    /// Node name (uppercase for elements, e.g., "DIV", "#text" for text nodes)
    pub node_name: String,

    /// Text content for text nodes
    pub node_value: Option<String>,

    /// Attributes as flat array [name, value, name, value, ...] (CDP style)
    /// Only present for element nodes
    pub attributes: Option<Vec<String>>,

    /// Child nodes
    pub children: Option<Vec<Node>>,
}

impl Node {
    /// Create a new element node
    pub fn element(tag_name: &str) -> Self {
        Self {
            node_type: NodeType::Element,
            node_name: tag_name.to_uppercase(),
            node_value: None,
            attributes: Some(Vec::new()),
            children: Some(Vec::new()),
        }
    }

    /// Create a new element node with attributes
    pub fn element_with_attrs(tag_name: &str, attrs: Vec<(&str, &str)>) -> Self {
        let flat_attrs: Vec<String> = attrs
            .into_iter()
            .flat_map(|(k, v)| vec![k.to_string(), v.to_string()])
            .collect();

        Self {
            node_type: NodeType::Element,
            node_name: tag_name.to_uppercase(),
            node_value: None,
            attributes: Some(flat_attrs),
            children: Some(Vec::new()),
        }
    }

    /// Create a new text node
    pub fn text(content: &str) -> Self {
        Self {
            node_type: NodeType::Text,
            node_name: "#text".to_string(),
            node_value: Some(content.to_string()),
            attributes: None,
            children: None,
        }
    }

    /// Create a document fragment node
    pub fn document_fragment() -> Self {
        Self {
            node_type: NodeType::DocumentFragment,
            node_name: "#document-fragment".to_string(),
            node_value: None,
            attributes: None,
            children: Some(Vec::new()),
        }
    }

    /// Check if this is an element node
    pub fn is_element(&self) -> bool {
        self.node_type == NodeType::Element
    }

    /// Check if this is a text node
    pub fn is_text(&self) -> bool {
        self.node_type == NodeType::Text
    }

    /// Get the tag name (lowercase)
    pub fn tag_name(&self) -> String {
        self.node_name.to_lowercase()
    }

    /// Get an attribute value by name
    pub fn attr(&self, name: &str) -> Option<&str> {
        let attrs = self.attributes.as_ref()?;
        let name_lower = name.to_lowercase();

        // CDP stores attributes as flat array: [name, value, name, value, ...]
        let mut iter = attrs.iter();
        while let Some(attr_name) = iter.next() {
            if let Some(attr_value) = iter.next() {
                if attr_name.to_lowercase() == name_lower {
                    return Some(attr_value.as_str());
                }
            }
        }
        None
    }

    /// Check if an attribute exists
    pub fn has_attr(&self, name: &str) -> bool {
        self.attr(name).is_some()
    }

    /// Get all child nodes
    pub fn children(&self) -> impl Iterator<Item = &Node> {
        self.children.iter().flat_map(|c| c.iter())
    }

    /// Get only element children
    pub fn element_children(&self) -> impl Iterator<Item = &Node> {
        self.children().filter(|n| n.is_element())
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Node) {
        if let Some(ref mut children) = self.children {
            children.push(child);
        } else {
            self.children = Some(vec![child]);
        }
    }

    /// Set an attribute
    pub fn set_attr(&mut self, name: &str, value: &str) {
        if self.attributes.is_none() {
            self.attributes = Some(Vec::new());
        }

        if let Some(ref mut attrs) = self.attributes {
            // Check if attribute already exists
            let name_lower = name.to_lowercase();
            let mut i = 0;
            while i + 1 < attrs.len() {
                if attrs[i].to_lowercase() == name_lower {
                    attrs[i + 1] = value.to_string();
                    return;
                }
                i += 2;
            }
            // Add new attribute
            attrs.push(name.to_string());
            attrs.push(value.to_string());
        }
    }

    /// Get all text content from this node and descendants
    pub fn text_content(&self) -> String {
        match self.node_type {
            NodeType::Text => self.node_value.clone().unwrap_or_default(),
            _ => self
                .children()
                .map(|child| child.text_content())
                .collect::<Vec<_>>()
                .join(""),
        }
    }

    /// Reconstruct outer HTML (for keep rules)
    pub fn outer_html(&self) -> String {
        match self.node_type {
            NodeType::Text => self.node_value.clone().unwrap_or_default(),
            NodeType::Element => {
                let tag = self.tag_name();
                let attrs = self.attributes_string();

                if self.is_void_element() {
                    if attrs.is_empty() {
                        format!("<{}>", tag)
                    } else {
                        format!("<{} {}>", tag, attrs)
                    }
                } else {
                    let inner = self.inner_html();
                    if attrs.is_empty() {
                        format!("<{}>{}</{}>", tag, inner, tag)
                    } else {
                        format!("<{} {}>{}</{}>", tag, attrs, inner, tag)
                    }
                }
            }
            _ => self.inner_html(),
        }
    }

    /// Reconstruct inner HTML
    pub fn inner_html(&self) -> String {
        self.children()
            .map(|child| child.outer_html())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Get attributes as a string for HTML output
    fn attributes_string(&self) -> String {
        let Some(ref attrs) = self.attributes else {
            return String::new();
        };

        let mut result = Vec::new();
        let mut iter = attrs.iter();
        while let Some(name) = iter.next() {
            if let Some(value) = iter.next() {
                if value.is_empty() {
                    result.push(name.clone());
                } else {
                    result.push(format!("{}=\"{}\"", name, escape_html_attr(value)));
                }
            }
        }
        result.join(" ")
    }

    /// Check if this is a void element
    fn is_void_element(&self) -> bool {
        const VOID_ELEMENTS: &[&str] = &[
            "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen",
            "link", "meta", "param", "source", "track", "wbr",
        ];
        VOID_ELEMENTS.contains(&self.tag_name().as_str())
    }
}

/// Escape HTML attribute value
fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// A reference to a node with parent context.
/// This allows navigation up the tree without storing parent pointers.
#[derive(Debug, Clone)]
pub struct NodeRef<'a> {
    /// The node itself
    pub node: &'a Node,
    /// Index path from root (for sibling/parent lookup)
    parent_tag: Option<&'a str>,
}

impl<'a> NodeRef<'a> {
    /// Create a new NodeRef without parent context
    pub fn new(node: &'a Node) -> Self {
        Self {
            node,
            parent_tag: None,
        }
    }

    /// Create a new NodeRef with parent tag context
    pub fn with_parent(node: &'a Node, parent_tag: &'a str) -> Self {
        Self {
            node,
            parent_tag: Some(parent_tag),
        }
    }

    /// Get the parent tag name if known
    pub fn parent_tag(&self) -> Option<&str> {
        self.parent_tag
    }

    /// Delegate to Node methods
    pub fn is_element(&self) -> bool {
        self.node.is_element()
    }

    pub fn is_text(&self) -> bool {
        self.node.is_text()
    }

    pub fn tag_name(&self) -> String {
        self.node.tag_name()
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.node.attr(name)
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.node.has_attr(name)
    }

    pub fn children(&self) -> impl Iterator<Item = &Node> {
        self.node.children()
    }

    pub fn element_children(&self) -> impl Iterator<Item = &Node> {
        self.node.element_children()
    }

    pub fn text_content(&self) -> String {
        self.node.text_content()
    }

    pub fn outer_html(&self) -> String {
        self.node.outer_html()
    }

    pub fn inner_html(&self) -> String {
        self.node.inner_html()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_element() {
        let node = Node::element("div");
        assert!(node.is_element());
        assert_eq!(node.tag_name(), "div");
        assert_eq!(node.node_name, "DIV");
    }

    #[test]
    fn test_create_text() {
        let node = Node::text("Hello World");
        assert!(node.is_text());
        assert_eq!(node.text_content(), "Hello World");
    }

    #[test]
    fn test_attributes() {
        let node = Node::element_with_attrs("a", vec![("href", "https://example.com"), ("title", "Example")]);
        assert_eq!(node.attr("href"), Some("https://example.com"));
        assert_eq!(node.attr("title"), Some("Example"));
        assert_eq!(node.attr("class"), None);
    }

    #[test]
    fn test_children() {
        let mut parent = Node::element("div");
        parent.add_child(Node::text("Hello"));
        parent.add_child(Node::element("span"));
        parent.add_child(Node::text("World"));

        assert_eq!(parent.children().count(), 3);
        assert_eq!(parent.element_children().count(), 1);
    }

    #[test]
    fn test_text_content() {
        let mut div = Node::element("div");
        div.add_child(Node::text("Hello "));
        let mut span = Node::element("span");
        span.add_child(Node::text("World"));
        div.add_child(span);

        assert_eq!(div.text_content(), "Hello World");
    }

    #[test]
    fn test_outer_html() {
        let mut a = Node::element_with_attrs("a", vec![("href", "https://example.com")]);
        a.add_child(Node::text("Link"));

        assert_eq!(a.outer_html(), "<a href=\"https://example.com\">Link</a>");
    }

    #[test]
    fn test_void_element_html() {
        let br = Node::element("br");
        assert_eq!(br.outer_html(), "<br>");

        let img = Node::element_with_attrs("img", vec![("src", "test.png"), ("alt", "Test")]);
        assert_eq!(img.outer_html(), "<img src=\"test.png\" alt=\"Test\">");
    }
}
