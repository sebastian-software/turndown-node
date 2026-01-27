# turndown-node

Convert HTML to Markdown - Native Node.js bindings for [turndown](https://github.com/mixmark-io/turndown), powered by Rust.

## Installation

```bash
npm install turndown-node
```

## Usage

```javascript
const TurndownService = require('turndown-node');

const turndownService = new TurndownService();
const markdown = turndownService.turndown('<h1>Hello World</h1>');
console.log(markdown); // "Hello World\n==========="
```

## Options

```javascript
const turndownService = new TurndownService({
  headingStyle: 'atx',        // 'setext' (default) or 'atx'
  codeBlockStyle: 'fenced',   // 'indented' (default) or 'fenced'
  bulletListMarker: '-',      // '*' (default), '-', or '+'
  emDelimiter: '*',           // '_' (default) or '*'
  strongDelimiter: '__',      // '**' (default) or '__'
  fence: '```',               // fence for fenced code blocks
  hr: '---',                  // horizontal rule string
});
```

## API

### `turndown(html)`

Convert an HTML string to Markdown.

### `keep(filter)`

Keep elements as HTML instead of converting them.

```javascript
turndownService.keep(['del', 'ins']);
```

### `remove(filter)`

Remove elements entirely from the output.

```javascript
turndownService.remove(['script', 'style']);
```

### `escape(text)`

Escape Markdown special characters in a string.

## Compatibility

This is a 1:1 compatible port of [turndown](https://www.npmjs.com/package/turndown) v7.2.0. The output is identical for all CommonMark elements.

## Performance

Built with Rust and [NAPI-RS](https://napi.rs) for native Node.js bindings. Uses [html5ever](https://github.com/servo/html5ever) (the Firefox HTML parser) for HTML parsing.

## Supported Platforms

- macOS ARM64 (Apple Silicon)
- Linux x64 (glibc)
- Linux ARM64 (glibc)
- Windows x64

## License

MIT
