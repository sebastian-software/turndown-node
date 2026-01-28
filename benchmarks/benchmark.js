#!/usr/bin/env node

/**
 * Benchmark: turndown-node (Rust) vs turndown (JavaScript)
 */

const TurndownNode = require("turndown-node");
const TurndownJS = require("turndown");

// Test HTML samples of varying complexity
const samples = {
  simple: "<p>Hello <strong>World</strong></p>",

  medium: `
    <article>
      <h1>Article Title</h1>
      <p>This is a <strong>paragraph</strong> with <em>formatting</em>.</p>
      <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
      </ul>
      <blockquote>
        <p>A quote with <a href="https://example.com">a link</a>.</p>
      </blockquote>
    </article>
  `,

  complex: `
    <html>
      <body>
        <article>
          <header>
            <h1>Comprehensive HTML Document</h1>
            <p class="meta">Published on <time>2026-01-28</time></p>
          </header>

          <section>
            <h2>Introduction</h2>
            <p>This document contains <strong>bold text</strong>, <em>italic text</em>,
               and <code>inline code</code>. Here's a <a href="https://example.com" title="Example">link with title</a>.</p>

            <h3>Code Example</h3>
            <pre><code class="language-javascript">function hello() {
  console.log("Hello, World!");
  return 42;
}</code></pre>
          </section>

          <section>
            <h2>Lists</h2>
            <h3>Unordered List</h3>
            <ul>
              <li>First item with <strong>bold</strong></li>
              <li>Second item with <em>emphasis</em></li>
              <li>Third item with <code>code</code></li>
              <li>Nested list:
                <ul>
                  <li>Nested item 1</li>
                  <li>Nested item 2</li>
                </ul>
              </li>
            </ul>

            <h3>Ordered List</h3>
            <ol>
              <li>Step one</li>
              <li>Step two</li>
              <li>Step three</li>
            </ol>
          </section>

          <section>
            <h2>Other Elements</h2>
            <blockquote>
              <p>This is a blockquote with multiple paragraphs.</p>
              <p>Second paragraph in the quote.</p>
            </blockquote>

            <hr>

            <p>An image: <img src="photo.jpg" alt="A photo" title="Photo title"></p>

            <table>
              <thead>
                <tr><th>Header 1</th><th>Header 2</th></tr>
              </thead>
              <tbody>
                <tr><td>Cell 1</td><td>Cell 2</td></tr>
                <tr><td>Cell 3</td><td>Cell 4</td></tr>
              </tbody>
            </table>
          </section>
        </article>
      </body>
    </html>
  `,
};

// Generate a large document by repeating the complex sample
samples.large = Array(50).fill(samples.complex).join("\n");

function formatNumber(num) {
  return num.toLocaleString("en-US", { maximumFractionDigits: 0 });
}

function formatDuration(ms) {
  if (ms < 1) return `${(ms * 1000).toFixed(2)}µs`;
  if (ms < 1000) return `${ms.toFixed(2)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

function benchmark(name, fn, iterations = 1000) {
  // Warmup
  for (let i = 0; i < 10; i++) fn();

  // Measure
  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    fn();
  }
  const end = performance.now();

  const totalMs = end - start;
  const avgMs = totalMs / iterations;
  const opsPerSec = 1000 / avgMs;

  return { name, totalMs, avgMs, opsPerSec, iterations };
}

function runBenchmarks() {
  console.log("# turndown-node Benchmark Results\n");
  console.log(`Date: ${new Date().toISOString()}`);
  console.log(`Node.js: ${process.version}`);
  console.log(`Platform: ${process.platform}-${process.arch}\n`);

  const nodeService = new TurndownNode();
  const jsService = new TurndownJS();

  const results = [];

  for (const [sampleName, html] of Object.entries(samples)) {
    const htmlSize = Buffer.byteLength(html, "utf8");
    const iterations = sampleName === "large" ? 100 : 1000;

    console.log(`## ${sampleName} (${formatNumber(htmlSize)} bytes)\n`);

    const nodeResult = benchmark(
      "turndown-node",
      () => nodeService.turndown(html),
      iterations
    );

    const jsResult = benchmark(
      "turndown",
      () => jsService.turndown(html),
      iterations
    );

    const speedup = jsResult.avgMs / nodeResult.avgMs;

    console.log(`| Library | Ops/sec | Avg Time | Iterations |`);
    console.log(`|---------|---------|----------|------------|`);
    console.log(
      `| turndown-node (Rust) | ${formatNumber(nodeResult.opsPerSec)} | ${formatDuration(nodeResult.avgMs)} | ${nodeResult.iterations} |`
    );
    console.log(
      `| turndown (JS) | ${formatNumber(jsResult.opsPerSec)} | ${formatDuration(jsResult.avgMs)} | ${jsResult.iterations} |`
    );
    console.log(`\n**Speedup: ${speedup.toFixed(2)}x faster**\n`);

    results.push({
      sample: sampleName,
      htmlSize,
      nodeOpsPerSec: nodeResult.opsPerSec,
      jsOpsPerSec: jsResult.opsPerSec,
      speedup,
    });
  }

  // Summary
  console.log("## Summary\n");
  const avgSpeedup =
    results.reduce((sum, r) => sum + r.speedup, 0) / results.length;
  console.log(
    `Average speedup across all tests: **${avgSpeedup.toFixed(2)}x faster**\n`
  );

  console.log("| Sample | Size | Speedup |");
  console.log("|--------|------|---------|");
  for (const r of results) {
    console.log(
      `| ${r.sample} | ${formatNumber(r.htmlSize)} bytes | ${r.speedup.toFixed(2)}x |`
    );
  }
}

runBenchmarks();
