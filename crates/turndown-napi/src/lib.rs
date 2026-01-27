#![deny(clippy::all)]

use napi_derive::napi;

use turndown::{
    CodeBlockStyle, Filter, HeadingStyle, LinkReferenceStyle, LinkStyle, Rule,
    TurndownOptions, TurndownService as RustTurndownService,
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

impl From<Options> for TurndownOptions {
    fn from(opts: Options) -> Self {
        let mut result = TurndownOptions::default();

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
    inner: RustTurndownService,
}

#[napi]
impl TurndownService {
    #[napi(constructor)]
    pub fn new(options: Option<Options>) -> Self {
        let inner = match options {
            Some(opts) => RustTurndownService::with_options(opts.into()),
            None => RustTurndownService::new(),
        };
        Self { inner }
    }

    /// Convert HTML to Markdown
    #[napi]
    pub fn turndown(&self, html: String) -> napi::Result<String> {
        self.inner
            .turndown_html(&html)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Add a custom rule
    #[napi]
    pub fn add_rule(&mut self, key: String, filter: String) -> napi::Result<&Self> {
        // For now, only support simple tag-based rules from JS
        // Full rule support would require more complex bindings
        let rule = Rule::for_tag(&filter, |_, content, _| content.to_string());
        self.inner.add_rule(&key, rule);
        Ok(self)
    }

    /// Keep elements matching the filter as HTML
    #[napi]
    pub fn keep(&mut self, filter: Vec<String>) -> &Self {
        for tag in filter {
            self.inner.keep(Filter::TagName(tag.to_lowercase()));
        }
        self
    }

    /// Remove elements matching the filter
    #[napi]
    pub fn remove(&mut self, filter: Vec<String>) -> &Self {
        for tag in filter {
            self.inner.remove(Filter::TagName(tag.to_lowercase()));
        }
        self
    }

    /// Escape markdown special characters
    #[napi]
    pub fn escape(&self, text: String) -> String {
        self.inner.escape(&text)
    }
}
