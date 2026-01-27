# turndown-node

[![CI](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml/badge.svg)](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml)
[![npm version](https://img.shields.io/npm/v/turndown-node.svg)](https://www.npmjs.com/package/turndown-node)

Convert HTML to Markdown - Native Node.js bindings for [turndown](https://github.com/mixmark-io/turndown), powered by Rust.

## Why turndown-node?

- **100% Compatible**: Drop-in replacement for turndown with identical output
- **Native Performance**: Built with Rust and NAPI-RS for maximum speed
- **Battle-tested Parser**: Uses html5ever (Firefox's HTML parser) via the scraper crate

## Installation

```bash
npm install turndown-node
# or
pnpm add turndown-node
```

## Usage

```javascript
const TurndownService = require("turndown-node");

const turndownService = new TurndownService();
const markdown = turndownService.turndown("<h1>Hello World</h1>");
console.log(markdown);
// Hello World
// ===========
```

### Options

````javascript
const turndownService = new TurndownService({
  headingStyle: "atx", // 'setext' (default) or 'atx'
  codeBlockStyle: "fenced", // 'indented' (default) or 'fenced'
  bulletListMarker: "-", // '*' (default), '-', or '+'
  emDelimiter: "*", // '_' (default) or '*'
  strongDelimiter: "__", // '**' (default) or '__'
  fence: "```", // fence for fenced code blocks
  hr: "---", // horizontal rule string
});
````

### API

#### `turndown(html)`

Convert an HTML string to Markdown.

```javascript
turndownService.turndown("<p>Hello <strong>World</strong></p>");
// => "Hello **World**"
```

#### `keep(filter)`

Keep elements as HTML instead of converting them.

```javascript
turndownService.keep(["del", "ins"]);
turndownService.turndown("<p>Hello <del>World</del></p>");
// => "Hello <del>World</del>"
```

#### `remove(filter)`

Remove elements entirely from the output.

```javascript
turndownService.remove(["script", "style"]);
```

#### `escape(text)`

Escape Markdown special characters in a string.

```javascript
turndownService.escape("*not emphasis*");
// => "\\*not emphasis\\*"
```

## Supported Platforms

| Platform | Architecture          | Status |
| -------- | --------------------- | ------ |
| macOS    | ARM64 (Apple Silicon) | ✅     |
| Linux    | x64 (glibc)           | ✅     |
| Linux    | ARM64 (glibc)         | ✅     |
| Windows  | x64                   | ✅     |

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
```

### Project Structure

```
turndown-node/
├── crates/
│   ├── turndown/           # Core Rust library
│   └── turndown-napi/      # NAPI-RS bindings
├── packages/
│   ├── turndown-node/      # Main npm package
│   ├── darwin-arm64/       # macOS ARM64 native binding
│   ├── linux-x64-gnu/      # Linux x64 native binding
│   ├── linux-arm64-gnu/    # Linux ARM64 native binding
│   └── win32-x64-msvc/     # Windows x64 native binding
└── tests/                  # Jest parity tests
```

## Compatibility

This is a 1:1 compatible port of [turndown](https://www.npmjs.com/package/turndown) v7.2.0. All CommonMark elements produce identical output.

## License

MIT
