//! HTML parsing support.
//!
//! This module provides functionality to parse HTML strings and convert them
//! to the CDP-style Node structure used by turndown.

use scraper::{ElementRef, Html, Node as ScraperNode};

use crate::node::Node;

/// Parse an HTML string into a Node tree.
///
/// This is useful when you need to manipulate the DOM tree before
/// converting to Markdown, or when integrating with other tools.
///
/// # Example
///
/// ```rust
/// use turndown::{parse_html, TurndownService};
///
/// // Parse HTML to a Node tree
/// let node = parse_html("<h1>Hello <em>World</em></h1>");
///
/// // Convert to Markdown
/// let service = TurndownService::new();
/// let markdown = service.turndown(&node).unwrap();
/// ```
pub fn parse_html(html: &str) -> Node {
    let document = Html::parse_fragment(html);
    scraper_to_node(document.root_element())
}

/// Convert a scraper ElementRef to our Node structure
fn scraper_to_node(element: ElementRef) -> Node {
    let tag = element.value().name();

    // Collect attributes
    let attrs: Vec<(&str, &str)> = element.value().attrs().collect();

    let mut node = if attrs.is_empty() {
        Node::element(tag)
    } else {
        Node::element_with_attrs(tag, attrs)
    };

    // Process children
    for child in element.children() {
        match child.value() {
            ScraperNode::Text(text) => {
                node.add_child(Node::text(&text.text));
            }
            ScraperNode::Element(_) => {
                if let Some(child_element) = ElementRef::wrap(child) {
                    node.add_child(scraper_to_node(child_element));
                }
            }
            _ => {}
        }
    }

    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TurndownService;

    #[test]
    fn test_parse_simple_html() {
        let node = parse_html("<p>Hello World</p>");
        assert!(node.is_element());
        assert_eq!(node.tag_name(), "html");
    }

    #[test]
    fn test_turndown_html() {
        let service = TurndownService::new();
        let result = service.turndown_html("<p>Hello World</p>").unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_turndown_html_with_formatting() {
        let service = TurndownService::new();
        let result = service
            .turndown_html("<p>Hello <strong>World</strong></p>")
            .unwrap();
        assert_eq!(result, "Hello **World**");
    }

    #[test]
    fn test_turndown_html_heading() {
        let service = TurndownService::new();
        let result = service.turndown_html("<h1>Title</h1>").unwrap();
        assert!(result.contains("Title"));
        assert!(result.contains("="));
    }
}
