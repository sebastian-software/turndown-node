# Contributing to turndown-node

Thank you for your interest in contributing to turndown-node! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Node.js >= 22
- Rust >= 1.70
- pnpm >= 10

### Getting Started

```bash
# Clone the repository
git clone https://github.com/sebastian-software/turndown-node.git
cd turndown-node

# Install dependencies
pnpm install

# Build the native module
pnpm build

# Run tests
pnpm test
```

## Project Structure

```
turndown-node/
├── crates/
│   ├── turndown-cdp/     # Core Rust library (published to crates.io)
│   └── turndown-napi/    # NAPI-RS bindings + HTML parsing
├── packages/
│   ├── turndown-node/    # Main npm package
│   └── */                # Platform-specific binaries
└── tests/                # JavaScript parity tests
```

## Development Workflow

### Making Changes

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `pnpm test && cargo test --workspace`
5. Commit with conventional commits: `feat: add new feature`
6. Push and create a Pull Request

### Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `test:` Test additions or changes
- `refactor:` Code refactoring
- `chore:` Maintenance tasks

### Running Tests

```bash
# Run all tests
pnpm test

# Run only Rust tests
cargo test --workspace

# Run only JavaScript parity tests
pnpm test:js

# Run tests in watch mode
pnpm --filter turndown-node-tests test:watch
```

### Building

```bash
# Build native module for current platform
pnpm build

# Build for specific target
cd crates/turndown-napi
pnpm build --target aarch64-apple-darwin
```

## Parity with turndown

This project aims for 100% compatibility with [turndown](https://github.com/mixmark-io/turndown) v7.2.0. When making changes:

1. Ensure parity tests pass
2. Add new parity tests for new functionality
3. Document any intentional deviations

## Code Style

### Rust

- Follow standard Rust formatting: `cargo fmt`
- Run clippy: `cargo clippy`

### JavaScript/TypeScript

- Use Prettier for formatting: `pnpm format`
- Follow ESLint rules: `pnpm lint`

## Releasing

Releases are automated via [Release Please](https://github.com/googleapis/release-please). When PRs are merged to `main`:

1. Release Please creates/updates a release PR
2. Merging the release PR triggers publishing to npm and crates.io

## Getting Help

- Open an [issue](https://github.com/sebastian-software/turndown-node/issues) for bugs or feature requests
- Start a [discussion](https://github.com/sebastian-software/turndown-node/discussions) for questions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
