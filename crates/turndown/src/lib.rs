//! # turndown
//!
//! Convert HTML to Markdown.
//!
//! This is a Rust port of [turndown](https://github.com/mixmark-io/turndown),
//! providing the same API and identical output.
//!
//! ## Example
//!
//! ```rust
//! use turndown::TurndownService;
//!
//! let service = TurndownService::new();
//! let markdown = service.turndown("<h1>Hello World</h1>").unwrap();
//! assert!(markdown.contains("Hello World"));
//! ```

mod rules;
mod service;
mod utilities;

pub use rules::{Filter, Rule, Rules};
pub use service::{
    CodeBlockStyle, HeadingStyle, LinkReferenceStyle, LinkStyle, TurndownOptions, TurndownService,
};
pub use utilities::*;

/// Error type for turndown operations
#[derive(Debug, thiserror::Error)]
pub enum TurndownError {
    #[error("Failed to parse HTML: {0}")]
    ParseError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, TurndownError>;
