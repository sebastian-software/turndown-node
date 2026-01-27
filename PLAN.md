# turndown-node

Ein 1:1 Rust-Port von [turndown](https://github.com/mixmark-io/turndown) mit Node.js Bindings.

## Übersicht

Turndown ist eine JavaScript-Bibliothek, die HTML in Markdown konvertiert. Dieses Projekt portiert die gesamte Funktionalität nach Rust und bietet native Node.js Bindings über NAPI-RS.

## Projektstruktur (pnpm Monorepo)

```
turndown-node/
├── package.json                  # Root package.json (private, workspaces)
├── pnpm-workspace.yaml           # pnpm Workspace-Konfiguration
├── pnpm-lock.yaml
├── Cargo.toml                    # Rust Workspace-Konfiguration
├── release-please-config.json
├── .release-please-manifest.json
│
├── crates/
│   ├── turndown/                 # Rust-Kernbibliothek (turndown)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── service.rs        # TurndownService
│   │       ├── rules/
│   │       │   ├── mod.rs
│   │       │   ├── commonmark.rs # CommonMark-Regeln
│   │       │   └── rule.rs       # Rule Trait & Structs
│   │       ├── node.rs           # Node-Erweiterungen
│   │       ├── whitespace.rs     # collapse-whitespace
│   │       └── utilities.rs      # Hilfsfunktionen
│   │
│   └── turndown-napi/            # Node.js Bindings (Build)
│       ├── Cargo.toml
│       ├── package.json          # @turndown-node/napi (intern)
│       ├── src/
│       │   └── lib.rs
│       └── build.rs
│
├── packages/
│   ├── turndown-node/            # turndown-node (Haupt-Paket)
│   │   ├── package.json
│   │   ├── index.js
│   │   ├── index.d.ts
│   │   └── README.md
│   │
│   ├── darwin-arm64/             # @turndown-node/darwin-arm64
│   │   └── package.json
│   │
│   ├── linux-x64-gnu/            # @turndown-node/linux-x64-gnu
│   │   └── package.json
│   │
│   ├── linux-arm64-gnu/          # @turndown-node/linux-arm64-gnu
│   │   └── package.json
│   │
│   └── win32-x64-msvc/           # @turndown-node/win32-x64-msvc
│       └── package.json
│
└── tests/                        # JavaScript Tests
    ├── package.json
    ├── jest.config.js
    ├── upstream/                 # Synchronisierte Turndown-Tests
    ├── parity/                   # Parity-Tests
    ├── fixtures/
    └── sync-tests.js
```

## Root-Konfiguration

### `package.json` (Root)

```json
{
  "name": "turndown-node-monorepo",
  "private": true,
  "packageManager": "pnpm@9.15.0",
  "scripts": {
    "build": "pnpm -r build",
    "build:native": "pnpm --filter @turndown-node/napi build",
    "test": "pnpm -r test",
    "test:rust": "cargo test --workspace",
    "test:js": "pnpm --filter turndown-node-tests test",
    "sync-tests": "pnpm --filter turndown-node-tests sync-tests",
    "clean": "pnpm -r clean && cargo clean",
    "typecheck": "pnpm -r typecheck"
  },
  "devDependencies": {
    "typescript": "^5.0.0"
  },
  "engines": {
    "node": ">=18",
    "pnpm": ">=9"
  }
}
```

### `pnpm-workspace.yaml`

```yaml
packages:
  - 'packages/*'
  - 'crates/turndown-napi'
  - 'tests'
```

### `Cargo.toml` (Workspace)

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.0.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/anthropics/turndown-node"

[workspace.dependencies]
html5ever = "0.27"
markup5ever_rcdom = "0.3"
ego-tree = "0.6"
regex = "1"
once_cell = "1"
thiserror = "1"
```

---

## Phase 1: Rust-Kernbibliothek

### 1.1 Dependencies (`crates/turndown/Cargo.toml`)

```toml
[package]
name = "turndown"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Convert HTML to Markdown"

[dependencies]
html5ever = { workspace = true }
markup5ever_rcdom = { workspace = true }
ego-tree = { workspace = true }
regex = { workspace = true }
once_cell = { workspace = true }
thiserror = { workspace = true }
```

### 1.2 Optionen (TurndownOptions)

```rust
pub struct TurndownOptions {
    /// "setext" oder "atx"
    pub heading_style: HeadingStyle,

    /// "---", "***", "___"
    pub hr: String,

    /// "*", "-", "+"
    pub bullet_list_marker: char,

    /// "indented" oder "fenced"
    pub code_block_style: CodeBlockStyle,

    /// "```" oder "~~~"
    pub fence: String,

    /// "_" oder "*"
    pub em_delimiter: char,

    /// "**" oder "__"
    pub strong_delimiter: String,

    /// "inlined" oder "referenced"
    pub link_style: LinkStyle,

    /// "full", "collapsed", "shortcut"
    pub link_reference_style: LinkReferenceStyle,
}
```

### 1.3 Service (TurndownService)

```rust
pub struct TurndownService {
    options: TurndownOptions,
    rules: Rules,
}

impl TurndownService {
    pub fn new() -> Self;
    pub fn with_options(options: TurndownOptions) -> Self;

    /// Hauptkonvertierung
    pub fn turndown(&self, input: &str) -> Result<String, TurndownError>;

    /// Regelmanagement
    pub fn add_rule(&mut self, key: &str, rule: Rule) -> &mut Self;
    pub fn keep(&mut self, filter: Filter) -> &mut Self;
    pub fn remove(&mut self, filter: Filter) -> &mut Self;

    /// Plugin-System
    pub fn use_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self;

    /// Escaping
    pub fn escape(&self, input: &str) -> String;
}
```

### 1.4 Regeln (Rules)

```rust
pub struct Rule {
    pub filter: Filter,
    pub replacement: Box<dyn Fn(&Node, &str, &TurndownOptions) -> String + Send + Sync>,
}

pub enum Filter {
    TagName(String),
    TagNames(Vec<String>),
    Predicate(Box<dyn Fn(&Node) -> bool + Send + Sync>),
}

pub struct Rules {
    custom_rules: IndexMap<String, Rule>,
    keep_rules: Vec<Filter>,
    remove_rules: Vec<Filter>,
    commonmark_rules: Vec<(&'static str, Rule)>,
}
```

### 1.5 CommonMark-Regeln

| Regel | HTML-Elemente | Markdown-Output |
|-------|---------------|-----------------|
| `paragraph` | `p` | `\n\n{content}\n\n` |
| `line_break` | `br` | `  \n` |
| `heading` | `h1`-`h6` | `# ` oder Setext |
| `blockquote` | `blockquote` | `> ` |
| `list` | `ul`, `ol` | Listen |
| `list_item` | `li` | `* ` oder `1. ` |
| `indented_code_block` | `pre > code` | 4 Spaces |
| `fenced_code_block` | `pre > code` | ``` |
| `horizontal_rule` | `hr` | `---` |
| `inline_link` | `a` | `[text](url)` |
| `reference_link` | `a` | `[text][ref]` |
| `emphasis` | `em`, `i` | `_text_` |
| `strong` | `strong`, `b` | `**text**` |
| `code` | `code` | `` `code` `` |
| `image` | `img` | `![alt](src)` |

### 1.6 Utilities

```rust
pub const BLOCK_ELEMENTS: &[&str] = &[
    "address", "article", "aside", "audio", "blockquote", "body", "canvas",
    "center", "dd", "dir", "div", "dl", "dt", "fieldset", "figcaption",
    "figure", "footer", "form", "frameset", "h1", "h2", "h3", "h4", "h5",
    "h6", "header", "hgroup", "hr", "html", "isindex", "li", "main", "menu",
    "nav", "noframes", "noscript", "ol", "output", "p", "pre", "section",
    "table", "tbody", "td", "tfoot", "th", "thead", "tr", "ul"
];

pub const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input",
    "keygen", "link", "meta", "param", "source", "track", "wbr"
];

pub fn is_block(node: &Node) -> bool;
pub fn is_void(node: &Node) -> bool;
pub fn collapse_whitespace(node: &mut Node);
```

---

## Phase 2: Node.js Bindings (NAPI-RS)

### 2.1 NAPI Crate (`crates/turndown-napi/Cargo.toml`)

```toml
[package]
name = "turndown-napi"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
turndown = { path = "../turndown" }
napi = { version = "2", features = ["napi4"] }
napi-derive = "2"

[build-dependencies]
napi-build = "2"
```

### 2.2 Build-Paket (`crates/turndown-napi/package.json`)

```json
{
  "name": "@turndown-node/napi",
  "private": true,
  "version": "0.0.0",
  "scripts": {
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform"
  },
  "devDependencies": {
    "@napi-rs/cli": "^2.18.0"
  },
  "napi": {
    "binaryName": "turndown",
    "targets": [
      "aarch64-apple-darwin",
      "x86_64-unknown-linux-gnu",
      "aarch64-unknown-linux-gnu",
      "x86_64-pc-windows-msvc"
    ]
  }
}
```

### 2.3 Haupt-Paket (`packages/turndown-node/package.json`)

```json
{
  "name": "turndown-node",
  "version": "0.0.0",
  "description": "Convert HTML to Markdown - Native Node.js bindings for turndown, powered by Rust",
  "main": "index.js",
  "types": "index.d.ts",
  "repository": {
    "type": "git",
    "url": "git+https://github.com/anthropics/turndown-node.git"
  },
  "keywords": ["html", "markdown", "turndown", "converter", "napi", "rust"],
  "license": "MIT",
  "engines": {
    "node": ">=18"
  },
  "optionalDependencies": {
    "@turndown-node/darwin-arm64": "0.0.0",
    "@turndown-node/linux-x64-gnu": "0.0.0",
    "@turndown-node/linux-arm64-gnu": "0.0.0",
    "@turndown-node/win32-x64-msvc": "0.0.0"
  }
}
```

### 2.4 Plattform-Paket (z.B. `packages/darwin-arm64/package.json`)

```json
{
  "name": "@turndown-node/darwin-arm64",
  "version": "0.0.0",
  "os": ["darwin"],
  "cpu": ["arm64"],
  "main": "turndown.darwin-arm64.node",
  "files": ["turndown.darwin-arm64.node"],
  "license": "MIT",
  "engines": {
    "node": ">=18"
  }
}
```

### 2.5 Entry Point (`packages/turndown-node/index.js`)

```javascript
const { platform, arch } = process;

const platformArchMap = {
  darwin: {
    arm64: '@turndown-node/darwin-arm64',
  },
  linux: {
    arm64: '@turndown-node/linux-arm64-gnu',
    x64: '@turndown-node/linux-x64-gnu',
  },
  win32: {
    x64: '@turndown-node/win32-x64-msvc',
  },
};

function loadNativeBinding() {
  const packageName = platformArchMap[platform]?.[arch];

  if (!packageName) {
    throw new Error(
      `Unsupported platform: ${platform}-${arch}. ` +
      `Supported: darwin-arm64, linux-x64, linux-arm64, win32-x64. ` +
      `Please open an issue at https://github.com/anthropics/turndown-node/issues`
    );
  }

  try {
    return require(packageName);
  } catch (e) {
    throw new Error(
      `Failed to load native binding for ${platform}-${arch}.\n` +
      `Package: ${packageName}\n` +
      `Error: ${e.message}\n\n` +
      `Try reinstalling with: npm install turndown-node`
    );
  }
}

const nativeBinding = loadNativeBinding();

module.exports = nativeBinding.TurndownService;
module.exports.TurndownService = nativeBinding.TurndownService;
module.exports.default = nativeBinding.TurndownService;
```

### 2.6 TypeScript-Definitionen (`packages/turndown-node/index.d.ts`)

```typescript
export interface Options {
  headingStyle?: 'setext' | 'atx';
  hr?: string;
  bulletListMarker?: '*' | '-' | '+';
  codeBlockStyle?: 'indented' | 'fenced';
  fence?: '```' | '~~~';
  emDelimiter?: '_' | '*';
  strongDelimiter?: '**' | '__';
  linkStyle?: 'inlined' | 'referenced';
  linkReferenceStyle?: 'full' | 'collapsed' | 'shortcut';
}

export interface Rule {
  filter: string | string[] | ((node: Node) => boolean);
  replacement: (content: string, node: Node, options: Options) => string;
}

export class TurndownService {
  constructor(options?: Options);
  turndown(html: string): string;
  addRule(key: string, rule: Rule): this;
  keep(filter: string | string[]): this;
  remove(filter: string | string[]): this;
  use(plugin: (service: TurndownService) => void): this;
  escape(str: string): string;
}

export default TurndownService;
```

---

## Phase 3: Tests (JavaScript)

Die Tests bleiben in JavaScript und laufen gegen die Node.js Bindings.

### 3.1 Test-Synchronisations-Script (`tests/sync-tests.js`)

```javascript
#!/usr/bin/env node

const https = require('https');
const fs = require('fs');
const path = require('path');

const TURNDOWN_REPO = 'mixmark-io/turndown';
const TURNDOWN_BRANCH = 'master';
const FILES_TO_SYNC = [
  'test/turndown-test.js',
  'test/internals-test.js',
];

const UPSTREAM_DIR = path.join(__dirname, 'upstream');

async function fetchFile(filePath) {
  const url = `https://raw.githubusercontent.com/${TURNDOWN_REPO}/${TURNDOWN_BRANCH}/${filePath}`;

  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => resolve(data));
      res.on('error', reject);
    });
  });
}

async function getLatestCommit() {
  const url = `https://api.github.com/repos/${TURNDOWN_REPO}/commits/${TURNDOWN_BRANCH}`;

  return new Promise((resolve, reject) => {
    https.get(url, { headers: { 'User-Agent': 'turndown-node-sync' } }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        const json = JSON.parse(data);
        resolve({
          sha: json.sha.substring(0, 8),
          date: json.commit.committer.date,
          message: json.commit.message.split('\n')[0]
        });
      });
      res.on('error', reject);
    });
  });
}

function adaptTestFile(content, filename) {
  let adapted = content
    .replace(
      /require\(['"]turndown['"]\)/g,
      "require('turndown-node')"
    )
    .replace(
      /import TurndownService from ['"]turndown['"]/g,
      "import TurndownService from 'turndown-node'"
    );

  const header = `/**
 * AUTO-GENERATED - DO NOT EDIT
 * Synchronized from: https://github.com/${TURNDOWN_REPO}/blob/${TURNDOWN_BRANCH}/test/${filename}
 * Run: pnpm sync-tests
 */

`;

  return header + adapted;
}

async function syncTests() {
  console.log('Syncing tests from turndown repository...\n');

  if (!fs.existsSync(UPSTREAM_DIR)) {
    fs.mkdirSync(UPSTREAM_DIR, { recursive: true });
  }

  const commit = await getLatestCommit();
  console.log(`Latest commit: ${commit.sha} (${commit.date})`);

  for (const filePath of FILES_TO_SYNC) {
    const filename = path.basename(filePath);
    console.log(`Fetching ${filename}...`);

    const content = await fetchFile(filePath);
    const adapted = adaptTestFile(content, filename);
    fs.writeFileSync(path.join(UPSTREAM_DIR, filename), adapted);
  }

  fs.writeFileSync(
    path.join(UPSTREAM_DIR, '.turndown-version'),
    JSON.stringify({ sha: commit.sha, syncedAt: new Date().toISOString() }, null, 2)
  );

  console.log('\nSync complete!');
}

syncTests().catch(console.error);
```

### 3.2 Parity-Tests (`tests/parity/parity.test.js`)

```javascript
const TurndownNode = require('turndown-node');
const TurndownOriginal = require('turndown');

describe('turndown-node vs turndown parity', () => {
  const testCases = [
    '<p>Hello World</p>',
    '<h1>Heading</h1>',
    '<a href="https://example.com">Link</a>',
    '<strong><em>Bold and italic</em></strong>',
    '<ul><li>One</li><li>Two</li></ul>',
    '<pre><code>code block</code></pre>',
    '<blockquote><p>Quote</p></blockquote>',
    '<hr>',
    '<img src="test.png" alt="Alt">',
  ];

  testCases.forEach((html, i) => {
    it(`case ${i + 1}: produces identical output`, () => {
      const node = new TurndownNode();
      const original = new TurndownOriginal();

      expect(node.turndown(html)).toBe(original.turndown(html));
    });
  });
});
```

### 3.3 Test-Konfiguration (`tests/package.json`)

```json
{
  "name": "turndown-node-tests",
  "private": true,
  "scripts": {
    "test": "jest",
    "test:watch": "jest --watch",
    "test:parity": "jest parity/",
    "test:upstream": "jest upstream/",
    "sync-tests": "node sync-tests.js"
  },
  "devDependencies": {
    "jest": "^29.0.0",
    "turndown": "^7.1.2",
    "turndown-node": "workspace:*"
  }
}
```

---

## Phase 4: Build & Release

### 4.1 CI Workflow (`.github/workflows/ci.yml`)

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace

  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
            package: darwin-arm64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            package: linux-x64-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            package: linux-arm64-gnu
            use-cross: true
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            package: win32-x64-msvc

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'pnpm'

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}

      - run: pnpm install

      - name: Install cross
        if: matrix.use-cross
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build
        run: |
          if [ "${{ matrix.use-cross }}" = "true" ]; then
            pnpm --filter @turndown-node/napi build -- --target ${{ matrix.target }} --use-cross
          else
            pnpm --filter @turndown-node/napi build -- --target ${{ matrix.target }}
          fi
        shell: bash

      - run: cp crates/turndown-napi/*.node packages/${{ matrix.package }}/
        shell: bash

      - uses: actions/upload-artifact@v4
        with:
          name: bindings-${{ matrix.target }}
          path: packages/${{ matrix.package }}/*.node

  test-js:
    runs-on: ubuntu-latest
    needs: [build]
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'pnpm'
      - uses: actions/download-artifact@v4
        with:
          name: bindings-x86_64-unknown-linux-gnu
          path: packages/linux-x64-gnu/
      - run: pnpm install
      - run: pnpm sync-tests
      - run: pnpm test:js
```

### 4.2 Release Workflow (`.github/workflows/release.yml`)

```yaml
name: Release

on:
  push:
    branches: [main]

permissions:
  contents: write
  pull-requests: write
  id-token: write

jobs:
  release-please:
    runs-on: ubuntu-latest
    outputs:
      release_created: ${{ steps.release.outputs.release_created }}
      version: ${{ steps.release.outputs.version }}
    steps:
      - uses: googleapis/release-please-action@v4
        id: release
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          config-file: release-please-config.json
          manifest-file: .release-please-manifest.json

  build:
    needs: release-please
    if: ${{ needs.release-please.outputs.release_created }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
            package: darwin-arm64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            package: linux-x64-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            package: linux-arm64-gnu
            use-cross: true
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            package: win32-x64-msvc

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'pnpm'
          registry-url: 'https://registry.npmjs.org'

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2

      - run: pnpm install

      - name: Install cross
        if: matrix.use-cross
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build
        run: |
          if [ "${{ matrix.use-cross }}" = "true" ]; then
            pnpm --filter @turndown-node/napi build -- --target ${{ matrix.target }} --use-cross
          else
            pnpm --filter @turndown-node/napi build -- --target ${{ matrix.target }}
          fi
        shell: bash

      - run: cp crates/turndown-napi/*.node packages/${{ matrix.package }}/
        shell: bash

      - name: Publish platform package
        working-directory: packages/${{ matrix.package }}
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: pnpm publish --access public --provenance --no-git-checks

  publish-main:
    needs: [release-please, build]
    if: ${{ needs.release-please.outputs.release_created }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'pnpm'
          registry-url: 'https://registry.npmjs.org'

      - run: pnpm install

      - name: Publish main package
        working-directory: packages/turndown-node
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: pnpm publish --access public --provenance --no-git-checks

      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: cargo publish -p turndown --allow-dirty
```

### 4.3 Release-Please Konfiguration

`release-please-config.json`:
```json
{
  "$schema": "https://raw.githubusercontent.com/googleapis/release-please/main/schemas/config.json",
  "packages": {
    ".": {
      "release-type": "node",
      "package-name": "turndown-node",
      "changelog-path": "CHANGELOG.md",
      "bump-minor-pre-major": true,
      "extra-files": [
        "packages/turndown-node/package.json",
        "packages/darwin-arm64/package.json",
        "packages/linux-x64-gnu/package.json",
        "packages/linux-arm64-gnu/package.json",
        "packages/win32-x64-msvc/package.json",
        {
          "type": "toml",
          "path": "crates/turndown/Cargo.toml",
          "jsonpath": "$.package.version"
        }
      ]
    }
  }
}
```

`.release-please-manifest.json`:
```json
{
  ".": "0.0.1"
}
```

---

## Implementierungsreihenfolge

### Phase 1: Grundstruktur
- [ ] Repository umbenennen/erstellen
- [ ] pnpm Workspace Setup
- [ ] Cargo Workspace Setup
- [ ] Basis-Struktur aller Pakete

### Phase 2: Rust-Kernbibliothek
- [ ] HTML-Parsing mit `html5ever`
- [ ] `TurndownService` Grundstruktur
- [ ] `Rule` und `Filter` System
- [ ] Alle 15 CommonMark-Regeln
- [ ] Whitespace-Kollabierung
- [ ] Escape-Funktion

### Phase 3: Node.js Bindings
- [ ] NAPI-RS Setup
- [ ] API-Mapping Rust → JS
- [ ] TypeScript-Definitionen
- [ ] Entry Point mit Platform-Detection

### Phase 4: Tests
- [ ] Test-Sync Script
- [ ] Upstream-Tests synchronisieren
- [ ] Parity-Tests gegen Original

### Phase 5: CI/CD & Release
- [ ] GitHub Actions Workflows
- [ ] Release-Please Konfiguration
- [ ] npm Scope `@turndown-node` erstellen
- [ ] Erstes Release

---

## Referenzen

- [turndown Source](https://github.com/mixmark-io/turndown)
- [html5ever](https://github.com/servo/html5ever) - Firefox HTML Parser in Rust
- [NAPI-RS](https://napi.rs/) - Node.js Bindings für Rust
- [CommonMark Spec](https://spec.commonmark.org/)
