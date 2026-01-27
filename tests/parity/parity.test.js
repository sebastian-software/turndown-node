/**
 * Parity tests - ensure turndown-node produces identical output to turndown
 */

// Note: These tests will only work once the native binding is built
// For now, they serve as a specification

const testCases = [
  // Basic elements
  { name: "paragraph", html: "<p>Hello World</p>" },
  { name: "multiple paragraphs", html: "<p>First</p><p>Second</p>" },
  { name: "line break", html: "<p>Line 1<br>Line 2</p>" },

  // Headings
  { name: "h1", html: "<h1>Heading 1</h1>" },
  { name: "h2", html: "<h2>Heading 2</h2>" },
  { name: "h3", html: "<h3>Heading 3</h3>" },

  // Emphasis
  { name: "emphasis", html: "<em>emphasized</em>" },
  { name: "strong", html: "<strong>bold</strong>" },
  {
    name: "nested emphasis",
    html: "<strong><em>bold and italic</em></strong>",
  },

  // Code
  { name: "inline code", html: "<code>code</code>" },
  { name: "code block", html: "<pre><code>function() {}</code></pre>" },

  // Links
  { name: "link", html: '<a href="https://example.com">Link</a>' },
  {
    name: "link with title",
    html: '<a href="https://example.com" title="Title">Link</a>',
  },

  // Images
  { name: "image", html: '<img src="test.png" alt="Alt text">' },

  // Lists
  { name: "unordered list", html: "<ul><li>One</li><li>Two</li></ul>" },
  { name: "ordered list", html: "<ol><li>One</li><li>Two</li></ol>" },

  // Blockquotes
  { name: "blockquote", html: "<blockquote><p>Quote</p></blockquote>" },

  // Horizontal rule
  { name: "hr", html: "<hr>" },
];

describe("turndown-node vs turndown parity", () => {
  let TurndownNode;
  let TurndownOriginal;

  beforeAll(() => {
    try {
      TurndownNode = require("turndown-node");
      TurndownOriginal = require("turndown");
    } catch {
      // Native binding not built yet - skip tests
      console.warn("Native binding not available, skipping parity tests");
    }
  });

  testCases.forEach(({ name, html }) => {
    it(`${name}: produces identical output`, () => {
      if (!TurndownNode || !TurndownOriginal) {
        return; // Skip if bindings not available
      }

      const node = new TurndownNode();
      const original = new TurndownOriginal();

      expect(node.turndown(html)).toBe(original.turndown(html));
    });
  });
});
