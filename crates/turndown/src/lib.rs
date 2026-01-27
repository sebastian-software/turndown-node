//! # turndown
//!
//! Convert DOM nodes to Markdown.
//!
//! This is a Rust implementation inspired by [turndown](https://github.com/mixmark-io/turndown),
//! providing a similar API for converting DOM trees to Markdown.
//!
//! ## Design
//!
//! Unlike the original turndown which parses HTML strings, this library accepts
//! a CDP-style DOM Node structure. This design allows:
//!
//! - **Zero parsing overhead**: When DOM is already available (e.g., from CDP/chromiumoxide)
//! - **Parser agnostic**: Any HTML parser can convert to the Node structure
//! - **Smaller binaries**: No HTML parser bundled by default
//!
//! ## Example (Node-based)
//!
//! ```rust
//! use turndown::{TurndownService, Node};
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
//!
//! ## Example (HTML string)
//!
//! ```rust
//! use turndown::TurndownService;
//!
//! let service = TurndownService::new();
//! let markdown = service.turndown_html("<h1>Hello World</h1>").unwrap();
//! assert!(markdown.contains("Hello World"));
//! ```

#[cfg(feature = "html")]
pub mod html;
pub mod node;
mod rules;
mod service;
mod utilities;

#[cfg(feature = "html")]
pub use html::parse_html;
pub use node::{Node, NodeRef, NodeType};
pub use rules::{Filter, Rule, Rules};
pub use service::{
    CodeBlockStyle, HeadingStyle, LinkReferenceStyle, LinkStyle, TurndownOptions, TurndownService,
};
pub use utilities::*;

/// Error type for turndown operations
#[derive(Debug, thiserror::Error)]
pub enum TurndownError {
    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, TurndownError>;
