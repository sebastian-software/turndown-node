# ADR-002: AST-basierte Architektur

**Status:** Accepted
**Datum:** 2026-01-28

## Kontext

Bei der HTML-zu-Markdown Konvertierung gibt es zwei grundlegende Architektur-Ansätze:

1. **AST-basiert**: HTML → AST → Markdown
2. **Direct Streaming**: HTML → Markdown (ohne Zwischenrepräsentation)

Wir haben beide Ansätze implementiert und verglichen.

## Entscheidung

**Wir verwenden eine AST-basierte Architektur mit einem Intermediate Representation (IR).**

```
HTML String → [tl Parser] → DOM → [AST Builder] → Block/Inline AST → [Serializer] → Markdown String
```

### Begründung

#### Profilierung zeigte: AST-Building ist nicht der Bottleneck

Detaillierte Profiling-Ergebnisse für ein 188KB HTML-Dokument:

| Phase         | Zeit   | Anteil    |
| ------------- | ------ | --------- |
| Parsing (tl)  | ~0.3ms | ~15-20%   |
| AST Building  | ~0.7ms | ~35-40%   |
| Serialization | ~0.6ms | ~35-40%   |
| **Gesamt**    | ~1.6ms | ~85-90%\* |

\*~10-15% unaccounted overhead (Memory alloc, etc.)

#### Direct Streaming brachte nur marginale Verbesserung

Ein Proof-of-Concept ohne AST zeigte:

- **Nur ~20% schneller** (1.25x Speedup)
- Bei einem Gesamtspeedup von 3.5x gegenüber JavaScript ist das marginal

#### AST ermöglicht wichtige Features

1. **Options-Support**: Verschiedene Markdown-Stile (ATX vs Setext Headings, etc.)
2. **Link Reference Collection**: Für `LinkStyle::Referenced`
3. **Testbarkeit**: AST kann unabhängig von Serialization getestet werden
4. **Erweiterbarkeit**: Custom Rules können auf AST-Ebene arbeiten
5. **Debugging**: AST kann zur Fehleranalyse inspiziert werden

## AST-Struktur

```rust
pub enum Block {
    Document(Vec<Block>),
    Paragraph(Vec<Inline>),
    Heading { level: u8, content: Vec<Inline> },
    BlockQuote(Vec<Block>),
    List { ordered: bool, start: u32, items: Vec<ListItem> },
    CodeBlock { language: Option<String>, code: String, fenced: bool },
    ThematicBreak,
    Table { headers: Vec<Vec<Inline>>, rows: Vec<Vec<Vec<Inline>>> },
}

pub enum Inline {
    Text(String),
    Strong(Vec<Inline>),
    Emphasis(Vec<Inline>),
    Code(String),
    Link { content: Vec<Inline>, url: String, title: Option<String> },
    Image { url: String, alt: String, title: Option<String> },
    LineBreak,
}
```

## Konsequenzen

### Positiv

- Klare Trennung der Concerns (Parsing, Building, Serializing)
- Einfach zu testen und zu debuggen
- Erweiterbar für zukünftige Features
- Options können sauber auf Serialization-Ebene angewendet werden

### Negativ

- Zusätzlicher Memory-Overhead für AST-Struktur
- Ein zusätzlicher Traversal-Pass (AST → Markdown)
- ~20% langsamer als theoretisch mögliches Direct Streaming

### Trade-off Analyse

| Aspekt          | AST-basiert  | Direct Streaming |
| --------------- | ------------ | ---------------- |
| Performance     | ~1.6ms/188KB | ~1.3ms/188KB     |
| Wartbarkeit     | Hoch         | Niedrig          |
| Testbarkeit     | Hoch         | Niedrig          |
| Erweiterbarkeit | Hoch         | Niedrig          |
| Memory          | ~2x Dokument | ~1x Dokument     |

**Fazit:** Der ~20% Performance-Verlust ist akzeptabel angesichts der deutlich besseren Wartbarkeit und Erweiterbarkeit.

## Alternativen

1. **SAX-Style Events** - Komplexer, schwierig für verschachtelte Strukturen
2. **Lazy AST** - Nodes erst bei Bedarf konstruieren (premature optimization)
3. **Hybrid** - Einfache Elemente streamen, komplexe via AST (zu komplex)

## Referenzen

- [turndown-core AST types](../crates/turndown-core/src/types.rs)
- [Direct Streaming PoC](commit: removed, showed only 1.25x improvement)
