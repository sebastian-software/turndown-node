//! Configuration options for Markdown serialization

/// Heading style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadingStyle {
    /// Use setext-style headings (underlined with = or -)
    /// Only works for h1 and h2, falls back to ATX for h3-h6
    #[default]
    Setext,
    /// Use ATX-style headings (prefixed with #)
    Atx,
}

/// Code block style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeBlockStyle {
    /// Use indented code blocks (4 spaces)
    #[default]
    Indented,
    /// Use fenced code blocks (```)
    Fenced,
}

/// Link style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkStyle {
    /// Use inline links [text](url)
    #[default]
    Inlined,
    /// Use reference links [text][ref]
    Referenced,
}

/// Reference style for referenced links
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkReferenceStyle {
    /// Full reference: [text][label]
    #[default]
    Full,
    /// Collapsed reference: [text][]
    Collapsed,
    /// Shortcut reference: [text]
    Shortcut,
}

/// Options for Markdown serialization
#[derive(Debug, Clone)]
pub struct Options {
    /// Heading style (setext or atx)
    pub heading_style: HeadingStyle,

    /// Horizontal rule string
    pub hr: String,

    /// Bullet list marker
    pub bullet_list_marker: char,

    /// Code block style
    pub code_block_style: CodeBlockStyle,

    /// Fence string for fenced code blocks
    pub fence: String,

    /// Emphasis delimiter
    pub em_delimiter: char,

    /// Strong delimiter
    pub strong_delimiter: String,

    /// Link style
    pub link_style: LinkStyle,

    /// Reference style for referenced links
    pub link_reference_style: LinkReferenceStyle,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            heading_style: HeadingStyle::Setext,
            hr: "* * *".to_string(),
            bullet_list_marker: '*',
            code_block_style: CodeBlockStyle::Indented,
            fence: "```".to_string(),
            em_delimiter: '_',
            strong_delimiter: "**".to_string(),
            link_style: LinkStyle::Inlined,
            link_reference_style: LinkReferenceStyle::Full,
        }
    }
}
