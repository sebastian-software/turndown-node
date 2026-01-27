# turndown-cdp

[![crates.io](https://img.shields.io/crates/v/turndown-cdp.svg)](https://crates.io/crates/turndown-cdp)
[![docs.rs](https://docs.rs/turndown-cdp/badge.svg)](https://docs.rs/turndown-cdp)
[![CI](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml/badge.svg)](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml)

Convert CDP-style DOM nodes to Markdown.

This crate provides a Rust implementation inspired by [turndown](https://github.com/mixmark-io/turndown), using a **Chrome DevTools Protocol (CDP) style Node structure**. This makes it ideal for browser automation and content extraction pipelines.

## Features

- **No HTML parser included** - Minimal dependencies, small binary size
- **CDP-compatible Node structure** - Works directly with browser DOM
- **Parser agnostic** - Use with any HTML parser (scraper, html5ever, etc.)
- **Full CommonMark support** - Headings, lists, code blocks, links, images, etc.

## Installation

```bash
cargo add turndown-cdp
```

## Usage

### Basic Example

```rust
use turndown_cdp::{TurndownService, Node};

let service = TurndownService::new();

// Create a DOM tree
let mut h1 = Node::element("h1");
h1.add_child(Node::text("Hello World"));

let markdown = service.turndown(&h1).unwrap();
assert!(markdown.contains("Hello World"));
```

### Building Complex Documents

```rust
use turndown_cdp::{TurndownService, Node};

let service = TurndownService::new();

// Create a document with multiple elements
let mut doc = Node::document_fragment();

// Add a heading
let mut h1 = Node::element("h1");
h1.add_child(Node::text("Welcome"));
doc.add_child(h1);

// Add a paragraph with emphasis
let mut p = Node::element("p");
p.add_child(Node::text("This is "));
let mut strong = Node::element("strong");
strong.add_child(Node::text("important"));
p.add_child(strong);
doc.add_child(p);

// Add a link
let mut link = Node::element_with_attrs("a", vec![("href", "https://example.com")]);
link.add_child(Node::text("Click here"));
doc.add_child(link);

let markdown = service.turndown(&doc).unwrap();
```

### With chromiumoxide

```rust
use turndown_cdp::{TurndownService, Node};
use chromiumoxide::Browser;

// After getting DOM from Chrome via CDP...
fn cdp_to_node(cdp_node: &chromiumoxide::cdp::browser_protocol::dom::Node) -> Node {
    // Map CDP node types to turndown Node
    match cdp_node.node_type as u32 {
        3 => Node::text(cdp_node.node_value.as_deref().unwrap_or("")),
        1 => {
            let mut node = Node::element(&cdp_node.node_name.to_lowercase());
            // Add attributes and children...
            node
        }
        _ => Node::text(""),
    }
}

let service = TurndownService::new();
let markdown = service.turndown(&your_node).unwrap();
```

### Custom Options

````rust
use turndown_cdp::{TurndownService, TurndownOptions, HeadingStyle, CodeBlockStyle};

let options = TurndownOptions {
    heading_style: HeadingStyle::Atx,           // Use # style headings
    code_block_style: CodeBlockStyle::Fenced,   // Use ``` code blocks
    bullet_list_marker: '-',                     // Use - for lists
    ..Default::default()
};

let service = TurndownService::with_options(options);
````

## Node Structure

The `Node` struct matches the CDP DOM.Node structure:

```rust
pub struct Node {
    pub node_type: NodeType,        // Element, Text, Document, etc.
    pub node_name: String,          // "DIV", "#text", etc.
    pub node_value: Option<String>, // Text content for text nodes
    pub attributes: Option<Vec<String>>, // ["href", "url", "class", "foo"]
    pub children: Option<Vec<Node>>,
}
```

### Creating Nodes

```rust
use turndown_cdp::Node;

// Text node
let text = Node::text("Hello");

// Element node
let div = Node::element("div");

// Element with attributes
let link = Node::element_with_attrs("a", vec![
    ("href", "https://example.com"),
    ("title", "Example"),
]);

// Document fragment (container)
let fragment = Node::document_fragment();
```

## Related

- [`turndown-node`](https://www.npmjs.com/package/turndown-node) - Node.js bindings with HTML parsing
- [turndown](https://github.com/mixmark-io/turndown) - The original JavaScript library
- [chromiumoxide](https://crates.io/crates/chromiumoxide) - Chrome DevTools Protocol client

## License

MIT
