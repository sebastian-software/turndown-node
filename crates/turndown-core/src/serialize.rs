//! Markdown AST serialization
//!
//! Converts Markdown AST nodes into Markdown text.

use crate::ast::{inlines_text_len, Block, Inline, ListItem};
use crate::options::{CodeBlockStyle, HeadingStyle, Options};

/// Serialize a block to Markdown string
pub fn serialize(block: &Block, options: &Options) -> String {
    // Estimate capacity: ~2x input for markdown overhead
    let mut output = String::with_capacity(4096);
    serialize_block(block, options, 0, &mut output);

    // Post-process: collapse multiple newlines and trim
    collapse_and_trim(&mut output);
    output
}

fn serialize_block(block: &Block, options: &Options, depth: usize, out: &mut String) {
    match block {
        Block::Document(blocks) => serialize_blocks(blocks, options, depth, out),

        Block::Heading { level, content } => serialize_heading(*level, content, options, out),

        Block::Paragraph(inlines) => {
            let start_len = out.len();
            serialize_inlines(inlines, options, out);
            if out[start_len..].trim().is_empty() {
                out.truncate(start_len);
            } else {
                out.push_str("\n\n");
            }
        }

        Block::BlockQuote(blocks) => {
            let start_len = out.len();
            serialize_blocks(blocks, options, depth, out);

            // Process the content we just wrote to add > prefixes
            let content = out[start_len..].trim_end().to_string();
            out.truncate(start_len);

            for (i, line) in content.lines().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                out.push('>');
                if !line.is_empty() {
                    out.push(' ');
                    out.push_str(line);
                }
            }
            out.push_str("\n\n");
        }

        Block::List {
            ordered,
            start,
            items,
        } => serialize_list(*ordered, *start, items, options, depth, out),

        Block::CodeBlock {
            language,
            code,
            fenced,
        } => serialize_code_block(language.as_deref(), code, *fenced, options, out),

        Block::ThematicBreak => {
            out.push_str(&options.hr);
            out.push_str("\n\n");
        }

        Block::Table { headers, rows } => serialize_table(headers, rows, options, out),

        Block::HtmlBlock(html) => {
            out.push_str(html);
            out.push_str("\n\n");
        }
    }
}

fn serialize_blocks(blocks: &[Block], options: &Options, depth: usize, out: &mut String) {
    for block in blocks {
        if !block.is_blank() {
            serialize_block(block, options, depth, out);
        }
    }
}

fn serialize_heading(level: u8, content: &[Inline], options: &Options, out: &mut String) {
    let start_len = out.len();
    serialize_inlines(content, options, out);

    if out[start_len..].trim().is_empty() {
        out.truncate(start_len);
        return;
    }

    let text_len = out.len() - start_len;

    match options.heading_style {
        HeadingStyle::Setext if level <= 2 => {
            out.push('\n');
            let underline = if level == 1 { '=' } else { '-' };
            for _ in 0..text_len {
                out.push(underline);
            }
            out.push_str("\n\n");
        }
        _ => {
            // Need to prepend hashes - shift content
            let text = out[start_len..].to_string();
            out.truncate(start_len);
            for _ in 0..level {
                out.push('#');
            }
            out.push(' ');
            out.push_str(&text);
            out.push_str("\n\n");
        }
    }
}

fn serialize_list(
    ordered: bool,
    start: u32,
    items: &[ListItem],
    options: &Options,
    depth: usize,
    out: &mut String,
) {
    let indent = "    ".repeat(depth);

    for (i, item) in items.iter().enumerate() {
        out.push_str(&indent);

        if ordered {
            // Write number prefix
            let num = start + i as u32;
            out.push_str(&num.to_string());
            out.push_str(".  ");
        } else {
            out.push(options.bullet_list_marker);
            out.push_str("   ");
        }

        let prefix_len = if ordered {
            (start + i as u32).to_string().len() + 3
        } else {
            4
        };

        serialize_list_item(item, options, depth + 1, prefix_len, &indent, out);
    }

    out.push('\n');
}

fn serialize_list_item(
    item: &ListItem,
    options: &Options,
    depth: usize,
    prefix_len: usize,
    indent: &str,
    out: &mut String,
) {
    let start_len = out.len();

    for (i, block) in item.content.iter().enumerate() {
        match block {
            Block::Paragraph(inlines) => {
                serialize_inlines(inlines, options, out);
                if i < item.content.len() - 1 {
                    out.push_str("\n\n");
                }
            }
            Block::List { .. } => {
                out.push('\n');
                serialize_block(block, options, depth, out);
            }
            _ => {
                serialize_block(block, options, depth, out);
            }
        }
    }

    // Indent continuation lines
    let content = out[start_len..].to_string();
    out.truncate(start_len);

    let continuation_indent: String = std::iter::repeat(' ').take(prefix_len).collect();

    for (i, line) in content.lines().enumerate() {
        if i > 0 {
            out.push_str(indent);
            out.push_str(&continuation_indent);
        }
        out.push_str(line);
        out.push('\n');
    }
}

fn serialize_code_block(
    language: Option<&str>,
    code: &str,
    fenced: bool,
    options: &Options,
    out: &mut String,
) {
    let use_fenced = fenced || options.code_block_style == CodeBlockStyle::Fenced;

    if use_fenced {
        out.push_str(&options.fence);
        out.push_str(language.unwrap_or(""));
        out.push('\n');
        out.push_str(code);
        out.push('\n');
        out.push_str(&options.fence);
        out.push_str("\n\n");
    } else {
        for line in code.lines() {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
    }
}

fn serialize_table(
    headers: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    options: &Options,
    out: &mut String,
) {
    if headers.is_empty() {
        return;
    }

    // Calculate column widths
    let col_count = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| inlines_text_len(h)).collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(inlines_text_len(cell));
            }
        }
    }

    // Minimum width of 3 for separator
    for w in &mut widths {
        *w = (*w).max(3);
    }

    // Header row
    out.push('|');
    for (i, header) in headers.iter().enumerate() {
        let start = out.len();
        out.push(' ');
        serialize_inlines(header, options, out);
        let text_len = out.len() - start - 1;
        let padding = widths.get(i).copied().unwrap_or(3).saturating_sub(text_len);
        for _ in 0..padding {
            out.push(' ');
        }
        out.push_str(" |");
    }
    out.push('\n');

    // Separator row
    out.push('|');
    for &width in &widths[..col_count] {
        out.push(' ');
        for _ in 0..width {
            out.push('-');
        }
        out.push_str(" |");
    }
    out.push('\n');

    // Data rows
    for row in rows {
        out.push('|');
        for (i, cell) in row.iter().enumerate() {
            let start = out.len();
            out.push(' ');
            serialize_inlines(cell, options, out);
            let text_len = out.len() - start - 1;
            let width = widths.get(i).copied().unwrap_or(3);
            let padding = width.saturating_sub(text_len);
            for _ in 0..padding {
                out.push(' ');
            }
            out.push_str(" |");
        }
        out.push('\n');
    }

    out.push('\n');
}

fn serialize_inlines(inlines: &[Inline], options: &Options, out: &mut String) {
    for inline in inlines {
        serialize_inline(inline, options, out);
    }
}

fn serialize_inline(inline: &Inline, options: &Options, out: &mut String) {
    match inline {
        Inline::Text(text) => out.push_str(text),

        Inline::Strong(content) => {
            let start = out.len();
            serialize_inlines(content, options, out);
            if out[start..].trim().is_empty() {
                out.truncate(start);
            } else {
                let inner = out[start..].to_string();
                out.truncate(start);
                out.push_str(&options.strong_delimiter);
                out.push_str(&inner);
                out.push_str(&options.strong_delimiter);
            }
        }

        Inline::Emphasis(content) => {
            let start = out.len();
            serialize_inlines(content, options, out);
            if out[start..].trim().is_empty() {
                out.truncate(start);
            } else {
                let inner = out[start..].to_string();
                out.truncate(start);
                out.push(options.em_delimiter);
                out.push_str(&inner);
                out.push(options.em_delimiter);
            }
        }

        Inline::Code(code) => {
            if !code.is_empty() {
                let backticks = if code.contains('`') { "``" } else { "`" };
                let space = if code.starts_with('`') || code.ends_with('`') {
                    " "
                } else {
                    ""
                };
                out.push_str(backticks);
                out.push_str(space);
                out.push_str(code);
                out.push_str(space);
                out.push_str(backticks);
            }
        }

        Inline::Link {
            content,
            url,
            title,
        } => {
            out.push('[');
            serialize_inlines(content, options, out);
            out.push_str("](");
            out.push_str(url);
            if let Some(t) = title {
                out.push_str(" \"");
                out.push_str(t);
                out.push('"');
            }
            out.push(')');
        }

        Inline::Image { alt, url, title } => {
            out.push_str("![");
            out.push_str(alt);
            out.push_str("](");
            out.push_str(url);
            if let Some(t) = title {
                out.push_str(" \"");
                out.push_str(t);
                out.push('"');
            }
            out.push(')');
        }

        Inline::LineBreak => out.push_str("  \n"),

        Inline::HtmlInline(html) => out.push_str(html),
    }
}

/// Collapse multiple consecutive newlines into at most two, in place
fn collapse_and_trim(s: &mut String) {
    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut newline_count = 0;
    let mut start = 0;
    let mut end = bytes.len();

    // Find start (skip leading newlines)
    while start < bytes.len() && bytes[start] == b'\n' {
        start += 1;
    }

    // Find end (skip trailing newlines)
    while end > start && bytes[end - 1] == b'\n' {
        end -= 1;
    }

    for &b in &bytes[start..end] {
        if b == b'\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(b);
            }
        } else {
            newline_count = 0;
            result.push(b);
        }
    }

    *s = String::from_utf8(result).unwrap_or_default();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_options() -> Options {
        Options::default()
    }

    #[test]
    fn test_paragraph() {
        let block = Block::Paragraph(vec![Inline::Text("Hello World".to_string())]);
        let result = serialize(&block, &default_options());
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_heading_setext_h1() {
        let block = Block::Heading {
            level: 1,
            content: vec![Inline::Text("Title".to_string())],
        };
        let result = serialize(&block, &default_options());
        assert_eq!(result, "Title\n=====");
    }

    #[test]
    fn test_heading_setext_h2() {
        let block = Block::Heading {
            level: 2,
            content: vec![Inline::Text("Subtitle".to_string())],
        };
        let result = serialize(&block, &default_options());
        assert_eq!(result, "Subtitle\n--------");
    }

    #[test]
    fn test_heading_atx() {
        let mut options = default_options();
        options.heading_style = HeadingStyle::Atx;

        let block = Block::Heading {
            level: 3,
            content: vec![Inline::Text("Section".to_string())],
        };
        let result = serialize(&block, &options);
        assert_eq!(result, "### Section");
    }

    #[test]
    fn test_strong() {
        let block = Block::Paragraph(vec![Inline::Strong(vec![Inline::Text(
            "bold".to_string(),
        )])]);
        let result = serialize(&block, &default_options());
        assert_eq!(result, "**bold**");
    }

    #[test]
    fn test_emphasis() {
        let block = Block::Paragraph(vec![Inline::Emphasis(vec![Inline::Text(
            "italic".to_string(),
        )])]);
        let result = serialize(&block, &default_options());
        assert_eq!(result, "_italic_");
    }

    #[test]
    fn test_inline_code() {
        let block = Block::Paragraph(vec![Inline::Code("code".to_string())]);
        let result = serialize(&block, &default_options());
        assert_eq!(result, "`code`");
    }

    #[test]
    fn test_link() {
        let block = Block::Paragraph(vec![Inline::Link {
            content: vec![Inline::Text("Example".to_string())],
            url: "https://example.com".to_string(),
            title: None,
        }]);
        let result = serialize(&block, &default_options());
        assert_eq!(result, "[Example](https://example.com)");
    }

    #[test]
    fn test_image() {
        let block = Block::Paragraph(vec![Inline::Image {
            alt: "Alt text".to_string(),
            url: "image.png".to_string(),
            title: None,
        }]);
        let result = serialize(&block, &default_options());
        assert_eq!(result, "![Alt text](image.png)");
    }

    #[test]
    fn test_code_block_indented() {
        let block = Block::CodeBlock {
            language: None,
            code: "let x = 1;".to_string(),
            fenced: false,
        };
        let result = serialize(&block, &default_options());
        assert_eq!(result, "    let x = 1;");
    }

    #[test]
    fn test_code_block_fenced() {
        let mut options = default_options();
        options.code_block_style = CodeBlockStyle::Fenced;

        let block = Block::CodeBlock {
            language: Some("rust".to_string()),
            code: "let x = 1;".to_string(),
            fenced: true,
        };
        let result = serialize(&block, &options);
        assert_eq!(result, "```rust\nlet x = 1;\n```");
    }

    #[test]
    fn test_blockquote() {
        let block = Block::BlockQuote(vec![Block::Paragraph(vec![Inline::Text(
            "Quote".to_string(),
        )])]);
        let result = serialize(&block, &default_options());
        assert_eq!(result, "> Quote");
    }

    #[test]
    fn test_unordered_list() {
        let block = Block::List {
            ordered: false,
            start: 1,
            items: vec![
                ListItem::from_inlines(vec![Inline::Text("One".to_string())]),
                ListItem::from_inlines(vec![Inline::Text("Two".to_string())]),
            ],
        };
        let result = serialize(&block, &default_options());
        assert_eq!(result, "*   One\n*   Two");
    }

    #[test]
    fn test_ordered_list() {
        let block = Block::List {
            ordered: true,
            start: 1,
            items: vec![
                ListItem::from_inlines(vec![Inline::Text("First".to_string())]),
                ListItem::from_inlines(vec![Inline::Text("Second".to_string())]),
            ],
        };
        let result = serialize(&block, &default_options());
        assert_eq!(result, "1.  First\n2.  Second");
    }

    #[test]
    fn test_thematic_break() {
        let block = Block::ThematicBreak;
        let result = serialize(&block, &default_options());
        assert_eq!(result, "* * *");
    }

    #[test]
    fn test_table() {
        let block = Block::Table {
            headers: vec![
                vec![Inline::Text("A".to_string())],
                vec![Inline::Text("B".to_string())],
            ],
            rows: vec![vec![
                vec![Inline::Text("1".to_string())],
                vec![Inline::Text("2".to_string())],
            ]],
        };
        let result = serialize(&block, &default_options());
        assert!(result.contains("| A"));
        assert!(result.contains("| B"));
        assert!(result.contains("---"));
    }
}
