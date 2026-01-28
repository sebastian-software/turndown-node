import { writeFileSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixturesDir = join(__dirname, "fixtures");

// Generate a section of realistic HTML content
function generateSection(n) {
  return `<article>
<h2>Section ${n}: Important Topic</h2>
<p>This is a paragraph with <strong>bold text</strong> and <em>italic text</em> and some <code>inline code</code> for good measure. Here's a <a href="https://example.com/page/${n}">link to page ${n}</a> that demonstrates how links work.</p>
<blockquote>
<p>This is a meaningful quote that adds context to the discussion. It spans multiple sentences and contains <strong>emphasized</strong> content.</p>
</blockquote>
<ul>
<li>First list item with some content</li>
<li>Second item with <a href="https://test.com">a link</a></li>
<li>Third item with <code>code</code> inside</li>
</ul>
<p>Another paragraph follows with more text. This helps simulate real-world article content that would be processed by a tool like Readability before conversion to Markdown.</p>
<pre><code class="language-javascript">function example${n}() {
    const value = ${n};
    return value * 2;
}</code></pre>
<ol>
<li>Numbered item one</li>
<li>Numbered item two</li>
<li>Numbered item three</li>
</ol>
<table>
<thead><tr><th>Column A</th><th>Column B</th><th>Column C</th></tr></thead>
<tbody>
<tr><td>Data ${n}-1</td><td>Value</td><td>Result</td></tr>
<tr><td>Data ${n}-2</td><td>Value</td><td>Result</td></tr>
</tbody>
</table>
</article>
`;
}

function generateHtml(targetBytes) {
  const header = `<!DOCTYPE html>
<html>
<head><title>Benchmark Document</title></head>
<body>
`;
  const footer = `</body>
</html>
`;

  let content = header;
  let sectionNum = 1;

  while (Buffer.byteLength(content + footer, "utf8") < targetBytes) {
    content += generateSection(sectionNum++);
  }

  content += footer;
  return content;
}

// Target sizes
const fixtures = [
  { name: "small", targetKB: 1 },
  { name: "medium", targetKB: 10 },
  { name: "large", targetKB: 100 },
  { name: "huge", targetKB: 1000 },
];

for (const { name, targetKB } of fixtures) {
  const html = generateHtml(targetKB * 1024);
  const path = join(fixturesDir, `${name}.html`);
  writeFileSync(path, html);
  const actualBytes = Buffer.byteLength(html, "utf8");
  console.log(
    `${name}.html: ${(actualBytes / 1024).toFixed(1)}KB (${actualBytes} bytes)`
  );
}
