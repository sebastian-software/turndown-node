//! turndown-core - Markdown AST and serialization
//!
//! This crate provides the core data structures and serialization for Markdown.
//! It is used by both `turndown-cdp` (for CDP DOM trees) and `turndown-napi`
//! (for streaming HTML parsing).
//!
//! # Architecture
//!
//! ```text
//! HTML String ──streaming──▶ ┌──────────────┐
//!                            │              │
//!                            │ Markdown AST │ ──▶ Markdown String
//! CDP Node Tree ────────────▶│              │
//!                            └──────────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use turndown_core::{Block, Inline, Options, serialize};
//!
//! let ast = Block::Document(vec![
//!     Block::Heading {
//!         level: 1,
//!         content: vec![Inline::Text("Hello World".to_string())],
//!     },
//!     Block::Paragraph(vec![
//!         Inline::Text("This is ".to_string()),
//!         Inline::Strong(vec![Inline::Text("bold".to_string())]),
//!         Inline::Text(" text.".to_string()),
//!     ]),
//! ]);
//!
//! let markdown = serialize(&ast, &Options::default());
//! ```

mod ast;
mod options;
mod serialize;

pub use ast::{inlines_text_len, Block, Inline, ListItem};
pub use options::{CodeBlockStyle, HeadingStyle, LinkReferenceStyle, LinkStyle, Options};
pub use serialize::serialize;
