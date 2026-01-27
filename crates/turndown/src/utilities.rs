//! Utility functions and constants for HTML processing.

/// Block-level HTML elements
pub const BLOCK_ELEMENTS: &[&str] = &[
    "address", "article", "aside", "audio", "blockquote", "body", "canvas",
    "center", "dd", "dir", "div", "dl", "dt", "fieldset", "figcaption",
    "figure", "footer", "form", "frameset", "h1", "h2", "h3", "h4", "h5",
    "h6", "header", "hgroup", "hr", "html", "isindex", "li", "main", "menu",
    "nav", "noframes", "noscript", "ol", "output", "p", "pre", "section",
    "table", "tbody", "td", "tfoot", "th", "thead", "tr", "ul",
];

/// Void (self-closing) HTML elements
pub const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input",
    "keygen", "link", "meta", "param", "source", "track", "wbr",
];

/// Elements that have meaning even when blank
pub const MEANINGFUL_WHEN_BLANK: &[&str] = &[
    "a", "table", "thead", "tbody", "tfoot", "th", "td", "iframe", "script",
    "audio", "video",
];

/// Check if a tag is a block-level element
pub fn is_block(tag: &str) -> bool {
    BLOCK_ELEMENTS.contains(&tag.to_lowercase().as_str())
}

/// Check if a tag is a void element
pub fn is_void(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag.to_lowercase().as_str())
}

/// Check if a tag is meaningful when blank
pub fn is_meaningful_when_blank(tag: &str) -> bool {
    MEANINGFUL_WHEN_BLANK.contains(&tag.to_lowercase().as_str())
}

/// Repeat a string n times
pub fn repeat(s: &str, n: usize) -> String {
    s.repeat(n)
}

/// Escape markdown special characters
pub fn escape_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for c in text.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')'
            | '#' | '+' | '-' | '.' | '!' | '|' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    result
}

/// Clean an attribute value (trim and handle empty)
pub fn clean_attribute(value: Option<&str>) -> String {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("*test*"), "\\*test\\*");
        assert_eq!(escape_markdown("_test_"), "\\_test\\_");
        assert_eq!(escape_markdown("[link]"), "\\[link\\]");
        assert_eq!(escape_markdown("normal"), "normal");
    }

    #[test]
    fn test_repeat() {
        assert_eq!(repeat("=", 5), "=====");
        assert_eq!(repeat("-", 3), "---");
    }

    #[test]
    fn test_is_block() {
        assert!(is_block("div"));
        assert!(is_block("p"));
        assert!(is_block("DIV"));
        assert!(!is_block("span"));
        assert!(!is_block("a"));
    }

    #[test]
    fn test_is_void() {
        assert!(is_void("br"));
        assert!(is_void("img"));
        assert!(is_void("HR"));
        assert!(!is_void("div"));
    }
}
