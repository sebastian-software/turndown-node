# Refactoring Plan: Streaming Architecture

## Ziel

HTML → Markdown Konvertierung optimieren durch:

1. Streaming HTML Parsing (lol_html statt DOM-Tree)
2. Gemeinsamer Markdown AST als Zwischenformat
3. Geteilte Serialisierung

## Neue Architektur

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  HTML String ───scraper────▶ ┌──────────────┐                   │
│       (turndown-napi)        │              │                   │
│                              │ Markdown AST │ ──▶ Markdown String
│  CDP Node Tree ─────────────▶│ (turndown-   │                   │
│       (turndown-cdp)         │    core)     │                   │
│                              └──────────────┘                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: turndown-core erstellen

### 1.1 Crate Setup

- [x] `crates/turndown-core/Cargo.toml` erstellen
- [x] `crates/turndown-core/src/lib.rs` erstellen
- [x] Workspace Cargo.toml aktualisieren (automatisch via `crates/*`)

### 1.2 Markdown AST definieren

- [x] `crates/turndown-core/src/ast.rs` - Block-Level Nodes
  - Document, Heading, Paragraph, List, ListItem
  - CodeBlock, Blockquote, Table, ThematicBreak, HtmlBlock
- [x] `crates/turndown-core/src/ast.rs` - Inline Nodes
  - Text, Strong, Emphasis, Code, Link, Image, HtmlInline

### 1.3 Serialisierung

- [x] `crates/turndown-core/src/serialize.rs` - AST → String
- [x] Tests für Serialisierung (16 Tests)

### 1.4 Options

- [x] `crates/turndown-core/src/options.rs` - Gemeinsame Optionen
  - HeadingStyle, CodeBlockStyle, LinkStyle, etc.
  - Von `turndown-cdp` übernehmen

**Checkpoint 1**: `cargo build -p turndown-core` funktioniert ✅

---

## Phase 2: turndown-cdp refactoren

### 2.1 Dependency hinzufügen

- [x] `turndown-core` als Dependency in `turndown-cdp/Cargo.toml`

### 2.2 Konvertierung implementieren

- [x] `crates/turndown-cdp/src/convert.rs` - CDP Node → MdNode
- [x] Bestehende Regeln (commonmark.rs) entfernt, Logik in convert.rs

### 2.3 Service anpassen

- [x] `TurndownService::turndown()` nutzt jetzt:
  - `convert::convert(node)` → Block (Markdown AST)
  - `turndown_core::serialize(ast)` → String

### 2.4 Tests

- [x] 27 Rust Tests grün (`cargo test -p turndown-cdp`)
- [x] 18 Parity Tests grün (`pnpm test`)

**Checkpoint 2**: `cargo test -p turndown-cdp` grün, API unverändert ✅

---

## Phase 3: Direkte AST-Konvertierung in turndown-napi

### 3.1 Dependencies anpassen

- [x] `turndown-core` als Dependency (statt `turndown-cdp`)
- [x] `scraper` für HTML Parsing beibehalten

> **Hinweis**: lol_html wurde evaluiert, aber die v2 API hatte Breaking Changes
> (fehlendes `on_end_tag`). Daher pragmatischer Ansatz: scraper → AST direkt.

### 3.2 Direkte Konvertierung

- [x] `crates/turndown-napi/src/streaming.rs`
  - HTML → scraper DOM → Markdown AST (ohne turndown-cdp Umweg)
  - Alle HTML-Elemente direkt auf Block/Inline gemappt
  - 9 Unit Tests

### 3.3 Integration

- [x] `TurndownService::turndown()` nutzt `streaming::html_to_ast()`
- [x] turndown-cdp Dependency entfernt (NAPI-Pfad unabhängig)

### 3.4 Tests

- [x] 18 Parity Tests grün (`pnpm test`)
- [x] Performance: ~3.5x Speedup (stabil)

**Checkpoint 3**: `pnpm test` grün ✅

---

## Phase 4: Cleanup & Benchmarks

### 4.1 Aufräumen

- [x] Ungenutzte Dependencies entfernen
- [x] Code Review: Unused methods entfernt (`enter_pre`, `enter_code`, `in_code`)

### 4.2 Benchmarks

- [x] Neue Benchmarks laufen lassen
- [x] Ergebnis: **3.49x average speedup**
  - simple (36 bytes): 5.91x
  - blog-post (2.5KB): 2.94x
  - documentation (4.3KB): 2.52x
  - large-article (28.6KB): 2.59x

### 4.3 Dokumentation

- [x] Crate-Level Dokumentation (lib.rs doc comments)
- [ ] README.md aktualisieren (optional, bestehende Docs ausreichend)

**Checkpoint 4**: Alles grün, Performance dokumentiert ✅

---

## Risiken & Fallbacks

| Risiko                 | Mitigation                        |
| ---------------------- | --------------------------------- |
| lol_html Edge Cases    | Parity Tests fangen Probleme früh |
| API Breaking Changes   | Phase 2 hält CDP API stabil       |
| Performance schlechter | Benchmarks nach jeder Phase       |
| Stuck in Refactoring   | Checkpoints erlauben Rollback     |

## Geschätzte Komplexität

| Phase | Dateien    | Komplexität |
| ----- | ---------- | ----------- |
| 1     | 4-5 neue   | Mittel      |
| 2     | 3-4 ändern | Mittel      |
| 3     | 2-3 neue   | Hoch        |
| 4     | diverse    | Niedrig     |

---

## Aktueller Status

**Phase**: 4 abgeschlossen ✅
**Letzter Checkpoint**: 4 ✅ (Cleanup, Benchmarks dokumentiert, alle Tests grün)

### Ergebnis

Refactoring erfolgreich abgeschlossen:

- **turndown-core**: Gemeinsamer Markdown AST + Serialisierung
- **turndown-cdp**: CDP Node → AST Konvertierung (für chromiumoxide)
- **turndown-napi**: Direkter HTML → AST Pfad (scraper)
- **Performance**: 3.49x durchschnittlicher Speedup vs. JS
