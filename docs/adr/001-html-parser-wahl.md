# ADR-001: HTML Parser Wahl

**Status:** Accepted
**Datum:** 2026-01-28

## Kontext

Für die HTML-zu-Markdown Konvertierung benötigen wir einen HTML Parser in Rust. Im Laufe der Entwicklung wurden drei Parser evaluiert:

1. **scraper** (v0.22) - Basiert auf html5ever (Servo/Mozilla), vollständiger DOM
2. **lol_html** (v2.7) - Cloudflare's Low Output Latency streaming HTML Rewriter
3. **tl** (v0.7) - Schneller, minimalistischer DOM-Parser

## Parser-Historie

### Phase 1: scraper (html5ever/Servo)

**Commit:** `fb4688e` - Initial implementation

```toml
[dependencies]
scraper = "0.22"
```

**Vorteile:**

- Spec-compliant HTML5 Parsing (html5ever ist Servo's Parser)
- Vollständige CSS-Selector Unterstützung
- Gut getestet, weit verbreitet

**Nachteile:**

- Relativ langsam durch vollständige Spec-Compliance
- Großer Dependency-Tree (html5ever, selectors, cssparser, etc.)
- Mehr Features als benötigt (CSS Selectors)

**Performance:** ~3.5x Speedup vs JavaScript (Baseline)

### Phase 2: lol_html (Cloudflare)

**Commit:** `35f2740` - feat: implement lol_html streaming parser

```toml
[dependencies]
lol_html = "2"
```

**Motivation:** Streaming-Parser für geringeren Memory-Footprint

**Problem:** Die API ist für HTML _Rewriting_ designed, nicht für AST-Building.

Das `end_tag_handlers()` API erwartet konkrete Closure-Typen zur Compile-Zeit:

```rust
// Kompiliert NICHT:
if let Some(handlers) = el.end_tag_handlers() {
    handlers.push(Box::new(move |_end| {
        state.borrow_mut().close_element();  // Shared State
        Ok(())
    }));
}
```

**Compile Error:**

```
error[E0271]: type mismatch resolving `<LocalHandlerTypes as HandlerTypes>::EndTagHandler<'static> == Box<...>`
   expected struct `Box<{closure@...}>`
   found struct `Box<(dyn for<'a, 'b> FnOnce(&'a mut EndTag<'b>) -> Result<...>)>`
```

**Fazit:** lol_html ist für Streaming-Rewriting auf CDN-Edge optimiert, nicht für AST-Konstruktion.

### Phase 3: tl (Aktuell)

**Commit:** Aktuell (nach lol_html Revert)

```toml
[dependencies]
tl = "0.7"
```

## Entscheidung

**Wir verwenden `tl` als HTML Parser.**

### Begründung

#### Performance-Vergleich

| Parser              | Parsing-Zeit | Dependencies | Memory              |
| ------------------- | ------------ | ------------ | ------------------- |
| scraper (html5ever) | ~15-20%      | ~15 crates   | Hoch (full DOM)     |
| lol_html            | ~15-20%      | ~8 crates    | Niedrig (streaming) |
| tl                  | ~15-20%      | ~3 crates    | Mittel (lazy DOM)   |

**Alle Parser zeigen ähnliche Parsing-Performance** (~15-20% der Gesamtzeit).

#### tl's Vorteile für unseren Use Case

1. **DOM-Traversal API** passt zu AST-Building:

```rust
fn process_element(dom: &VDom, parser: &Parser, tag: &HTMLTag) -> Option<Block> {
    let children = tag.children();
    for handle in children.top().iter() {
        // Rekursive Verarbeitung
    }
}
```

2. **Minimale Dependencies** - nur 3 direkte Dependencies

3. **Lazy Parsing** - Nodes werden erst bei Zugriff geparst

4. **Zero-Copy wo möglich** - Strings referenzieren Original-Input

#### Warum nicht scraper behalten?

- tl hat weniger Dependencies
- Ähnliche Performance
- Einfachere API für unseren Use Case (keine CSS Selectors nötig)
- scraper's html5ever ist "overkill" für Markdown-Konvertierung

## Konsequenzen

### Positiv

- Einfache, intuitive API für DOM-Traversal
- Minimaler Dependency-Footprint
- Ausreichend schnell (~15-20% der Gesamtzeit)
- Kein komplexes State-Management wie bei lol_html

### Negativ

- Nicht streaming (gesamtes Dokument im Speicher)
- Nicht spec-compliant wie html5ever
- Weniger verbreitet als scraper

### Benchmark-Ergebnisse (Alle Parser)

| Fixture          | Größe   | Speedup vs JS |
| ---------------- | ------- | ------------- |
| simple           | 36 B    | 5.80x         |
| blog-post        | 2.5 KB  | 2.87x         |
| documentation    | 4.3 KB  | 2.57x         |
| large-article    | 28.6 KB | 2.59x         |
| large-100kb      | 188 KB  | 3.48x         |
| **Durchschnitt** |         | **3.46x**     |

_Performance war bei allen drei Parsern nahezu identisch._

## Alternativen (Zusammenfassung)

| Parser              | Status          | Grund                                        |
| ------------------- | --------------- | -------------------------------------------- |
| scraper (html5ever) | Ersetzt         | Zu viele Dependencies, CSS Selectors unnötig |
| lol_html            | Nicht nutzbar   | API für Rewriting, nicht AST-Building        |
| tl                  | **Gewählt**     | Gute Balance aus Performance und Einfachheit |
| quick-xml           | Nicht evaluiert | Für XML, nicht HTML                          |
| roxmltree           | Nicht evaluiert | Für XML, nicht HTML                          |

## Lessons Learned

1. **Streaming ≠ Schneller**: lol_html's Streaming brachte keine Performance-Vorteile
2. **API-Design matters**: lol_html's API machte unseren Use Case unmöglich
3. **Weniger Dependencies = Weniger Probleme**: tl's minimaler Footprint ist ein Vorteil
4. **Parser ist nicht der Bottleneck**: Nur ~15-20% der Zeit wird für Parsing aufgewendet

## Referenzen

- [tl crate](https://crates.io/crates/tl)
- [scraper crate](https://crates.io/crates/scraper)
- [html5ever (Servo)](https://github.com/servo/html5ever)
- [lol_html (Cloudflare)](https://github.com/cloudflare/lol-html)
- Git History: `fb4688e` → `35f2740` → current
