# ADR-003: Performance-Optimierungen

**Status:** Accepted
**Datum:** 2026-01-28

## Kontext

Nach der initialen Implementierung haben wir verschiedene Optimierungen evaluiert und implementiert, um die Performance zu verbessern. Das Ziel war ein signifikanter Speedup gegenüber der JavaScript-Referenzimplementierung (turndown.js).

## Entscheidungen

### 1. Writer-Pattern statt String-Konkatenation

**Problem:** Serialization via String-Rückgabe erzeugt viele temporäre Allocations.

**Lösung:** Alle Serialize-Funktionen schreiben in einen `&mut String` Buffer.

```rust
// Vorher (langsam):
fn serialize_block(block: &Block) -> String {
    match block {
        Block::Paragraph(inlines) => serialize_inlines(inlines),
        // Viele temporäre Strings werden erzeugt und konkateniert
    }
}

// Nachher (schnell):
fn serialize_block(block: &Block, out: &mut String) {
    match block {
        Block::Paragraph(inlines) => serialize_inlines(inlines, out),
        // Direkt in den Output-Buffer schreiben
    }
}
```

**Ergebnis:** 3.5x schnellere Serialization (537ms → 153ms für 500 Iterationen)

### 2. SmallVec für Inline-Elemente

**Problem:** Die meisten Inline-Container (z.B. `<strong>`, `<em>`) haben nur 1-4 Kinder. `Vec` alloziert immer auf dem Heap.

**Lösung:** `SmallVec<[Inline; 4]>` speichert bis zu 4 Elemente inline (auf dem Stack).

```rust
type InlineVec = SmallVec<[Inline; 4]>;

// Bei <= 4 Elementen: keine Heap-Allocation
// Bei > 4 Elementen: automatischer Fallback zu Heap
```

**Ergebnis:** ~30% schnelleres AST-Building für typische Dokumente

### 3. Combined Whitespace Collapsing und Markdown Escaping

**Problem:** Zwei separate Passes über jeden Text-Node.

**Lösung:** Single-Pass Funktion die beides kombiniert.

```rust
fn collapse_and_escape(s: &str) -> String {
    const NEEDS_ESCAPE: [bool; 128] = { /* Lookup table */ };

    let mut result = String::with_capacity(s.len());
    let mut prev_ws = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_ws {
                result.push(' ');
                prev_ws = true;
            }
        } else {
            prev_ws = false;
            let b = c as u32;
            if b < 128 && NEEDS_ESCAPE[b as usize] {
                result.push('\\');
            }
            result.push(c);
        }
    }
    result
}
```

**Ergebnis:** ~15% schnellere Text-Verarbeitung

### 4. eq_ignore_ascii_case statt to_lowercase()

**Problem:** `tag_name.to_lowercase()` alloziert einen neuen String.

**Lösung:** `tag_name.eq_ignore_ascii_case("p")` vergleicht in-place.

```rust
// Vorher:
if tag_name.to_lowercase() == "p" { ... }

// Nachher:
if tag_name.eq_ignore_ascii_case("p") { ... }
```

**Ergebnis:** Keine Allocation pro Tag-Vergleich

### 5. Pre-Allocation mit Capacity

**Problem:** Strings und Vecs wachsen dynamisch und re-allozieren.

**Lösung:** Capacity basierend auf Input-Größe schätzen.

```rust
// Output ist typischerweise ähnlich groß wie Input (etwas kleiner)
let mut result = String::with_capacity(4096);

// Bei bekannter Größe direkt allozieren
let mut result = String::with_capacity(s.len());
```

## Nicht implementierte Optimierungen

### SIMD/memchr für Escaping

**Evaluiert:** `memchr` Crate für schnelles Finden von Escape-Charactern.

**Ergebnis:** Kein messbarer Unterschied, da:

- Text-Chunks sind typischerweise kurz (< 1KB)
- SIMD-Overhead überwiegt bei kleinen Inputs
- Die meisten Zeichen müssen nicht escaped werden

### Arena Allocation

**Evaluiert:** Bumpalo oder ähnliche Arena-Allocators.

**Ergebnis:** Nicht implementiert, da:

- Komplexerer Code
- Lifetime-Management wird schwieriger
- Aktuelle Performance ist ausreichend

## Konsequenzen

### Gesamt-Performance

| Phase         | Vorher | Nachher | Verbesserung |
| ------------- | ------ | ------- | ------------ |
| AST Building  | ~1.0ms | ~0.7ms  | 1.4x         |
| Serialization | ~0.9ms | ~0.3ms  | 3x           |
| **Gesamt**    | ~2.9ms | ~1.6ms  | 1.8x         |

### Benchmark-Ergebnisse vs JavaScript

| Dokument         | Größe    | Speedup   |
| ---------------- | -------- | --------- |
| simple           | 36 bytes | 5.80x     |
| blog-post        | 2.5 KB   | 2.87x     |
| documentation    | 4.3 KB   | 2.57x     |
| large-article    | 28.6 KB  | 2.59x     |
| large-100kb      | 188 KB   | 3.48x     |
| **Durchschnitt** |          | **3.46x** |

### Code-Komplexität

- Writer-Pattern: Minimal erhöhte Komplexität
- SmallVec: Transparent, keine API-Änderung
- Combined Functions: Etwas längere Funktionen, aber verständlich

## Lessons Learned

1. **Profilen vor Optimieren**: Parsing war nicht der Bottleneck (nur ~15%)
2. **Allocation-Reduktion > Algorithmus-Optimierung**: Die meisten Gewinne kamen durch weniger Allocations
3. **Lookup-Tables sind schnell**: Compile-time const Arrays für Character-Classification
4. **SIMD lohnt sich erst ab größeren Inputs**: Für typische HTML-Chunks zu viel Overhead

## Referenzen

- [SmallVec Crate](https://crates.io/crates/smallvec)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
