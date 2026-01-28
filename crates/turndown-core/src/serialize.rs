//! Markdown AST serialization
//!
//! Converts Markdown AST nodes into Markdown text.

use crate::ast::{inlines_text_len, Block, Inline, ListItem};
use crate::options::{CodeBlockStyle, HeadingStyle, LinkStyle, Options};

/// Serialize a block to Markdown string
pub fn serialize(block: &Block, options: &Options) -> String {
    let mut output = serialize_block(block, options, 0);

    // Post-process: collapse multiple newlines
    output = collapse_newlines(&output);

    // Trim leading/trailing newlines
    output.trim_matches('\n').to_string()
}

fn serialize_block(block: &Block, options: &Options, depth: usize) -> String {
    match block {
        Block::Document(blocks) => serialize_blocks(blocks, options, depth),

        Block::Heading { level, content } => serialize_heading(*level, content, options),

        Block::Paragraph(inlines) => {
            let text = serialize_inlines(inlines, options);
            if text.trim().is_empty() {
                String::new()
            } else {
                format!("{}\n\n", text)
            }
        }

        Block::BlockQuote(blocks) => {
            let content = serialize_blocks(blocks, options, depth);
            let lines: Vec<&str> = content.trim_end().lines().collect();
            let quoted = lines
                .iter()
                .map(|line| {
                    if line.is_empty() {
                        ">".to_string()
                    } else {
                        format!("> {}", line)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}\n\n", quoted)
        }

        Block::List {
            ordered,
            start,
            items,
        } => serialize_list(*ordered, *start, items, options, depth),

        Block::CodeBlock {
            language,
            code,
            fenced,
        } => serialize_code_block(language.as_deref(), code, *fenced, options),

        Block::ThematicBreak => format!("{}\n\n", options.hr),

        Block::Table { headers, rows } => serialize_table(headers, rows, options),

        Block::HtmlBlock(html) => format!("{}\n\n", html),
    }
}

fn serialize_blocks(blocks: &[Block], options: &Options, depth: usize) -> String {
    let mut result = String::new();
    for block in blocks {
        if !block.is_blank() {
            result.push_str(&serialize_block(block, options, depth));
        }
    }
    result
}

fn serialize_heading(level: u8, content: &[Inline], options: &Options) -> String {
    let text = serialize_inlines(content, options);

    if text.trim().is_empty() {
        return String::new();
    }

    match options.heading_style {
        HeadingStyle::Setext if level <= 2 => {
            let underline = if level == 1 { '=' } else { '-' };
            let underline_str: String = std::iter::repeat(underline).take(text.len()).collect();
            format!("{}\n{}\n\n", text, underline_str)
        }
        _ => {
            let hashes: String = std::iter::repeat('#').take(level as usize).collect();
            format!("{} {}\n\n", hashes, text)
        }
    }
}

fn serialize_list(
    ordered: bool,
    start: u32,
    items: &[ListItem],
    options: &Options,
    depth: usize,
) -> String {
    let mut result = String::new();
    let indent = "    ".repeat(depth);

    for (i, item) in items.iter().enumerate() {
        let prefix = if ordered {
            format!("{}.  ", start + i as u32)
        } else {
            format!("{}   ", options.bullet_list_marker)
        };

        let content = serialize_list_item(item, options, depth + 1);
        let content = content.trim();

        // Indent continuation lines
        let mut lines: Vec<&str> = content.lines().collect();
        if !lines.is_empty() {
            let first = lines.remove(0);
            result.push_str(&indent);
            result.push_str(&prefix);
            result.push_str(first);
            result.push('\n');

            for line in lines {
                result.push_str(&indent);
                result.push_str(&" ".repeat(prefix.len()));
                result.push_str(line);
                result.push('\n');
            }
        }
    }

    result.push('\n');
    result
}

fn serialize_list_item(item: &ListItem, options: &Options, depth: usize) -> String {
    let mut result = String::new();

    for (i, block) in item.content.iter().enumerate() {
        match block {
            Block::Paragraph(inlines) => {
                let text = serialize_inlines(inlines, options);
                result.push_str(&text);
                if i < item.content.len() - 1 {
                    result.push_str("\n\n");
                }
            }
            Block::List { .. } => {
                result.push('\n');
                result.push_str(&serialize_block(block, options, depth));
            }
            _ => {
                result.push_str(&serialize_block(block, options, depth));
            }
        }
    }

    result
}

fn serialize_code_block(
    language: Option<&str>,
    code: &str,
    fenced: bool,
    options: &Options,
) -> String {
    let use_fenced = fenced || options.code_block_style == CodeBlockStyle::Fenced;

    if use_fenced {
        let lang = language.unwrap_or("");
        format!("{}{}\n{}\n{}\n\n", options.fence, lang, code, options.fence)
    } else {
        // Indented code block
        let indented: Vec<String> = code.lines().map(|line| format!("    {}", line)).collect();
        format!("{}\n\n", indented.join("\n"))
    }
}

fn serialize_table(
    headers: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    options: &Options,
) -> String {
    if headers.is_empty() {
        return String::new();
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

    let mut result = String::new();

    // Header row
    result.push('|');
    for (i, header) in headers.iter().enumerate() {
        let text = serialize_inlines(header, options);
        let padding = widths.get(i).copied().unwrap_or(3) - text.len();
        result.push(' ');
        result.push_str(&text);
        result.push_str(&" ".repeat(padding));
        result.push_str(" |");
    }
    result.push('\n');

    // Separator row
    result.push('|');
    for &width in &widths[..col_count] {
        result.push(' ');
        result.push_str(&"-".repeat(width));
        result.push_str(" |");
    }
    result.push('\n');

    // Data rows
    for row in rows {
        result.push('|');
        for (i, cell) in row.iter().enumerate() {
            let text = serialize_inlines(cell, options);
            let width = widths.get(i).copied().unwrap_or(3);
            let padding = width.saturating_sub(text.len());
            result.push(' ');
            result.push_str(&text);
            result.push_str(&" ".repeat(padding));
            result.push_str(" |");
        }
        result.push('\n');
    }

    result.push('\n');
    result
}

fn serialize_inlines(inlines: &[Inline], options: &Options) -> String {
    let mut result = String::new();
    for inline in inlines {
        result.push_str(&serialize_inline(inline, options));
    }
    result
}

fn serialize_inline(inline: &Inline, options: &Options) -> String {
    match inline {
        Inline::Text(text) => text.clone(),

        Inline::Strong(content) => {
            let inner = serialize_inlines(content, options);
            if inner.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}{}", options.strong_delimiter, inner, options.strong_delimiter)
            }
        }

        Inline::Emphasis(content) => {
            let inner = serialize_inlines(content, options);
            if inner.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}{}", options.em_delimiter, inner, options.em_delimiter)
            }
        }

        Inline::Code(code) => {
            if code.is_empty() {
                String::new()
            } else {
                // Handle backticks in code
                let backticks = if code.contains('`') { "``" } else { "`" };
                let space = if code.starts_with('`') || code.ends_with('`') {
                    " "
                } else {
                    ""
                };
                format!("{}{}{}{}{}", backticks, space, code, space, backticks)
            }
        }

        Inline::Link {
            content,
            url,
            title,
        } => {
            let text = serialize_inlines(content, options);
            match options.link_style {
                LinkStyle::Inlined => {
                    if let Some(t) = title {
                        format!("[{}]({} \"{}\")", text, url, t)
                    } else {
                        format!("[{}]({})", text, url)
                    }
                }
                LinkStyle::Referenced => {
                    // For now, just use inline style
                    // TODO: Collect references and output at end
                    format!("[{}]({})", text, url)
                }
            }
        }

        Inline::Image { alt, url, title } => {
            if let Some(t) = title {
                format!("![{}]({} \"{}\")", alt, url, t)
            } else {
                format!("![{}]({})", alt, url)
            }
        }

        Inline::LineBreak => "  \n".to_string(),

        Inline::HtmlInline(html) => html.clone(),
    }
}

/// Collapse multiple consecutive newlines into at most two
fn collapse_newlines(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut newline_count = 0;

    for c in input.chars() {
        if c == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(c);
            }
        } else {
            newline_count = 0;
            result.push(c);
        }
    }

    result
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
