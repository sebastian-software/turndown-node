//! CommonMark rules for HTML to Markdown conversion.

use super::{Filter, Rule};
use crate::service::{CodeBlockStyle, HeadingStyle, LinkStyle};
use crate::utilities::{clean_attribute, repeat};

/// Create all CommonMark rules
pub fn commonmark_rules() -> Vec<Rule> {
    vec![
        paragraph_rule(),
        line_break_rule(),
        heading_rule(),
        blockquote_rule(),
        list_rule(),
        list_item_rule(),
        indented_code_block_rule(),
        fenced_code_block_rule(),
        horizontal_rule(),
        inline_link_rule(),
        reference_link_rule(),
        emphasis_rule(),
        strong_rule(),
        code_rule(),
        image_rule(),
    ]
}

fn paragraph_rule() -> Rule {
    Rule::for_tag("p", |_, content, _| {
        format!("\n\n{}\n\n", content.trim())
    })
}

fn line_break_rule() -> Rule {
    Rule::for_tag("br", |_, _, _| "  \n".to_string())
}

fn heading_rule() -> Rule {
    Rule::new(
        Filter::tags(&["h1", "h2", "h3", "h4", "h5", "h6"]),
        |node, content, options| {
            let tag = node.tag_name();
            let level: usize = tag[1..].parse().unwrap_or(1);

            let content = content.trim();
            if content.is_empty() {
                return String::new();
            }

            match options.heading_style {
                HeadingStyle::Setext if level <= 2 => {
                    let underline = if level == 1 { "=" } else { "-" };
                    format!(
                        "\n\n{}\n{}\n\n",
                        content,
                        repeat(underline, content.len())
                    )
                }
                _ => {
                    format!("\n\n{} {}\n\n", repeat("#", level), content)
                }
            }
        },
    )
}

fn blockquote_rule() -> Rule {
    Rule::for_tag("blockquote", |_, content, _| {
        let content = content.trim();
        if content.is_empty() {
            return String::new();
        }
        let lines: Vec<&str> = content.lines().collect();
        let quoted: Vec<String> = lines.iter().map(|line| format!("> {}", line)).collect();
        format!("\n\n{}\n\n", quoted.join("\n"))
    })
}

fn list_rule() -> Rule {
    Rule::new(Filter::tags(&["ul", "ol"]), |node, content, _| {
        let content = content.trim();

        // Check if this list is nested inside a list item
        let is_nested = node
            .parent_tag()
            .map(|t| t == "li")
            .unwrap_or(false);

        if is_nested {
            // Nested lists don't get surrounding newlines
            format!("\n{}", content)
        } else {
            format!("\n\n{}\n\n", content)
        }
    })
}

fn list_item_rule() -> Rule {
    Rule::for_tag("li", |node, content, options| {
        let content = content
            .trim()
            .replace("\n\n\n", "\n\n")
            .replace('\n', "\n    "); // Indent continuation lines

        // Check if parent is ordered list
        let is_ordered = node
            .parent_tag()
            .map(|t| t == "ol")
            .unwrap_or(false);

        let prefix = if is_ordered {
            // For ordered lists, we need to track the item index
            // Since we don't have sibling access in NodeRef, we'll use a simple approach
            // The actual index will be computed during tree traversal
            // For now, use placeholder that gets replaced
            format!("1.  ")
        } else {
            format!("{}   ", options.bullet_list_marker)
        };

        format!("{}{}\n", prefix, content)
    })
}

fn indented_code_block_rule() -> Rule {
    Rule::new(
        Filter::predicate(|tag, node, options| {
            if tag != "pre" {
                return false;
            }
            // Check if first child is <code>
            let has_code = node
                .element_children()
                .any(|c| c.tag_name() == "code");
            has_code && matches!(options.code_block_style, CodeBlockStyle::Indented)
        }),
        |node, _, _| {
            // Get the text content from the code element
            let code_content: String = node
                .element_children()
                .find(|c| c.tag_name() == "code")
                .map(|c| c.text_content())
                .unwrap_or_default();

            let lines: Vec<&str> = code_content.lines().collect();
            let indented: Vec<String> = lines.iter().map(|line| format!("    {}", line)).collect();

            format!("\n\n{}\n\n", indented.join("\n"))
        },
    )
}

fn fenced_code_block_rule() -> Rule {
    Rule::new(
        Filter::predicate(|tag, node, options| {
            if tag != "pre" {
                return false;
            }
            let has_code = node
                .element_children()
                .any(|c| c.tag_name() == "code");
            has_code && matches!(options.code_block_style, CodeBlockStyle::Fenced)
        }),
        |node, _, options| {
            let code_node = node
                .element_children()
                .find(|c| c.tag_name() == "code");

            let code_node = match code_node {
                Some(c) => c,
                None => return String::new(),
            };

            let code_content = code_node.text_content();

            // Extract language from class
            let class = code_node.attr("class").unwrap_or("");
            let language = class
                .split_whitespace()
                .find(|c| c.starts_with("language-"))
                .map(|c| &c[9..])
                .unwrap_or("");

            let fence = &options.fence;
            format!(
                "\n\n{}{}\n{}\n{}\n\n",
                fence,
                language,
                code_content.trim_end(),
                fence
            )
        },
    )
}

fn horizontal_rule() -> Rule {
    Rule::for_tag("hr", |_, _, options| {
        format!("\n\n{}\n\n", options.hr)
    })
}

fn inline_link_rule() -> Rule {
    Rule::new(
        Filter::predicate(|tag, node, options| {
            tag == "a"
                && node.attr("href").is_some()
                && matches!(options.link_style, LinkStyle::Inlined)
        }),
        |node, content, _| {
            let href = clean_attribute(node.attr("href"));
            let title = node.attr("title");

            if href.is_empty() && title.is_none() {
                return content.to_string();
            }

            let title_part = title.map(|t| format!(" \"{}\"", t)).unwrap_or_default();

            format!("[{}]({}{})", content, href, title_part)
        },
    )
}

fn reference_link_rule() -> Rule {
    Rule::new(
        Filter::predicate(|tag, node, options| {
            tag == "a"
                && node.attr("href").is_some()
                && matches!(options.link_style, LinkStyle::Referenced)
        }),
        |node, content, _| {
            let href = clean_attribute(node.attr("href"));
            let title = node.attr("title");

            if href.is_empty() {
                return content.to_string();
            }

            let title_part = title.map(|t| format!(" \"{}\"", t)).unwrap_or_default();

            // For now, use inline style for referenced links
            // Full reference link support would require state tracking
            format!("[{}]({}{})", content, href, title_part)
        },
    )
}

fn emphasis_rule() -> Rule {
    Rule::new(Filter::tags(&["em", "i"]), |_, content, options| {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        let delimiter = options.em_delimiter;
        format!("{}{}{}", delimiter, content, delimiter)
    })
}

fn strong_rule() -> Rule {
    Rule::new(Filter::tags(&["strong", "b"]), |_, content, options| {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        let delimiter = &options.strong_delimiter;
        format!("{}{}{}", delimiter, content, delimiter)
    })
}

fn code_rule() -> Rule {
    Rule::new(
        Filter::predicate(|tag, node, _| {
            // Match <code> that is NOT inside <pre>
            if tag != "code" {
                return false;
            }
            // Check parent is not <pre>
            node.parent_tag()
                .map(|t| t != "pre")
                .unwrap_or(true)
        }),
        |node, _, _| {
            let content = node.text_content();
            if content.is_empty() {
                return String::new();
            }

            // Count backticks needed
            let max_consecutive_backticks = content
                .chars()
                .fold((0, 0), |(max, current), c| {
                    if c == '`' {
                        (max.max(current + 1), current + 1)
                    } else {
                        (max, 0)
                    }
                })
                .0;

            let backticks = "`".repeat((max_consecutive_backticks + 1).max(1));

            // Add spacing if content starts/ends with backtick or space
            let needs_space =
                content.starts_with('`') || content.ends_with('`') || content.starts_with(' ') || content.ends_with(' ');

            if needs_space && max_consecutive_backticks > 0 {
                format!("{} {} {}", backticks, content, backticks)
            } else {
                format!("{}{}{}", backticks, content, backticks)
            }
        },
    )
}

fn image_rule() -> Rule {
    Rule::for_tag("img", |node, _, _| {
        let alt = clean_attribute(node.attr("alt"));
        let src = clean_attribute(node.attr("src"));
        let title = node.attr("title");

        if src.is_empty() {
            return String::new();
        }

        let title_part = title.map(|t| format!(" \"{}\"", t)).unwrap_or_default();

        format!("![{}]({}{})", alt, src, title_part)
    })
}
