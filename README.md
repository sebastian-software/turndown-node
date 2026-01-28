# turndown-node

[![CI](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml/badge.svg)](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml)
[![npm version](https://img.shields.io/npm/v/turndown-node.svg)](https://www.npmjs.com/package/turndown-node)
[![npm downloads](https://img.shields.io/npm/dm/turndown-node.svg)](https://www.npmjs.com/package/turndown-node)
[![crates.io](https://img.shields.io/crates/v/turndown-cdp.svg)](https://crates.io/crates/turndown-cdp)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Node.js](https://img.shields.io/badge/Node.js-%3E%3D18-green.svg)](https://nodejs.org/)

Convert HTML to Markdown. Native Node.js bindings powered by Rust for maximum performance.

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

## Performance

turndown-node is significantly faster than the JavaScript implementation thanks to native Rust code:

| Fixture       | Size     | turndown-node | turndown (JS) | Speedup   |
| ------------- | -------- | ------------- | ------------- | --------- |
| simple        | 36 bytes | 315,048 ops/s | 54,320 ops/s  | **5.80x** |
| blog-post     | 2.5 KB   | 9,445 ops/s   | 3,287 ops/s   | **2.87x** |
| documentation | 4.3 KB   | 3,978 ops/s   | 1,546 ops/s   | **2.57x** |
| large-article | 28.6 KB  | 786 ops/s     | 304 ops/s     | **2.59x** |
| large-100kb   | 188.8 KB | 97 ops/s      | 28 ops/s      | **3.48x** |

**Average speedup: 3.46x faster**

> Benchmarks run on Apple M-Series, Node.js v24. Run `node benchmarks/benchmark.js` to reproduce.

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
┌─────────────────────────────────────────────────────────────────┐
│                        turndown-core                            │
│              Markdown AST types + Serialization                 │
└─────────────────────────────────────────────────────────────────┘
                            ▲
                            │
              ┌─────────────┴─────────────┐
              │                           │
   ┌──────────┴──────────┐     ┌──────────┴──────────┐
   │   turndown-napi     │     │    turndown-cdp     │
   │   HTML → AST        │     │   CDP Node → AST    │
   │   (tl parser)       │     │   (for Rust apps)   │
   └──────────┬──────────┘     └─────────────────────┘
              │
   ┌──────────┴──────────┐
   │   turndown-node     │
   │   (npm package)     │
   └─────────────────────┘
```

See [Architecture Decision Records](docs/adr/) for design rationale.

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

# Run benchmarks
node benchmarks/benchmark.js
```

### Project Structure

```
turndown-node/
├── crates/
│   ├── turndown-core/     # Shared Markdown AST + serialization
│   ├── turndown-cdp/      # CDP Node → AST (crates.io)
│   └── turndown-napi/     # HTML → AST with NAPI-RS bindings
├── packages/
│   ├── turndown-node/     # Main npm package
│   ├── darwin-arm64/      # Platform-specific bindings
│   ├── linux-x64-gnu/
│   ├── linux-arm64-gnu/
│   └── win32-x64-msvc/
├── benchmarks/            # Performance benchmarks
├── tests/                 # Jest parity tests
└── docs/adr/              # Architecture Decision Records
```

## License

MIT
