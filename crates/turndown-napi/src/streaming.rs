//! HTML to Markdown AST conversion
//!
//! Uses scraper for HTML parsing and converts directly to Markdown AST.

use scraper::{ElementRef, Html, Node as ScraperNode};
use turndown_core::{Block, Inline, ListItem, Options};

/// Convert HTML string to Markdown AST
pub fn html_to_ast(html: &str, options: &Options) -> Block {
    let document = Html::parse_fragment(html);
    let root = document.root_element();
    convert_element(root, options)
}

/// Convert a scraper element to Markdown AST
fn convert_element(element: ElementRef, options: &Options) -> Block {
    let tag = element.value().name();

    // Handle specific elements
    match tag {
        // Root/container element - convert children
        "html" => Block::Document(collect_blocks(element, options)),

        // Block elements
        "p" => {
            let inlines = collect_inlines(element, options);
            if inlines_are_blank(&inlines) {
                Block::Document(vec![])
            } else {
                Block::Paragraph(inlines)
            }
        }

        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag.chars().nth(1).and_then(|c| c.to_digit(10)).unwrap_or(1) as u8;
            let inlines = collect_inlines(element, options);
            if inlines_are_blank(&inlines) {
                Block::Document(vec![])
            } else {
                Block::Heading {
                    level,
                    content: inlines,
                }
            }
        }

        "blockquote" => {
            let blocks = collect_blocks(element, options);
            if blocks.is_empty() {
                Block::Document(vec![])
            } else {
                Block::BlockQuote(blocks)
            }
        }

        "ul" => {
            let items = collect_list_items(element, options);
            if items.is_empty() {
                Block::Document(vec![])
            } else {
                Block::List {
                    ordered: false,
                    start: 1,
                    items,
                }
            }
        }

        "ol" => {
            let start = element
                .value()
                .attr("start")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            let items = collect_list_items(element, options);
            if items.is_empty() {
                Block::Document(vec![])
            } else {
                Block::List {
                    ordered: true,
                    start,
                    items,
                }
            }
        }

        "pre" => {
            // Look for <code> child
            let code_el = element
                .children()
                .filter_map(ElementRef::wrap)
                .find(|el| el.value().name() == "code");

            if let Some(code) = code_el {
                let code_text = code.text().collect::<String>();
                let language = code
                    .value()
                    .attr("class")
                    .and_then(|c| {
                        c.split_whitespace()
                            .find(|s| s.starts_with("language-"))
                            .map(|s| s[9..].to_string())
                    });

                let fenced =
                    matches!(options.code_block_style, turndown_core::CodeBlockStyle::Fenced);

                Block::CodeBlock {
                    language,
                    code: code_text,
                    fenced,
                }
            } else {
                // Pre without code
                let text = element.text().collect::<String>();
                Block::CodeBlock {
                    language: None,
                    code: text,
                    fenced: false,
                }
            }
        }

        "hr" => Block::ThematicBreak,

        "table" => convert_table(element, options),

        // Container elements - just process children
        "div" | "section" | "article" | "main" | "aside" | "header" | "footer" | "nav"
        | "figure" | "figcaption" | "address" | "form" | "fieldset" => {
            let blocks = collect_blocks(element, options);
            if blocks.len() == 1 {
                blocks.into_iter().next().unwrap()
            } else {
                Block::Document(blocks)
            }
        }

        // Inline-only elements at block level - wrap in paragraph
        "a" | "strong" | "b" | "em" | "i" | "code" | "span" | "img" | "br" => {
            if let Some(inline) = convert_inline_element(element, options) {
                Block::Paragraph(vec![inline])
            } else {
                Block::Document(vec![])
            }
        }

        // Skip these elements
        "script" | "style" | "noscript" | "template" => Block::Document(vec![]),

        // Unknown elements - try to get content
        _ => {
            let blocks = collect_blocks(element, options);
            if blocks.is_empty() {
                let inlines = collect_inlines(element, options);
                if inlines_are_blank(&inlines) {
                    Block::Document(vec![])
                } else {
                    Block::Paragraph(inlines)
                }
            } else if blocks.len() == 1 {
                blocks.into_iter().next().unwrap()
            } else {
                Block::Document(blocks)
            }
        }
    }
}

/// Collect blocks from children
fn collect_blocks(element: ElementRef, options: &Options) -> Vec<Block> {
    let mut blocks = Vec::new();

    for child in element.children() {
        match child.value() {
            ScraperNode::Text(text) => {
                let t = text.text.trim();
                if !t.is_empty() {
                    blocks.push(Block::Paragraph(vec![Inline::Text(escape_markdown(t))]));
                }
            }
            ScraperNode::Element(_) => {
                if let Some(el) = ElementRef::wrap(child) {
                    let block = convert_element(el, options);
                    // Flatten Document blocks
                    match block {
                        Block::Document(inner) => blocks.extend(inner),
                        other if !other.is_blank() => blocks.push(other),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    blocks
}

/// Collect list items from ul/ol
fn collect_list_items(element: ElementRef, options: &Options) -> Vec<ListItem> {
    let mut items = Vec::new();

    for child in element.children() {
        if let Some(el) = ElementRef::wrap(child) {
            if el.value().name() == "li" {
                let blocks = collect_blocks(el, options);
                let content = if blocks.is_empty() {
                    let inlines = collect_inlines(el, options);
                    vec![Block::Paragraph(inlines)]
                } else {
                    blocks
                };
                items.push(ListItem::new(content));
            }
        }
    }

    items
}

/// Convert a table element
fn convert_table(element: ElementRef, options: &Options) -> Block {
    let mut headers: Vec<Vec<Inline>> = Vec::new();
    let mut rows: Vec<Vec<Vec<Inline>>> = Vec::new();

    for child in element.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        match el.value().name() {
            "thead" => {
                for tr in el.children().filter_map(ElementRef::wrap) {
                    if tr.value().name() == "tr" {
                        for th in tr.children().filter_map(ElementRef::wrap) {
                            let name = th.value().name();
                            if name == "th" || name == "td" {
                                headers.push(collect_inlines(th, options));
                            }
                        }
                        break;
                    }
                }
            }
            "tbody" => {
                for tr in el.children().filter_map(ElementRef::wrap) {
                    if tr.value().name() == "tr" {
                        let mut row = Vec::new();
                        for td in tr.children().filter_map(ElementRef::wrap) {
                            let name = td.value().name();
                            if name == "td" || name == "th" {
                                row.push(collect_inlines(td, options));
                            }
                        }
                        if !row.is_empty() {
                            rows.push(row);
                        }
                    }
                }
            }
            "tr" => {
                let mut row = Vec::new();
                let mut is_header = false;

                for cell in el.children().filter_map(ElementRef::wrap) {
                    let name = cell.value().name();
                    if name == "th" {
                        is_header = true;
                        row.push(collect_inlines(cell, options));
                    } else if name == "td" {
                        row.push(collect_inlines(cell, options));
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
        return Block::Document(vec![]);
    }

    if headers.is_empty() && !rows.is_empty() {
        headers = rows.remove(0);
    }

    Block::Table { headers, rows }
}

/// Collect inline content from an element
fn collect_inlines(element: ElementRef, options: &Options) -> Vec<Inline> {
    let mut inlines = Vec::new();

    for child in element.children() {
        match child.value() {
            ScraperNode::Text(text) => {
                let collapsed = collapse_whitespace(&text.text);
                if !collapsed.is_empty() {
                    inlines.push(Inline::Text(escape_markdown(&collapsed)));
                }
            }
            ScraperNode::Element(_) => {
                if let Some(el) = ElementRef::wrap(child) {
                    if let Some(inline) = convert_inline_element(el, options) {
                        inlines.push(inline);
                    }
                }
            }
            _ => {}
        }
    }

    inlines
}

/// Convert an inline element
fn convert_inline_element(element: ElementRef, options: &Options) -> Option<Inline> {
    let tag = element.value().name();

    match tag {
        "strong" | "b" => {
            let inner = collect_inlines(element, options);
            if inlines_are_blank(&inner) {
                None
            } else {
                Some(Inline::Strong(inner))
            }
        }

        "em" | "i" => {
            let inner = collect_inlines(element, options);
            if inlines_are_blank(&inner) {
                None
            } else {
                Some(Inline::Emphasis(inner))
            }
        }

        "code" => {
            let text = element.text().collect::<String>();
            if text.is_empty() {
                None
            } else {
                Some(Inline::Code(text))
            }
        }

        "a" => {
            let href = element.value().attr("href").unwrap_or("");
            let title = element.value().attr("title").map(|s| s.to_string());
            let content = collect_inlines(element, options);

            if href.is_empty() && title.is_none() {
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
            let src = element.value().attr("src").unwrap_or("");
            if src.is_empty() {
                return None;
            }

            let alt = element.value().attr("alt").unwrap_or("").to_string();
            let title = element.value().attr("title").map(|s| s.to_string());

            Some(Inline::Image {
                alt,
                url: src.to_string(),
                title,
            })
        }

        "br" => Some(Inline::LineBreak),

        "span" | "small" | "mark" | "abbr" | "cite" | "q" | "sub" | "sup" | "time" => {
            let inner = collect_inlines(element, options);
            if inner.len() == 1 {
                Some(inner.into_iter().next().unwrap())
            } else if inner.is_empty() {
                None
            } else {
                Some(Inline::Text(
                    inner.iter().map(|i| inline_to_text(i)).collect(),
                ))
            }
        }

        _ => {
            let inner = collect_inlines(element, options);
            if inner.len() == 1 {
                Some(inner.into_iter().next().unwrap())
            } else if inner.is_empty() {
                None
            } else {
                Some(Inline::Text(
                    inner.iter().map(|i| inline_to_text(i)).collect(),
                ))
            }
        }
    }
}

fn inline_to_text(inline: &Inline) -> String {
    match inline {
        Inline::Text(t) => t.clone(),
        Inline::Strong(inner) | Inline::Emphasis(inner) => {
            inner.iter().map(inline_to_text).collect()
        }
        Inline::Code(c) => c.clone(),
        Inline::Link { content, .. } => content.iter().map(inline_to_text).collect(),
        Inline::Image { alt, .. } => alt.clone(),
        Inline::LineBreak => "\n".to_string(),
        Inline::HtmlInline(h) => h.clone(),
    }
}

fn inlines_are_blank(inlines: &[Inline]) -> bool {
    inlines.iter().all(|i| i.is_blank())
}

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
        assert!(result.contains("="));
    }

    #[test]
    fn test_strong() {
        let result = convert("<strong>bold</strong>");
        assert_eq!(result, "**bold**");
    }

    #[test]
    fn test_emphasis() {
        let result = convert("<em>italic</em>");
        assert_eq!(result, "_italic_");
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
        let result = convert("<code>code</code>");
        assert_eq!(result, "`code`");
    }

    #[test]
    fn test_code_block() {
        let result = convert("<pre><code>let x = 1;</code></pre>");
        assert_eq!(result, "    let x = 1;");
    }

    #[test]
    fn test_list() {
        let result = convert("<ul><li>One</li><li>Two</li></ul>");
        assert!(result.contains("*   One"));
        assert!(result.contains("*   Two"));
    }
}
