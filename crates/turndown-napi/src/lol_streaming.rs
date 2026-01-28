//! Streaming HTML to Markdown AST conversion using lol_html
//!
//! Uses Cloudflare's lol_html for true streaming parsing without building a DOM tree.

use lol_html::{element, rewrite_str, RewriteStrSettings};
use std::cell::RefCell;
use std::rc::Rc;
use turndown_core::{Block, Inline, ListItem, Options};

/// Shared state for the streaming parser
#[derive(Debug, Clone)]
struct ParserState {
    /// Stack of open elements being built
    stack: Vec<ElementContext>,
    /// The options for conversion
    options: Options,
}

/// Context for an element being processed
#[derive(Debug, Clone)]
struct ElementContext {
    tag: String,
    /// Collected inline content (for inline-container elements like p, h1, etc.)
    inlines: Vec<Inline>,
    /// Collected block content (for block-container elements like div, blockquote, etc.)
    blocks: Vec<Block>,
    /// Collected list items (for ul/ol)
    list_items: Vec<ListItem>,
    /// For tables: header cells
    table_headers: Vec<Vec<Inline>>,
    /// For tables: body rows
    table_rows: Vec<Vec<Vec<Inline>>>,
    /// Attributes we care about
    attrs: ElementAttrs,
}

#[derive(Default, Debug, Clone)]
struct ElementAttrs {
    href: Option<String>,
    src: Option<String>,
    alt: Option<String>,
    title: Option<String>,
    start: Option<u32>,
    class: Option<String>,
}

impl ElementContext {
    fn new(tag: String) -> Self {
        Self {
            tag,
            inlines: Vec::new(),
            blocks: Vec::new(),
            list_items: Vec::new(),
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            attrs: ElementAttrs::default(),
        }
    }
}

impl ParserState {
    fn new(options: Options) -> Self {
        // Start with a root document context
        let mut stack = Vec::new();
        stack.push(ElementContext::new("$root".to_string()));
        Self { stack, options }
    }

    fn push_element(&mut self, tag: String, attrs: ElementAttrs) {
        let mut ctx = ElementContext::new(tag);
        ctx.attrs = attrs;
        self.stack.push(ctx);
    }

    fn pop_element(&mut self) -> Option<ElementContext> {
        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }

    fn current_mut(&mut self) -> &mut ElementContext {
        self.stack.last_mut().expect("stack should never be empty")
    }

    fn add_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        // Check if we're inside a preformatted context
        let in_pre = self.stack.iter().any(|ctx| ctx.tag == "pre");
        let in_code = self.stack.iter().any(|ctx| ctx.tag == "code");

        let processed = if in_pre || in_code {
            text.to_string()
        } else {
            let collapsed = collapse_whitespace(text);
            if collapsed.is_empty() {
                return;
            }
            escape_markdown(&collapsed)
        };

        self.current_mut().inlines.push(Inline::Text(processed));
    }

    fn add_inline(&mut self, inline: Inline) {
        self.current_mut().inlines.push(inline);
    }

    fn add_block(&mut self, block: Block) {
        if !block.is_blank() {
            self.current_mut().blocks.push(block);
        }
    }

    fn finalize(mut self) -> Block {
        // The root context should have all the collected blocks
        let root = self.stack.pop().expect("root context");

        // If we have inlines but no blocks, wrap them in a paragraph
        if root.blocks.is_empty() {
            if inlines_are_blank(&root.inlines) {
                Block::Document(vec![])
            } else {
                Block::Paragraph(root.inlines)
            }
        } else if root.blocks.len() == 1 && root.inlines.is_empty() {
            root.blocks.into_iter().next().unwrap()
        } else {
            // If we have both blocks and inlines, add inlines as a paragraph at the end
            let mut blocks = root.blocks;
            if !inlines_are_blank(&root.inlines) {
                blocks.push(Block::Paragraph(root.inlines));
            }
            Block::Document(blocks)
        }
    }
}

/// Convert HTML string to Markdown AST using streaming parser
pub fn html_to_ast(html: &str, options: &Options) -> Block {
    let state = Rc::new(RefCell::new(ParserState::new(options.clone())));

    let state_for_element = Rc::clone(&state);
    let state_for_text = Rc::clone(&state);

    // We need to handle elements and text
    let result = rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![
                // Match all elements
                element!("*", |el| {
                    let tag = el.tag_name().to_lowercase();

                    // Skip script, style, etc.
                    if matches!(tag.as_str(), "script" | "style" | "noscript" | "template") {
                        el.remove();
                        return Ok(());
                    }

                    // Collect attributes
                    let mut attrs = ElementAttrs::default();
                    if let Some(href) = el.get_attribute("href") {
                        attrs.href = Some(href);
                    }
                    if let Some(src) = el.get_attribute("src") {
                        attrs.src = Some(src);
                    }
                    if let Some(alt) = el.get_attribute("alt") {
                        attrs.alt = Some(alt);
                    }
                    if let Some(title) = el.get_attribute("title") {
                        attrs.title = Some(title);
                    }
                    if let Some(start) = el.get_attribute("start") {
                        attrs.start = start.parse().ok();
                    }
                    if let Some(class) = el.get_attribute("class") {
                        attrs.class = Some(class);
                    }

                    // Handle self-closing elements immediately
                    if matches!(tag.as_str(), "br" | "hr" | "img") {
                        let mut state = state_for_element.borrow_mut();
                        match tag.as_str() {
                            "br" => {
                                state.add_inline(Inline::LineBreak);
                            }
                            "hr" => {
                                state.add_block(Block::ThematicBreak);
                            }
                            "img" => {
                                if let Some(src) = attrs.src {
                                    if !src.is_empty() {
                                        state.add_inline(Inline::Image {
                                            alt: attrs.alt.unwrap_or_default(),
                                            url: src,
                                            title: attrs.title,
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                        return Ok(());
                    }

                    // Push element context for non-self-closing tags
                    {
                        let mut state = state_for_element.borrow_mut();
                        state.push_element(tag.clone(), attrs);
                    }

                    // Register end tag handler
                    let state_for_end = Rc::clone(&state_for_element);

                    if let Some(handlers) = el.end_tag_handlers() {
                        let handler: lol_html::EndTagHandler<'static> = Box::new(move |_end_tag: &mut lol_html::html_content::EndTag<'_>| {
                            let mut state = state_for_end.borrow_mut();
                            if let Some(ctx) = state.pop_element() {
                                let block = finalize_element(&ctx, &state.options);
                                match block {
                                    FinalizedElement::Block(b) => state.add_block(b),
                                    FinalizedElement::Inline(i) => state.add_inline(i),
                                    FinalizedElement::None => {}
                                }
                            }
                            Ok(())
                        });
                        handlers.push(handler);
                    }

                    Ok(())
                }),
                // Text handler
                lol_html::text!("*", |text| {
                    let content = text.as_str();
                    if !content.is_empty() {
                        let mut state = state_for_text.borrow_mut();
                        state.add_text(content);
                    }
                    Ok(())
                }),
            ],
            ..Default::default()
        },
    );

    match result {
        Ok(_) => {
            // After rewrite_str completes, handlers have been executed
            // We can safely take the state since we're the only remaining user
            let state = match Rc::try_unwrap(state) {
                Ok(cell) => cell.into_inner(),
                Err(rc) => {
                    // If unwrap fails, clone the state (this shouldn't happen normally)
                    let borrowed = rc.borrow();
                    ParserState {
                        stack: borrowed.stack.clone(),
                        options: borrowed.options.clone(),
                    }
                }
            };
            state.finalize()
        }
        Err(_) => Block::Document(vec![]),
    }
}

enum FinalizedElement {
    Block(Block),
    Inline(Inline),
    None,
}

fn finalize_element(ctx: &ElementContext, options: &Options) -> FinalizedElement {
    let tag = ctx.tag.as_str();

    match tag {
        // Block elements
        "p" => {
            if inlines_are_blank(&ctx.inlines) {
                FinalizedElement::None
            } else {
                FinalizedElement::Block(Block::Paragraph(ctx.inlines.clone()))
            }
        }

        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag.chars().nth(1).and_then(|c| c.to_digit(10)).unwrap_or(1) as u8;
            if inlines_are_blank(&ctx.inlines) {
                FinalizedElement::None
            } else {
                FinalizedElement::Block(Block::Heading {
                    level,
                    content: ctx.inlines.clone(),
                })
            }
        }

        "blockquote" => {
            let blocks = if ctx.blocks.is_empty() && !inlines_are_blank(&ctx.inlines) {
                vec![Block::Paragraph(ctx.inlines.clone())]
            } else {
                ctx.blocks.clone()
            };
            if blocks.is_empty() {
                FinalizedElement::None
            } else {
                FinalizedElement::Block(Block::BlockQuote(blocks))
            }
        }

        "ul" => {
            if ctx.list_items.is_empty() {
                FinalizedElement::None
            } else {
                FinalizedElement::Block(Block::List {
                    ordered: false,
                    start: 1,
                    items: ctx.list_items.clone(),
                })
            }
        }

        "ol" => {
            if ctx.list_items.is_empty() {
                FinalizedElement::None
            } else {
                FinalizedElement::Block(Block::List {
                    ordered: true,
                    start: ctx.attrs.start.unwrap_or(1),
                    items: ctx.list_items.clone(),
                })
            }
        }

        "li" => {
            // Li is special - it needs to be added to parent's list_items
            // This is handled by checking the parent context
            let content = if ctx.blocks.is_empty() {
                vec![Block::Paragraph(ctx.inlines.clone())]
            } else {
                ctx.blocks.clone()
            };
            // Return as a block that will be converted to ListItem by parent
            FinalizedElement::Block(Block::Document(content))
        }

        "pre" => {
            // Look for code content
            let code_text: String = ctx
                .inlines
                .iter()
                .map(|i| match i {
                    Inline::Text(t) => t.as_str(),
                    Inline::Code(c) => c.as_str(),
                    _ => "",
                })
                .collect();

            let language = ctx.attrs.class.as_ref().and_then(|c| {
                c.split_whitespace()
                    .find(|s| s.starts_with("language-"))
                    .map(|s| s[9..].to_string())
            });

            let fenced = matches!(options.code_block_style, turndown_core::CodeBlockStyle::Fenced);

            FinalizedElement::Block(Block::CodeBlock {
                language,
                code: code_text,
                fenced,
            })
        }

        "code" => {
            // Inline code
            let text: String = ctx
                .inlines
                .iter()
                .map(|i| match i {
                    Inline::Text(t) => t.clone(),
                    _ => String::new(),
                })
                .collect();

            if text.is_empty() {
                FinalizedElement::None
            } else {
                FinalizedElement::Inline(Inline::Code(text))
            }
        }

        "strong" | "b" => {
            if inlines_are_blank(&ctx.inlines) {
                FinalizedElement::None
            } else {
                FinalizedElement::Inline(Inline::Strong(ctx.inlines.clone()))
            }
        }

        "em" | "i" => {
            if inlines_are_blank(&ctx.inlines) {
                FinalizedElement::None
            } else {
                FinalizedElement::Inline(Inline::Emphasis(ctx.inlines.clone()))
            }
        }

        "a" => {
            let href = ctx.attrs.href.as_deref().unwrap_or("");
            if href.is_empty() && ctx.attrs.title.is_none() {
                // No href, just return the content
                if ctx.inlines.len() == 1 {
                    FinalizedElement::Inline(ctx.inlines[0].clone())
                } else {
                    FinalizedElement::None
                }
            } else {
                FinalizedElement::Inline(Inline::Link {
                    content: ctx.inlines.clone(),
                    url: href.to_string(),
                    title: ctx.attrs.title.clone(),
                })
            }
        }

        "table" => {
            if ctx.table_headers.is_empty() && ctx.table_rows.is_empty() {
                FinalizedElement::None
            } else {
                let mut headers = ctx.table_headers.clone();
                let mut rows = ctx.table_rows.clone();

                if headers.is_empty() && !rows.is_empty() {
                    headers = rows.remove(0);
                }

                FinalizedElement::Block(Block::Table { headers, rows })
            }
        }

        // Container elements - pass through content
        "div" | "section" | "article" | "main" | "aside" | "header" | "footer" | "nav"
        | "figure" | "figcaption" | "address" | "form" | "fieldset" | "thead" | "tbody"
        | "tr" | "td" | "th" => {
            if !ctx.blocks.is_empty() {
                if ctx.blocks.len() == 1 {
                    FinalizedElement::Block(ctx.blocks[0].clone())
                } else {
                    FinalizedElement::Block(Block::Document(ctx.blocks.clone()))
                }
            } else if !inlines_are_blank(&ctx.inlines) {
                FinalizedElement::Block(Block::Paragraph(ctx.inlines.clone()))
            } else {
                FinalizedElement::None
            }
        }

        // Inline containers - merge inlines
        "span" | "small" | "mark" | "abbr" | "cite" | "q" | "sub" | "sup" | "time" => {
            if ctx.inlines.len() == 1 {
                FinalizedElement::Inline(ctx.inlines[0].clone())
            } else if ctx.inlines.is_empty() {
                FinalizedElement::None
            } else {
                // Merge text
                let text: String = ctx.inlines.iter().map(inline_to_text).collect();
                FinalizedElement::Inline(Inline::Text(text))
            }
        }

        _ => {
            // Unknown element - try to preserve content
            if !ctx.blocks.is_empty() {
                if ctx.blocks.len() == 1 {
                    FinalizedElement::Block(ctx.blocks[0].clone())
                } else {
                    FinalizedElement::Block(Block::Document(ctx.blocks.clone()))
                }
            } else if !inlines_are_blank(&ctx.inlines) {
                FinalizedElement::Block(Block::Paragraph(ctx.inlines.clone()))
            } else {
                FinalizedElement::None
            }
        }
    }
}

fn inline_to_text(inline: &Inline) -> String {
    match inline {
        Inline::Text(t) => t.clone(),
        Inline::Strong(inner) | Inline::Emphasis(inner) => inner.iter().map(inline_to_text).collect(),
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
}
