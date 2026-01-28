import { Bench } from "tinybench";
import { createRequire } from "module";
import { readFileSync, readdirSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const require = createRequire(import.meta.url);
const TurndownNode = require("turndown-node");
const TurndownJS = require("turndown");

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixturesDir = join(__dirname, "fixtures");

// Load all fixture files
const fixtures = readdirSync(fixturesDir)
  .filter((f) => f.endsWith(".html"))
  .map((f) => ({
    name: f.replace(".html", ""),
    html: readFileSync(join(fixturesDir, f), "utf8"),
  }))
  .sort((a, b) => a.html.length - b.html.length);

console.log("# turndown-node Benchmark Results\n");
console.log(`Date: ${new Date().toISOString()}`);
console.log(`Node.js: ${process.version}`);
console.log(`Platform: ${process.platform}-${process.arch}\n`);

const nodeService = new TurndownNode();
const jsService = new TurndownJS();

const results = [];

for (const fixture of fixtures) {
  const size = Buffer.byteLength(fixture.html, "utf8");
  console.log(`## ${fixture.name} (${size.toLocaleString()} bytes)\n`);

  const bench = new Bench({ time: 1000 });

  bench
    .add("turndown-node (Rust)", () => {
      nodeService.turndown(fixture.html);
    })
    .add("turndown (JS)", () => {
      jsService.turndown(fixture.html);
    });

  await bench.run();

  const nodeResult = bench.tasks.find(
    (t) => t.name === "turndown-node (Rust)"
  ).result;
  const jsResult = bench.tasks.find((t) => t.name === "turndown (JS)").result;

  const speedup = jsResult.mean / nodeResult.mean;

  console.log("| Library | Ops/sec | Mean Time | Margin |");
  console.log("|---------|---------|-----------|--------|");
  console.log(
    `| turndown-node (Rust) | ${nodeResult.hz.toLocaleString("en-US", { maximumFractionDigits: 0 })} | ${formatTime(nodeResult.mean)} | ±${nodeResult.rme.toFixed(2)}% |`
  );
  console.log(
    `| turndown (JS) | ${jsResult.hz.toLocaleString("en-US", { maximumFractionDigits: 0 })} | ${formatTime(jsResult.mean)} | ±${jsResult.rme.toFixed(2)}% |`
  );
  console.log(`\n**Speedup: ${speedup.toFixed(2)}x faster**\n`);
  console.log("---\n");

  results.push({
    name: fixture.name,
    size,
    nodeHz: nodeResult.hz,
    jsHz: jsResult.hz,
    speedup,
  });
}

// Summary
console.log("## Summary\n");
const avgSpeedup =
  results.reduce((sum, r) => sum + r.speedup, 0) / results.length;
console.log(
  `Average speedup across all fixtures: **${avgSpeedup.toFixed(2)}x faster**\n`
);

console.log("| Fixture | Size | Rust Ops/s | JS Ops/s | Speedup |");
console.log("|---------|------|------------|----------|---------|");
for (const r of results) {
  console.log(
    `| ${r.name} | ${r.size.toLocaleString()} bytes | ${r.nodeHz.toLocaleString("en-US", { maximumFractionDigits: 0 })} | ${r.jsHz.toLocaleString("en-US", { maximumFractionDigits: 0 })} | **${r.speedup.toFixed(2)}x** |`
  );
}

function formatTime(ns) {
  if (ns < 0.001) return `${(ns * 1_000_000).toFixed(2)}ns`;
  if (ns < 1) return `${(ns * 1000).toFixed(2)}µs`;
  if (ns < 1000) return `${ns.toFixed(2)}ms`;
  return `${(ns / 1000).toFixed(2)}s`;
}
