# ADR-004: NAPI-Integration und Overhead

**Status:** Accepted
**Datum:** 2026-01-28

## Kontext

turndown-node ist eine Rust-Bibliothek, die via NAPI-RS als Node.js Native Addon bereitgestellt wird. Bei der Performance-Analyse haben wir signifikanten NAPI-Overhead identifiziert.

## Analyse

### NAPI String-Copying Overhead

NAPI muss Strings zwischen JavaScript (V8) und Rust kopieren. Dieser Overhead wurde gemessen:

```
String Größe | NAPI Overhead | Rust Processing
-------------|---------------|------------------
36 bytes     | ~0.002ms      | ~0.003ms
2.5 KB       | ~0.13ms       | ~0.09ms
28 KB        | ~1.5ms        | ~0.8ms
188 KB       | ~8ms          | ~2ms
```

**Wichtige Erkenntnis:** NAPI-Overhead skaliert **linear mit der Datengröße**, nicht pro Aufruf.

Gemessene Rate: **~0.054 µs/byte** für String-Kopieren

### Implikationen

Für ein 188KB Dokument:

- **NAPI Overhead**: ~8ms (Kopieren: JS → Rust, Rust → JS)
- **Rust Processing**: ~2ms
- **Gesamt**: ~10ms

Der NAPI-Overhead dominiert bei großen Dokumenten!

## Entscheidung

**Wir akzeptieren den NAPI-Overhead und optimieren den Rust-Code maximal.**

### Begründung

1. **Immer noch 3.5x schneller als JavaScript**
   - Selbst mit NAPI-Overhead sind wir deutlich schneller
   - JavaScript muss ebenfalls Strings verarbeiten

2. **Zero-Copy ist nicht möglich**
   - V8 Strings sind nicht direkt als Rust `&str` zugreifbar
   - NAPI-RS abstrahiert dies sauber

3. **Alternativen haben andere Trade-offs**
   - WebAssembly: Ähnlicher Kopier-Overhead
   - Worker Threads: Komplexere API, Serialization-Overhead

### Optimierungen die wir NICHT machen

1. **Chunk-basierte Verarbeitung**
   - Würde API komplizieren
   - Streaming-Support in lol_html funktioniert nicht für unseren Use Case

2. **Shared Memory / SharedArrayBuffer**
   - Komplexe Integration
   - Nicht thread-safe mit Rust's Ownership

3. **Caching auf Rust-Seite**
   - Würde Memory-Leaks riskieren
   - Lifetime-Management über FFI-Grenze problematisch

## Konsequenzen

### API Design

```typescript
// Einfache, synchrone API
const markdown = turndownService.turndown(html);

// NICHT: Streaming oder Chunks
// Das würde die API verkomplizieren ohne signifikanten Gewinn
```

### Performance-Charakteristik

| Dokument-Größe | NAPI % | Rust % | Empfehlung                             |
| -------------- | ------ | ------ | -------------------------------------- |
| < 1 KB         | ~20%   | ~80%   | Rust lohnt sich sehr                   |
| 1-10 KB        | ~40%   | ~60%   | Rust lohnt sich                        |
| 10-100 KB      | ~60%   | ~40%   | Rust lohnt sich noch                   |
| > 100 KB       | ~80%   | ~20%   | Grenzwertig, aber immer noch schneller |

### Batch-Processing

Für viele kleine Dokumente ist der Pro-Aufruf-Overhead gering:

```javascript
// Gut: Viele kleine Konvertierungen
htmlFiles.map((html) => service.turndown(html));

// Der Overhead ist pro Call minimal (~microseconds)
```

## Gemessene End-to-End Performance

| Fixture       | Größe   | Rust+NAPI | JavaScript | Speedup |
| ------------- | ------- | --------- | ---------- | ------- |
| simple        | 36 B    | 3.17µs    | 18.41µs    | 5.80x   |
| blog-post     | 2.5 KB  | 105.88µs  | 304.20µs   | 2.87x   |
| documentation | 4.3 KB  | 251.41µs  | 646.77µs   | 2.57x   |
| large-article | 28.6 KB | 1.27ms    | 3.29ms     | 2.59x   |
| large-100kb   | 188 KB  | 10.31ms   | 35.93ms    | 3.48x   |

**Fazit:** Trotz NAPI-Overhead bleiben wir durchschnittlich **3.46x schneller**.

## Lessons Learned

1. **FFI-Overhead ist real und messbar**
   - Bei Performance-kritischen Anwendungen immer messen
   - Nicht annehmen dass "Native = schneller"

2. **Lineare Skalierung beachten**
   - Overhead skaliert mit Daten, nicht mit Aufrufen
   - Wichtig für Kapazitätsplanung

3. **Einfache API > marginale Performance**
   - Komplexe Streaming-APIs bringen nur ~20% Verbesserung
   - Developer Experience ist wichtiger

## Referenzen

- [NAPI-RS](https://napi.rs/)
- [Node.js N-API](https://nodejs.org/api/n-api.html)
- [V8 String Internals](https://v8.dev/blog/string)
