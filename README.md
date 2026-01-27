# turndown-node

[![CI](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml/badge.svg)](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml)
[![npm version](https://img.shields.io/npm/v/turndown-node.svg)](https://www.npmjs.com/package/turndown-node)
[![crates.io](https://img.shields.io/crates/v/turndown-cdp.svg)](https://crates.io/crates/turndown-cdp)

Convert HTML to Markdown. Available as a native Node.js module and as a Rust crate.

## Packages

This monorepo provides two ways to use turndown:

| Package                                                        | Platform  | Description                        |
| -------------------------------------------------------------- | --------- | ---------------------------------- |
| [`turndown-node`](https://www.npmjs.com/package/turndown-node) | npm       | Node.js bindings with HTML parsing |
| [`turndown-cdp`](https://crates.io/crates/turndown-cdp)        | crates.io | Rust crate for CDP-style DOM trees |

## Node.js Usage

```bash
npm install turndown-node
```

```javascript
const TurndownService = require("turndown-node");

const turndownService = new TurndownService();
const markdown = turndownService.turndown("<h1>Hello World</h1>");
// => "Hello World\n==========="
```

**100% compatible** with [turndown](https://github.com/mixmark-io/turndown) v7.2.0 - drop-in replacement with identical output.

[Full Node.js documentation →](packages/turndown-node/README.md)

## Rust Usage

```bash
cargo add turndown-cdp
```

```rust
use turndown_cdp::{TurndownService, Node};

let service = TurndownService::new();

// Build a DOM tree
let mut h1 = Node::element("h1");
h1.add_child(Node::text("Hello World"));

let markdown = service.turndown(&h1).unwrap();
```

The Rust crate uses a **CDP-style Node structure** (Chrome DevTools Protocol), making it ideal for:

- Browser automation with [chromiumoxide](https://crates.io/crates/chromiumoxide)
- Readability/content extraction pipelines
- Any scenario where DOM is already parsed

[Full Rust documentation →](crates/turndown-cdp/README.md)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   turndown-cdp (Rust)                   │
│              Pure Node → Markdown conversion            │
│                   No HTML parser included               │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │
              ┌─────────────┴─────────────┐
              │                           │
   ┌──────────┴──────────┐     ┌──────────┴──────────┐
   │   turndown-napi     │     │   Your Rust App     │
   │   (HTML parsing)    │     │   (chromiumoxide)   │
   └──────────┬──────────┘     └─────────────────────┘
              │
   ┌──────────┴──────────┐
   │   turndown-node     │
   │   (npm package)     │
   └─────────────────────┘
```

## Supported Platforms

| Platform | Architecture          | npm | Rust |
| -------- | --------------------- | --- | ---- |
| macOS    | ARM64 (Apple Silicon) | ✅  | ✅   |
| macOS    | x64                   | -   | ✅   |
| Linux    | x64 (glibc)           | ✅  | ✅   |
| Linux    | ARM64 (glibc)         | ✅  | ✅   |
| Windows  | x64                   | ✅  | ✅   |

## Development

### Prerequisites

- Node.js >= 18
- Rust >= 1.70
- pnpm >= 9

### Setup

```bash
# Install dependencies
pnpm install

# Build native module
pnpm build

# Run tests
pnpm test

# Run Rust tests
cargo test --workspace
```

### Project Structure

```
turndown-node/
├── crates/
│   ├── turndown-cdp/       # Core Rust library (crates.io)
│   └── turndown-napi/      # NAPI-RS bindings + HTML parsing
├── packages/
│   ├── turndown-node/      # Main npm package
│   ├── darwin-arm64/       # Platform-specific bindings
│   ├── linux-x64-gnu/
│   ├── linux-arm64-gnu/
│   └── win32-x64-msvc/
└── tests/                  # Jest parity tests
```

## License

MIT
