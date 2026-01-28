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
│  HTML String ───lol_html───▶ ┌──────────────┐                   │
│       (streaming)            │              │                   │
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

## Phase 3: Streaming in turndown-napi

### 3.1 lol_html Dependency

- [ ] `lol_html` zu `turndown-napi/Cargo.toml` hinzufügen

### 3.2 Streaming Konverter

- [ ] `crates/turndown-napi/src/streaming.rs`
  - State-Machine für offene Tags
  - lol_html Handlers → MdNode Aufbau

### 3.3 Integration

- [ ] `TurndownService::turndown()` nutzt Streaming-Pfad
- [ ] Alte scraper-Logik entfernen (oder als Fallback behalten?)

### 3.4 Tests

- [ ] Parity Tests müssen weiterhin passieren
- [ ] `pnpm test`

**Checkpoint 3**: `pnpm test` grün, Performance-Verbesserung messbar

---

## Phase 4: Cleanup & Benchmarks

### 4.1 Aufräumen

- [ ] Ungenutzte Dependencies entfernen (scraper?)
- [ ] Code Review der neuen Struktur

### 4.2 Benchmarks

- [ ] Neue Benchmarks laufen lassen
- [ ] Vergleich vorher/nachher dokumentieren

### 4.3 Dokumentation

- [ ] README.md aktualisieren mit neuer Architektur
- [ ] Crate-Level Dokumentation

**Checkpoint 4**: Alles grün, Performance dokumentiert

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

**Phase**: 2 abgeschlossen, Phase 3 bereit
**Letzter Checkpoint**: 2 ✅ (turndown-cdp refactored, 27+18 Tests grün)
