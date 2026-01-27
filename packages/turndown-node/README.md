# turndown-node

[![npm version](https://img.shields.io/npm/v/turndown-node.svg)](https://www.npmjs.com/package/turndown-node)
[![CI](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml/badge.svg)](https://github.com/sebastian-software/turndown-node/actions/workflows/ci.yml)

Convert HTML to Markdown - Native Node.js bindings powered by Rust.

**100% compatible** with [turndown](https://github.com/mixmark-io/turndown) v7.2.0 - drop-in replacement with identical output.

## Installation

```bash
npm install turndown-node
# or
pnpm add turndown-node
# or
yarn add turndown-node
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

### ESM

```javascript
import TurndownService from "turndown-node";

const turndownService = new TurndownService();
const markdown = turndownService.turndown(
  "<p>Hello <strong>World</strong></p>"
);
// => "Hello **World**"
```

## Options

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

## API

### `turndown(html)`

Convert an HTML string to Markdown.

```javascript
turndownService.turndown("<p>Hello <strong>World</strong></p>");
// => "Hello **World**"
```

### `keep(filter)`

Keep elements as HTML instead of converting them.

```javascript
turndownService.keep(["del", "ins"]);
turndownService.turndown("<p>Hello <del>World</del></p>");
// => "Hello <del>World</del>"
```

### `remove(filter)`

Remove elements entirely from the output.

```javascript
turndownService.remove(["script", "style"]);
turndownService.turndown("<p>Hello</p><script>alert('!')</script>");
// => "Hello"
```

### `escape(text)`

Escape Markdown special characters in a string.

```javascript
turndownService.escape("*not emphasis*");
// => "\\*not emphasis\\*"
```

## Supported Platforms

| Platform | Architecture          | Supported |
| -------- | --------------------- | --------- |
| macOS    | ARM64 (Apple Silicon) | ✅        |
| Linux    | x64 (glibc)           | ✅        |
| Linux    | ARM64 (glibc)         | ✅        |
| Windows  | x64                   | ✅        |

## Performance

Built with Rust and [NAPI-RS](https://napi.rs) for native performance. Uses [html5ever](https://github.com/servo/html5ever) (the Firefox HTML parser) via the [scraper](https://crates.io/crates/scraper) crate.

## Compatibility

This is a 1:1 compatible port of [turndown](https://www.npmjs.com/package/turndown) v7.2.0. All CommonMark elements produce identical output. The test suite verifies parity with the original library.

## Related

- [`turndown-cdp`](https://crates.io/crates/turndown-cdp) - The underlying Rust crate (for use with chromiumoxide/CDP)
- [turndown](https://github.com/mixmark-io/turndown) - The original JavaScript library

## License

MIT
