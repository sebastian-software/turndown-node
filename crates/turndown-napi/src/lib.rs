#![deny(clippy::all)]

mod lol_streaming;

use napi_derive::napi;
use turndown_core::{
    CodeBlockStyle, HeadingStyle, LinkReferenceStyle, LinkStyle, Options as CoreOptions,
};

#[napi(object)]
pub struct Options {
    pub heading_style: Option<String>,
    pub hr: Option<String>,
    pub bullet_list_marker: Option<String>,
    pub code_block_style: Option<String>,
    pub fence: Option<String>,
    pub em_delimiter: Option<String>,
    pub strong_delimiter: Option<String>,
    pub link_style: Option<String>,
    pub link_reference_style: Option<String>,
}

impl From<Options> for CoreOptions {
    fn from(opts: Options) -> Self {
        let mut result = CoreOptions::default();

        if let Some(style) = opts.heading_style {
            result.heading_style = match style.to_lowercase().as_str() {
                "atx" => HeadingStyle::Atx,
                _ => HeadingStyle::Setext,
            };
        }

        if let Some(hr) = opts.hr {
            result.hr = hr;
        }

        if let Some(marker) = opts.bullet_list_marker {
            if let Some(c) = marker.chars().next() {
                result.bullet_list_marker = c;
            }
        }

        if let Some(style) = opts.code_block_style {
            result.code_block_style = match style.to_lowercase().as_str() {
                "fenced" => CodeBlockStyle::Fenced,
                _ => CodeBlockStyle::Indented,
            };
        }

        if let Some(fence) = opts.fence {
            result.fence = fence;
        }

        if let Some(delim) = opts.em_delimiter {
            if let Some(c) = delim.chars().next() {
                result.em_delimiter = c;
            }
        }

        if let Some(delim) = opts.strong_delimiter {
            result.strong_delimiter = delim;
        }

        if let Some(style) = opts.link_style {
            result.link_style = match style.to_lowercase().as_str() {
                "referenced" => LinkStyle::Referenced,
                _ => LinkStyle::Inlined,
            };
        }

        if let Some(style) = opts.link_reference_style {
            result.link_reference_style = match style.to_lowercase().as_str() {
                "collapsed" => LinkReferenceStyle::Collapsed,
                "shortcut" => LinkReferenceStyle::Shortcut,
                _ => LinkReferenceStyle::Full,
            };
        }

        result
    }
}

#[napi]
pub struct TurndownService {
    options: CoreOptions,
}

#[napi]
impl TurndownService {
    #[napi(constructor)]
    pub fn new(options: Option<Options>) -> Self {
        let options = match options {
            Some(opts) => opts.into(),
            None => CoreOptions::default(),
        };
        Self { options }
    }

    /// Convert HTML to Markdown using lol_html streaming parser
    #[napi]
    pub fn turndown(&self, html: String) -> napi::Result<String> {
        // Use lol_html streaming conversion: HTML → AST → Markdown
        let ast = lol_streaming::html_to_ast(&html, &self.options);
        let result = turndown_core::serialize(&ast, &self.options);
        Ok(result)
    }

    /// Add a custom rule (currently no-op)
    #[napi]
    pub fn add_rule(&mut self, _key: String, _filter: String) -> napi::Result<&Self> {
        // TODO: Re-implement custom rules
        Ok(self)
    }

    /// Keep elements matching the filter as HTML (currently no-op)
    #[napi]
    pub fn keep(&mut self, _filter: Vec<String>) -> &Self {
        // TODO: Implement keep in streaming
        self
    }

    /// Remove elements matching the filter (currently no-op)
    #[napi]
    pub fn remove(&mut self, _filter: Vec<String>) -> &Self {
        // TODO: Implement remove in streaming
        self
    }

    /// Escape markdown special characters
    #[napi]
    pub fn escape(&self, text: String) -> String {
        escape_markdown(&text)
    }
}

fn escape_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for c in text.chars() {
        match c {
            '\\' | '*' | '_' | '[' | ']' | '#' | '+' | '-' | '!' | '`' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    result
}
