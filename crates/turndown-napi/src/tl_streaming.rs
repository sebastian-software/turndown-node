//! HTML to Markdown AST conversion using tl parser
//!
//! Uses tl for fast DOM parsing with DOM traversal for AST building.

use smallvec::SmallVec;
use tl::{HTMLTag, Node, NodeHandle, Parser, ParserOptions, VDom};
use turndown_core::{Block, Inline, ListItem, Options};

// Most inline elements have few children - avoid heap allocation
type InlineVec = SmallVec<[Inline; 4]>;

/// Convert HTML string to Markdown AST using tl parser
pub fn html_to_ast(html: &str, _options: &Options) -> Block {
    let dom = tl::parse(html, ParserOptions::default()).expect("HTML parse error");
    let parser = dom.parser();

    let children = dom.children();
    let blocks = process_nodes(&dom, parser, children);

    if blocks.is_empty() {
        Block::Document(vec![])
    } else if blocks.len() == 1 {
        blocks.into_iter().next().unwrap()
    } else {
        Block::Document(blocks)
    }
}

fn process_nodes(dom: &VDom, parser: &Parser, handles: &[NodeHandle]) -> Vec<Block> {
    let mut blocks = Vec::new();

    for handle in handles {
        if let Some(node) = handle.get(parser) {
            match node {
                Node::Tag(tag) => {
                    if let Some(block) = process_element(dom, parser, tag) {
                        blocks.push(block);
                    }
                }
                Node::Raw(text) => {
                    let text_str = text.as_utf8_str();
                    if !text_str.trim().is_empty() {
                        // Text at root level becomes paragraph
                        let processed = collapse_and_escape(&text_str);
                        if !processed.trim().is_empty() {
                            blocks.push(Block::Paragraph(vec![Inline::Text(processed)]));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    blocks
}

fn process_element(dom: &VDom, parser: &Parser, tag: &HTMLTag) -> Option<Block> {
    let tag_name = tag.name().as_utf8_str();
    let tag_lower = tag_name.to_ascii_lowercase();

    match tag_lower.as_str() {
        "p" => {
            let inlines = collect_inlines(dom, parser, tag);
            if inlines.is_empty() {
                None
            } else {
                Some(Block::Paragraph(inlines))
            }
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag_lower.chars().nth(1).and_then(|c| c.to_digit(10)).unwrap_or(1) as u8;
            let inlines = collect_inlines(dom, parser, tag);
            if inlines.is_empty() {
                None
            } else {
                Some(Block::Heading { level, content: inlines })
            }
        }
        "blockquote" => {
            let children = tag.children();
            let inner_blocks = process_nodes(dom, parser, children.top().as_slice());
            if inner_blocks.is_empty() {
                // Try to get text content directly
                let inlines = collect_inlines(dom, parser, tag);
                if inlines.is_empty() {
                    None
                } else {
                    Some(Block::BlockQuote(vec![Block::Paragraph(inlines)]))
                }
            } else {
                Some(Block::BlockQuote(inner_blocks))
            }
        }
        "ul" | "ol" => {
            let ordered = tag_lower == "ol";
            let start = tag.attributes()
                .get("start")
                .flatten()
                .and_then(|s| s.as_utf8_str().parse().ok())
                .unwrap_or(1);

            let items = collect_list_items(dom, parser, tag);
            if items.is_empty() {
                None
            } else {
                Some(Block::List { ordered, start, items })
            }
        }
        "pre" => {
            // Look for code element inside
            let (code, lang) = extract_code_content(dom, parser, tag);
            Some(Block::CodeBlock {
                language: lang,
                code,
                fenced: true,
            })
        }
        "hr" => Some(Block::ThematicBreak),
        "table" => {
            let (headers, rows) = collect_table(dom, parser, tag);
            if headers.is_empty() && rows.is_empty() {
                None
            } else {
                Some(Block::Table { headers, rows })
            }
        }
        "div" | "section" | "article" | "main" | "aside" | "header" | "footer" | "nav" | "figure" | "body" | "html" => {
            // Container elements - process children
            let children = tag.children();
            let inner_blocks = process_nodes(dom, parser, children.top().as_slice());
            match inner_blocks.len() {
                0 => {
                    // Maybe just text content?
                    let inlines = collect_inlines(dom, parser, tag);
                    if inlines.is_empty() {
                        None
                    } else {
                        Some(Block::Paragraph(inlines))
                    }
                }
                1 => Some(inner_blocks.into_iter().next().unwrap()),
                _ => Some(Block::Document(inner_blocks)),
            }
        }
        "a" => {
            // Standalone link at block level
            let inlines = vec![process_link(dom, parser, tag)];
            Some(Block::Paragraph(inlines))
        }
        "img" => {
            // Standalone image at block level
            if let Some(inline) = process_image(tag) {
                Some(Block::Paragraph(vec![inline]))
            } else {
                None
            }
        }
        "script" | "style" | "noscript" | "template" | "head" | "meta" | "link" => None,
        _ => {
            // Unknown element - try to extract content
            let children = tag.children();
            let inner_blocks = process_nodes(dom, parser, children.top().as_slice());
            if !inner_blocks.is_empty() {
                if inner_blocks.len() == 1 {
                    Some(inner_blocks.into_iter().next().unwrap())
                } else {
                    Some(Block::Document(inner_blocks))
                }
            } else {
                let inlines = collect_inlines(dom, parser, tag);
                if inlines.is_empty() {
                    None
                } else {
                    Some(Block::Paragraph(inlines))
                }
            }
        }
    }
}

fn collect_inlines(dom: &VDom, parser: &Parser, tag: &HTMLTag) -> Vec<Inline> {
    let mut inlines = InlineVec::new();
    let children = tag.children();

    for handle in children.top().iter() {
        if let Some(node) = handle.get(parser) {
            collect_inline_node(dom, parser, node, &mut inlines);
        }
    }

    inlines.into_vec()
}

fn collect_inline_node(dom: &VDom, parser: &Parser, node: &Node, inlines: &mut InlineVec) {
    match node {
        Node::Tag(tag) => {
            let tag_name = tag.name().as_utf8_str();
            let tag_lower = tag_name.to_ascii_lowercase();

            match tag_lower.as_str() {
                "strong" | "b" => {
                    let inner = collect_inlines(dom, parser, tag);
                    if !inner.is_empty() {
                        inlines.push(Inline::Strong(inner));
                    }
                }
                "em" | "i" => {
                    let inner = collect_inlines(dom, parser, tag);
                    if !inner.is_empty() {
                        inlines.push(Inline::Emphasis(inner));
                    }
                }
                "code" => {
                    let code = get_text_content(dom, parser, tag);
                    if !code.is_empty() {
                        inlines.push(Inline::Code(code));
                    }
                }
                "a" => {
                    inlines.push(process_link(dom, parser, tag));
                }
                "img" => {
                    if let Some(img) = process_image(tag) {
                        inlines.push(img);
                    }
                }
                "br" => {
                    inlines.push(Inline::LineBreak);
                }
                "span" | "small" | "sub" | "sup" | "mark" | "del" | "ins" | "u" => {
                    // Pass through content
                    let children = tag.children();
                    for handle in children.top().iter() {
                        if let Some(child) = handle.get(parser) {
                            collect_inline_node(dom, parser, child, inlines);
                        }
                    }
                }
                _ => {
                    // Unknown inline - try to get content
                    let children = tag.children();
                    for handle in children.top().iter() {
                        if let Some(child) = handle.get(parser) {
                            collect_inline_node(dom, parser, child, inlines);
                        }
                    }
                }
            }
        }
        Node::Raw(text) => {
            let text_str = text.as_utf8_str();
            let processed = collapse_and_escape(&text_str);
            if !processed.trim().is_empty() {
                inlines.push(Inline::Text(processed));
            }
        }
        _ => {}
    }
}

fn process_link(dom: &VDom, parser: &Parser, tag: &HTMLTag) -> Inline {
    let href = tag.attributes()
        .get("href")
        .flatten()
        .map(|s| s.as_utf8_str().to_string())
        .unwrap_or_default();
    let title = tag.attributes()
        .get("title")
        .flatten()
        .map(|s| s.as_utf8_str().to_string());
    let content = collect_inlines(dom, parser, tag);

    Inline::Link {
        content,
        url: href,
        title,
    }
}

fn process_image(tag: &HTMLTag) -> Option<Inline> {
    let src = tag.attributes()
        .get("src")
        .flatten()
        .map(|s| s.as_utf8_str().to_string())
        .unwrap_or_default();

    if src.is_empty() {
        return None;
    }

    let alt = tag.attributes()
        .get("alt")
        .flatten()
        .map(|s| s.as_utf8_str().to_string())
        .unwrap_or_default();
    let title = tag.attributes()
        .get("title")
        .flatten()
        .map(|s| s.as_utf8_str().to_string());

    Some(Inline::Image { url: src, alt, title })
}

fn get_text_content(dom: &VDom, parser: &Parser, tag: &HTMLTag) -> String {
    let mut result = String::new();
    let children = tag.children();

    for handle in children.top().iter() {
        if let Some(node) = handle.get(parser) {
            collect_text_recursive(dom, parser, node, &mut result);
        }
    }

    result
}

fn collect_text_recursive(dom: &VDom, parser: &Parser, node: &Node, result: &mut String) {
    match node {
        Node::Tag(tag) => {
            let children = tag.children();
            for handle in children.top().iter() {
                if let Some(child) = handle.get(parser) {
                    collect_text_recursive(dom, parser, child, result);
                }
            }
        }
        Node::Raw(text) => {
            result.push_str(&text.as_utf8_str());
        }
        _ => {}
    }
}

fn collect_list_items(dom: &VDom, parser: &Parser, tag: &HTMLTag) -> Vec<ListItem> {
    let mut items = Vec::new();
    let children = tag.children();

    for handle in children.top().iter() {
        if let Some(Node::Tag(li_tag)) = handle.get(parser) {
            let tag_name = li_tag.name().as_utf8_str();
            if tag_name.eq_ignore_ascii_case("li") {
                let li_children = li_tag.children();
                let inner_blocks = process_nodes(dom, parser, li_children.top().as_slice());

                let content = if inner_blocks.is_empty() {
                    let inlines = collect_inlines(dom, parser, li_tag);
                    if inlines.is_empty() {
                        vec![]
                    } else {
                        vec![Block::Paragraph(inlines)]
                    }
                } else {
                    inner_blocks
                };

                items.push(ListItem::new(content));
            }
        }
    }

    items
}

fn extract_code_content(dom: &VDom, parser: &Parser, pre_tag: &HTMLTag) -> (String, Option<String>) {
    let children = pre_tag.children();

    for handle in children.top().iter() {
        if let Some(Node::Tag(code_tag)) = handle.get(parser) {
            let tag_name = code_tag.name().as_utf8_str();
            if tag_name.eq_ignore_ascii_case("code") {
                let class = code_tag.attributes()
                    .get("class")
                    .flatten()
                    .map(|s| s.as_utf8_str().to_string());

                let lang = class.and_then(|c| {
                    c.split_whitespace()
                        .find(|s| s.starts_with("language-"))
                        .map(|s| s[9..].to_string())
                });

                let code = get_text_content(dom, parser, code_tag);
                return (code, lang);
            }
        }
    }

    // No code element, get text directly
    let code = get_text_content(dom, parser, pre_tag);
    (code, None)
}

fn collect_table(dom: &VDom, parser: &Parser, table_tag: &HTMLTag) -> (Vec<Vec<Inline>>, Vec<Vec<Vec<Inline>>>) {
    let mut headers = Vec::new();
    let mut rows = Vec::new();

    fn process_table_section(dom: &VDom, parser: &Parser, tag: &HTMLTag, headers: &mut Vec<Vec<Inline>>, rows: &mut Vec<Vec<Vec<Inline>>>) {
        let children = tag.children();
        for handle in children.top().iter() {
            if let Some(Node::Tag(child_tag)) = handle.get(parser) {
                let tag_name = child_tag.name().as_utf8_str();
                let tag_lower = tag_name.to_ascii_lowercase();

                match tag_lower.as_str() {
                    "thead" | "tbody" | "tfoot" => {
                        process_table_section(dom, parser, child_tag, headers, rows);
                    }
                    "tr" => {
                        let (row, is_header) = collect_table_row(dom, parser, child_tag);
                        if !row.is_empty() {
                            if is_header && headers.is_empty() {
                                *headers = row;
                            } else {
                                rows.push(row);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    process_table_section(dom, parser, table_tag, &mut headers, &mut rows);
    (headers, rows)
}

fn collect_table_row(dom: &VDom, parser: &Parser, tr_tag: &HTMLTag) -> (Vec<Vec<Inline>>, bool) {
    let mut cells = Vec::new();
    let mut is_header = false;
    let children = tr_tag.children();

    for handle in children.top().iter() {
        if let Some(Node::Tag(cell_tag)) = handle.get(parser) {
            let tag_name = cell_tag.name().as_utf8_str();
            let tag_lower = tag_name.to_ascii_lowercase();

            if tag_lower == "th" {
                is_header = true;
                cells.push(collect_inlines(dom, parser, cell_tag));
            } else if tag_lower == "td" {
                cells.push(collect_inlines(dom, parser, cell_tag));
            }
        }
    }

    (cells, is_header)
}

/// Combined whitespace collapsing and markdown escaping in single pass
#[inline]
fn collapse_and_escape(s: &str) -> String {
    const NEEDS_ESCAPE: [bool; 128] = {
        let mut table = [false; 128];
        table[b'\\' as usize] = true;
        table[b'*' as usize] = true;
        table[b'_' as usize] = true;
        table[b'[' as usize] = true;
        table[b']' as usize] = true;
        table[b'#' as usize] = true;
        table[b'+' as usize] = true;
        table[b'-' as usize] = true;
        table[b'!' as usize] = true;
        table[b'`' as usize] = true;
        table
    };

    let mut result = String::with_capacity(s.len());
    let mut prev_ws = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_ws {
                result.push(' ');
                prev_ws = true;
            }
        } else {
            prev_ws = false;
            let b = c as u32;
            if b < 128 && NEEDS_ESCAPE[b as usize] {
                result.push('\\');
            }
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convert(html: &str) -> String {
        let options = Options::default();
        let ast = html_to_ast(html, &options);
        turndown_core::serialize(&ast, &options)
    }

    #[test]
    fn test_paragraph() {
        let result = convert("<p>Hello World</p>");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_heading() {
        let result = convert("<h1>Title</h1>");
        assert!(result.contains("Title"));
    }

    #[test]
    fn test_strong() {
        let result = convert("<p><strong>bold</strong></p>");
        assert!(result.contains("**bold**"));
    }

    #[test]
    fn test_emphasis() {
        let result = convert("<p><em>italic</em></p>");
        assert!(result.contains("_italic_"));
    }

    #[test]
    fn test_link() {
        let result = convert("<a href=\"https://example.com\">Link</a>");
        assert_eq!(result, "[Link](https://example.com)");
    }

    #[test]
    fn test_image() {
        let result = convert("<img src=\"test.png\" alt=\"Alt\">");
        assert_eq!(result, "![Alt](test.png)");
    }

    #[test]
    fn test_code_inline() {
        let result = convert("<p><code>code</code></p>");
        assert!(result.contains("`code`"));
    }

    #[test]
    fn test_code_block() {
        let result = convert("<pre><code>let x = 1;</code></pre>");
        assert!(result.contains("let x = 1;"));
    }

    #[test]
    fn test_list() {
        let result = convert("<ul><li>Item 1</li><li>Item 2</li></ul>");
        // Check for list marker followed by Item text (may have variable spacing)
        assert!(result.contains("Item 1"), "Expected Item 1, got: {}", result);
        assert!(result.contains("Item 2"), "Expected Item 2, got: {}", result);
        assert!(result.contains("*") || result.contains("-"), "Expected list marker, got: {}", result);
    }
}

#[cfg(test)]
mod profiling {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore]
    fn profile_phases() {
        let html = std::fs::read_to_string(
            std::env::current_dir()
                .unwrap()
                .join("../../benchmarks/fixtures/large-100kb.html"),
        )
        .unwrap_or_else(|_| "<p>Test</p>".repeat(1000));

        println!("\n=== Input size: {} bytes ===", html.len());

        let options = Options::default();
        let iterations = if html.len() > 50_000 { 500 } else { 50_000 };

        // Full pipeline
        let start = Instant::now();
        for _ in 0..iterations {
            let ast = html_to_ast(&html, &options);
            let _ = turndown_core::serialize(&ast, &options);
        }
        let full_pipeline = start.elapsed();

        println!("\n=== Profile Results ({} iterations) ===", iterations);
        println!("Full pipeline:       {:>8.2?} ({:.2?}/iter)", full_pipeline, full_pipeline / iterations as u32);
    }
}
