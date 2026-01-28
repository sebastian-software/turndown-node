//! Markdown Abstract Syntax Tree
//!
//! This module defines the AST nodes for representing Markdown documents.
//! The AST is the common intermediate format used by both CDP and streaming converters.

/// A block-level Markdown node
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Root document container
    Document(Vec<Block>),

    /// Heading with level (1-6) and inline content
    Heading {
        level: u8,
        content: Vec<Inline>,
    },

    /// Paragraph containing inline content
    Paragraph(Vec<Inline>),

    /// Block quote containing nested blocks
    BlockQuote(Vec<Block>),

    /// List (ordered or unordered)
    List {
        ordered: bool,
        start: u32,
        items: Vec<ListItem>,
    },

    /// Fenced or indented code block
    CodeBlock {
        language: Option<String>,
        code: String,
        fenced: bool,
    },

    /// Thematic break (horizontal rule)
    ThematicBreak,

    /// Table with headers and rows
    Table {
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },

    /// Raw HTML block (for `keep` elements)
    HtmlBlock(String),
}

/// A list item containing blocks
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub content: Vec<Block>,
}

impl ListItem {
    pub fn new(content: Vec<Block>) -> Self {
        Self { content }
    }

    pub fn from_inlines(inlines: Vec<Inline>) -> Self {
        Self {
            content: vec![Block::Paragraph(inlines)],
        }
    }
}

/// An inline Markdown node
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain text
    Text(String),

    /// Strong emphasis (bold)
    Strong(Vec<Inline>),

    /// Emphasis (italic)
    Emphasis(Vec<Inline>),

    /// Inline code
    Code(String),

    /// Link with text, URL, and optional title
    Link {
        content: Vec<Inline>,
        url: String,
        title: Option<String>,
    },

    /// Image with alt text, URL, and optional title
    Image {
        alt: String,
        url: String,
        title: Option<String>,
    },

    /// Hard line break
    LineBreak,

    /// Raw HTML inline (for `keep` elements)
    HtmlInline(String),
}

impl Block {
    /// Check if this block is empty/blank
    pub fn is_blank(&self) -> bool {
        match self {
            Block::Document(blocks) => blocks.iter().all(|b| b.is_blank()),
            Block::Paragraph(inlines) => inlines.iter().all(|i| i.is_blank()),
            Block::Heading { content, .. } => content.iter().all(|i| i.is_blank()),
            Block::BlockQuote(blocks) => blocks.iter().all(|b| b.is_blank()),
            Block::List { items, .. } => items.iter().all(|i| i.is_blank()),
            Block::CodeBlock { code, .. } => code.trim().is_empty(),
            Block::Table { headers, rows } => {
                headers.iter().all(|h| h.iter().all(|i| i.is_blank()))
                    && rows
                        .iter()
                        .all(|r| r.iter().all(|c| c.iter().all(|i| i.is_blank())))
            }
            Block::ThematicBreak => false,
            Block::HtmlBlock(html) => html.trim().is_empty(),
        }
    }
}

impl ListItem {
    pub fn is_blank(&self) -> bool {
        self.content.iter().all(|b| b.is_blank())
    }
}

impl Inline {
    /// Check if this inline is empty/blank
    pub fn is_blank(&self) -> bool {
        match self {
            Inline::Text(text) => text.trim().is_empty(),
            Inline::Strong(inlines) | Inline::Emphasis(inlines) => {
                inlines.iter().all(|i| i.is_blank())
            }
            Inline::Code(code) => code.is_empty(),
            Inline::Link { content, .. } => content.iter().all(|i| i.is_blank()),
            Inline::Image { .. } => false,
            Inline::LineBreak => false,
            Inline::HtmlInline(html) => html.trim().is_empty(),
        }
    }

    /// Get the text content of this inline (for measuring table column widths)
    pub fn text_len(&self) -> usize {
        match self {
            Inline::Text(text) => text.len(),
            Inline::Strong(inlines) | Inline::Emphasis(inlines) => {
                inlines.iter().map(|i| i.text_len()).sum::<usize>() + 4 // ** or _
            }
            Inline::Code(code) => code.len() + 2, // backticks
            Inline::Link { content, .. } => {
                content.iter().map(|i| i.text_len()).sum::<usize>() + 4 // []()
            }
            Inline::Image { alt, .. } => alt.len() + 5, // ![]()
            Inline::LineBreak => 0,
            Inline::HtmlInline(html) => html.len(),
        }
    }
}

/// Helper to calculate text length of inline vec
pub fn inlines_text_len(inlines: &[Inline]) -> usize {
    inlines.iter().map(|i| i.text_len()).sum()
}
