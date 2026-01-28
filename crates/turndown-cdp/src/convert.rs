//! Convert CDP Node tree to Markdown AST
//!
//! This module transforms a CDP-style DOM tree into the Markdown AST
//! defined in turndown-core.

use crate::node::{Node, NodeType};
use turndown_core::{Block, Inline, ListItem, Options};

/// Convert a CDP Node tree to a Markdown AST Block
pub fn convert(node: &Node, options: &Options) -> Block {
    let ctx = Context::default();

    // If the root node is itself an element, convert it directly
    if node.is_element() {
        if let Some(block) = convert_element(node, options, &ctx) {
            return flatten_document(block);
        }
    }

    // Otherwise, convert children
    let blocks = convert_children(node, options, &ctx);
    Block::Document(blocks)
}

/// Flatten nested documents
fn flatten_document(block: Block) -> Block {
    match block {
        Block::Document(blocks) if blocks.len() == 1 => {
            flatten_document(blocks.into_iter().next().unwrap())
        }
        other => other,
    }
}

/// Context for conversion (tracks parent elements)
#[derive(Default, Clone)]
struct Context {
    in_pre: bool,
}

/// Convert children of a node to blocks
fn convert_children(node: &Node, options: &Options, ctx: &Context) -> Vec<Block> {
    let mut blocks = Vec::new();

    for child in node.children() {
        match child.node_type {
            NodeType::Text => {
                // Text at block level gets wrapped in paragraph if non-empty
                let text = child.node_value.as_deref().unwrap_or("");
                if !text.trim().is_empty() && !ctx.in_pre {
                    let inlines = vec![Inline::Text(escape_markdown(&collapse_whitespace(text)))];
                    blocks.push(Block::Paragraph(inlines));
                }
            }
            NodeType::Element => {
                if let Some(block) = convert_element(child, options, ctx) {
                    blocks.push(block);
                }
            }
            _ => {}
        }
    }

    blocks
}

/// Convert an element node to a Block
fn convert_element(node: &Node, options: &Options, ctx: &Context) -> Option<Block> {
    let tag = node.tag_name();

    match tag.as_str() {
        // Block elements
        "p" => {
            let inlines = collect_inlines(node, options, ctx);
            if inlines_are_blank(&inlines) {
                None
            } else {
                Some(Block::Paragraph(inlines))
            }
        }

        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag.chars().nth(1)?.to_digit(10)? as u8;
            let inlines = collect_inlines(node, options, ctx);
            if inlines_are_blank(&inlines) {
                None
            } else {
                Some(Block::Heading {
                    level,
                    content: inlines,
                })
            }
        }

        "blockquote" => {
            let blocks = convert_children(node, options, ctx);
            if blocks.is_empty() {
                None
            } else {
                Some(Block::BlockQuote(blocks))
            }
        }

        "ul" => {
            let items = collect_list_items(node, options, ctx);
            if items.is_empty() {
                None
            } else {
                Some(Block::List {
                    ordered: false,
                    start: 1,
                    items,
                })
            }
        }

        "ol" => {
            let start = node
                .attr("start")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            let items = collect_list_items(node, options, ctx);
            if items.is_empty() {
                None
            } else {
                Some(Block::List {
                    ordered: true,
                    start,
                    items,
                })
            }
        }

        "pre" => {
            // Look for <code> child
            let code_node = node.element_children().find(|c| c.tag_name() == "code");

            if let Some(code) = code_node {
                let code_text = code.text_content();
                let language = code
                    .attr("class")
                    .and_then(|c| {
                        c.split_whitespace()
                            .find(|s| s.starts_with("language-"))
                            .map(|s| s[9..].to_string())
                    });

                let fenced = matches!(
                    options.code_block_style,
                    turndown_core::CodeBlockStyle::Fenced
                );

                Some(Block::CodeBlock {
                    language,
                    code: code_text,
                    fenced,
                })
            } else {
                // Pre without code
                let text = node.text_content();
                Some(Block::CodeBlock {
                    language: None,
                    code: text,
                    fenced: false,
                })
            }
        }

        "hr" => Some(Block::ThematicBreak),

        "table" => convert_table(node, options, ctx),

        // Container elements - just process children
        "div" | "section" | "article" | "main" | "aside" | "header" | "footer" | "nav"
        | "figure" | "figcaption" | "address" | "form" | "fieldset" => {
            let blocks = convert_children(node, options, ctx);
            // Return as document fragment (will be flattened)
            if blocks.len() == 1 {
                Some(blocks.into_iter().next().unwrap())
            } else if blocks.is_empty() {
                None
            } else {
                Some(Block::Document(blocks))
            }
        }

        // Inline-only elements at block level - convert as inline and wrap in paragraph
        "a" | "strong" | "b" | "em" | "i" | "code" | "span" | "img" | "br" => {
            if let Some(inline) = convert_inline_element(node, options, ctx) {
                Some(Block::Paragraph(vec![inline]))
            } else {
                None
            }
        }

        // Skip these elements
        "script" | "style" | "noscript" | "template" => None,

        // Unknown elements - try to get content
        _ => {
            let blocks = convert_children(node, options, ctx);
            if blocks.is_empty() {
                // Try as inline
                let inlines = collect_inlines(node, options, ctx);
                if inlines_are_blank(&inlines) {
                    None
                } else {
                    Some(Block::Paragraph(inlines))
                }
            } else if blocks.len() == 1 {
                Some(blocks.into_iter().next().unwrap())
            } else {
                Some(Block::Document(blocks))
            }
        }
    }
}

/// Collect list items from ul/ol
fn collect_list_items(node: &Node, options: &Options, ctx: &Context) -> Vec<ListItem> {
    let mut items = Vec::new();

    for child in node.children() {
        if child.is_element() && child.tag_name() == "li" {
            let blocks = convert_children(child, options, ctx);
            items.push(ListItem::new(if blocks.is_empty() {
                // Try getting inline content
                let inlines = collect_inlines(child, options, ctx);
                vec![Block::Paragraph(inlines)]
            } else {
                blocks
            }));
        }
    }

    items
}

/// Convert a table element
fn convert_table(node: &Node, options: &Options, ctx: &Context) -> Option<Block> {
    let mut headers: Vec<Vec<Inline>> = Vec::new();
    let mut rows: Vec<Vec<Vec<Inline>>> = Vec::new();

    // Find thead and tbody
    for child in node.children() {
        if !child.is_element() {
            continue;
        }

        match child.tag_name().as_str() {
            "thead" => {
                // Get header row
                for tr in child.element_children() {
                    if tr.tag_name() == "tr" {
                        for th in tr.element_children() {
                            if th.tag_name() == "th" || th.tag_name() == "td" {
                                headers.push(collect_inlines(th, options, ctx));
                            }
                        }
                        break; // Only first row as headers
                    }
                }
            }
            "tbody" => {
                for tr in child.element_children() {
                    if tr.tag_name() == "tr" {
                        let mut row = Vec::new();
                        for td in tr.element_children() {
                            if td.tag_name() == "td" || td.tag_name() == "th" {
                                row.push(collect_inlines(td, options, ctx));
                            }
                        }
                        if !row.is_empty() {
                            rows.push(row);
                        }
                    }
                }
            }
            "tr" => {
                // Direct tr children (no thead/tbody)
                let mut row = Vec::new();
                let mut is_header = false;

                for cell in child.element_children() {
                    let tag = cell.tag_name();
                    if tag == "th" {
                        is_header = true;
                        row.push(collect_inlines(cell, options, ctx));
                    } else if tag == "td" {
                        row.push(collect_inlines(cell, options, ctx));
                    }
                }

                if !row.is_empty() {
                    if is_header && headers.is_empty() {
                        headers = row;
                    } else {
                        rows.push(row);
                    }
                }
            }
            _ => {}
        }
    }

    if headers.is_empty() && rows.is_empty() {
        return None;
    }

    // If no headers, use first row as headers
    if headers.is_empty() && !rows.is_empty() {
        headers = rows.remove(0);
    }

    Some(Block::Table { headers, rows })
}

/// Collect inline content from a node
fn collect_inlines(node: &Node, options: &Options, ctx: &Context) -> Vec<Inline> {
    let mut inlines = Vec::new();

    for child in node.children() {
        match child.node_type {
            NodeType::Text => {
                let text = child.node_value.as_deref().unwrap_or("");
                if ctx.in_pre {
                    inlines.push(Inline::Text(text.to_string()));
                } else {
                    let collapsed = collapse_whitespace(text);
                    if !collapsed.is_empty() {
                        inlines.push(Inline::Text(escape_markdown(&collapsed)));
                    }
                }
            }
            NodeType::Element => {
                if let Some(inline) = convert_inline_element(child, options, ctx) {
                    inlines.push(inline);
                }
            }
            _ => {}
        }
    }

    inlines
}

/// Convert an inline element to an Inline node
fn convert_inline_element(node: &Node, options: &Options, ctx: &Context) -> Option<Inline> {
    let tag = node.tag_name();

    match tag.as_str() {
        "strong" | "b" => {
            let inner = collect_inlines(node, options, ctx);
            if inlines_are_blank(&inner) {
                None
            } else {
                Some(Inline::Strong(inner))
            }
        }

        "em" | "i" => {
            let inner = collect_inlines(node, options, ctx);
            if inlines_are_blank(&inner) {
                None
            } else {
                Some(Inline::Emphasis(inner))
            }
        }

        "code" => {
            let text = node.text_content();
            if text.is_empty() {
                None
            } else {
                Some(Inline::Code(text))
            }
        }

        "a" => {
            let href = node.attr("href").unwrap_or("");
            let title = node.attr("title").map(|s| s.to_string());
            let content = collect_inlines(node, options, ctx);

            if href.is_empty() && title.is_none() {
                // No link target, just return content
                if content.len() == 1 {
                    return Some(content.into_iter().next().unwrap());
                }
                return None;
            }

            Some(Inline::Link {
                content,
                url: href.to_string(),
                title,
            })
        }

        "img" => {
            let src = node.attr("src").unwrap_or("");
            if src.is_empty() {
                return None;
            }

            let alt = node.attr("alt").unwrap_or("").to_string();
            let title = node.attr("title").map(|s| s.to_string());

            Some(Inline::Image {
                alt,
                url: src.to_string(),
                title,
            })
        }

        "br" => Some(Inline::LineBreak),

        "span" | "small" | "mark" | "abbr" | "cite" | "q" | "sub" | "sup" | "time" => {
            // Pass-through inline containers
            let inner = collect_inlines(node, options, ctx);
            if inner.len() == 1 {
                Some(inner.into_iter().next().unwrap())
            } else if inner.is_empty() {
                None
            } else {
                // Flatten multiple inlines
                Some(Inline::Text(
                    inner
                        .iter()
                        .map(|i| inline_to_text(i))
                        .collect::<Vec<_>>()
                        .join(""),
                ))
            }
        }

        // Nested block elements inside inline context - extract text
        "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let text = node.text_content();
            if text.trim().is_empty() {
                None
            } else {
                Some(Inline::Text(escape_markdown(&collapse_whitespace(&text))))
            }
        }

        _ => {
            // Unknown inline - try to get content
            let inner = collect_inlines(node, options, ctx);
            if inner.len() == 1 {
                Some(inner.into_iter().next().unwrap())
            } else if inner.is_empty() {
                None
            } else {
                Some(Inline::Text(
                    inner
                        .iter()
                        .map(|i| inline_to_text(i))
                        .collect::<Vec<_>>()
                        .join(""),
                ))
            }
        }
    }
}

/// Get plain text from an inline (for flattening)
fn inline_to_text(inline: &Inline) -> String {
    match inline {
        Inline::Text(t) => t.clone(),
        Inline::Strong(inner) | Inline::Emphasis(inner) => {
            inner.iter().map(|i| inline_to_text(i)).collect()
        }
        Inline::Code(c) => c.clone(),
        Inline::Link { content, .. } => content.iter().map(|i| inline_to_text(i)).collect(),
        Inline::Image { alt, .. } => alt.clone(),
        Inline::LineBreak => "\n".to_string(),
        Inline::HtmlInline(h) => h.clone(),
    }
}

/// Check if inlines are all blank
fn inlines_are_blank(inlines: &[Inline]) -> bool {
    inlines.iter().all(|i| i.is_blank())
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

/// Escape markdown special characters in text
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
    use turndown_core::serialize;

    fn convert_and_serialize(node: &Node) -> String {
        let options = Options::default();
        let ast = convert(node, &options);
        serialize(&ast, &options)
    }

    #[test]
    fn test_paragraph() {
        let mut p = Node::element("p");
        p.add_child(Node::text("Hello World"));
        let result = convert_and_serialize(&p);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_heading() {
        let mut h1 = Node::element("h1");
        h1.add_child(Node::text("Title"));
        let result = convert_and_serialize(&h1);
        assert!(result.contains("Title"));
        assert!(result.contains("="));
    }

    #[test]
    fn test_strong() {
        let mut p = Node::element("p");
        let mut strong = Node::element("strong");
        strong.add_child(Node::text("bold"));
        p.add_child(strong);
        let result = convert_and_serialize(&p);
        assert_eq!(result, "**bold**");
    }

    #[test]
    fn test_link() {
        let mut a = Node::element_with_attrs("a", vec![("href", "https://example.com")]);
        a.add_child(Node::text("Link"));
        let result = convert_and_serialize(&a);
        assert_eq!(result, "[Link](https://example.com)");
    }

    #[test]
    fn test_image() {
        let img = Node::element_with_attrs("img", vec![("src", "test.png"), ("alt", "Alt")]);
        let result = convert_and_serialize(&img);
        assert_eq!(result, "![Alt](test.png)");
    }

    #[test]
    fn test_code_block() {
        let mut pre = Node::element("pre");
        let mut code = Node::element("code");
        code.add_child(Node::text("let x = 1;"));
        pre.add_child(code);
        let result = convert_and_serialize(&pre);
        assert_eq!(result, "    let x = 1;");
    }

    #[test]
    fn test_list() {
        let mut ul = Node::element("ul");
        let mut li1 = Node::element("li");
        li1.add_child(Node::text("One"));
        let mut li2 = Node::element("li");
        li2.add_child(Node::text("Two"));
        ul.add_child(li1);
        ul.add_child(li2);
        let result = convert_and_serialize(&ul);
        assert!(result.contains("*   One"));
        assert!(result.contains("*   Two"));
    }
}
