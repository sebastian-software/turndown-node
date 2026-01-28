//! # turndown-cdp
//!
//! Convert CDP-style DOM nodes to Markdown.
//!
//! This is a Rust implementation inspired by [turndown](https://github.com/mixmark-io/turndown),
//! providing a similar API for converting DOM trees to Markdown.
//!
//! ## Design
//!
//! This library accepts a CDP-style DOM Node structure (matching Chrome DevTools
//! Protocol). This design allows:
//!
//! - **Zero parsing overhead**: When DOM is already available (e.g., from CDP/chromiumoxide)
//! - **Parser agnostic**: Any HTML parser can convert to the Node structure
//! - **Minimal dependencies**: No HTML parser bundled
//!
//! ## Example
//!
//! ```rust
//! use turndown_cdp::{TurndownService, Node};
//!
//! let service = TurndownService::new();
//!
//! // Create a simple DOM tree
//! let mut h1 = Node::element("h1");
//! h1.add_child(Node::text("Hello World"));
//!
//! let markdown = service.turndown(&h1).unwrap();
//! assert!(markdown.contains("Hello World"));
//! ```

mod convert;
pub mod node;
mod service;

pub use node::{Node, NodeRef, NodeType};
pub use service::{CodeBlockStyle, HeadingStyle, LinkReferenceStyle, LinkStyle, TurndownOptions, TurndownService};

/// Error type for turndown operations
#[derive(Debug, thiserror::Error)]
pub enum TurndownError {
    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, TurndownError>;
