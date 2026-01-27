//! Rule and Filter types for HTML conversion.

use scraper::ElementRef;

use crate::service::TurndownOptions;

/// Type alias for replacement functions
pub type ReplacementFn = Box<dyn Fn(&ElementRef, &str, &TurndownOptions) -> String + Send + Sync>;

/// A filter determines which elements a rule applies to
pub enum Filter {
    /// Match a single tag name
    TagName(String),
    /// Match any of multiple tag names
    TagNames(Vec<String>),
    /// Match using a predicate function
    Predicate(Box<dyn Fn(&str, &ElementRef, &TurndownOptions) -> bool + Send + Sync>),
}

impl Filter {
    /// Create a filter for a single tag
    pub fn tag(name: &str) -> Self {
        Filter::TagName(name.to_lowercase())
    }

    /// Create a filter for multiple tags
    pub fn tags(names: &[&str]) -> Self {
        Filter::TagNames(names.iter().map(|s| s.to_lowercase()).collect())
    }

    /// Create a filter with a predicate
    pub fn predicate<F>(f: F) -> Self
    where
        F: Fn(&str, &ElementRef, &TurndownOptions) -> bool + Send + Sync + 'static,
    {
        Filter::Predicate(Box::new(f))
    }

    /// Check if this filter matches an element
    pub fn matches(&self, tag: &str, element: &ElementRef, options: &TurndownOptions) -> bool {
        let tag_lower = tag.to_lowercase();
        match self {
            Filter::TagName(t) => tag_lower == *t,
            Filter::TagNames(tags) => tags.contains(&tag_lower),
            Filter::Predicate(f) => f(&tag_lower, element, options),
        }
    }
}

/// A rule defines how to convert a matched HTML element to Markdown
pub struct Rule {
    /// Filter to determine which elements this rule applies to
    pub filter: Filter,
    /// Replacement function that generates Markdown
    pub replacement: ReplacementFn,
}

impl Rule {
    /// Create a new rule
    pub fn new<F>(filter: Filter, replacement: F) -> Self
    where
        F: Fn(&ElementRef, &str, &TurndownOptions) -> String + Send + Sync + 'static,
    {
        Self {
            filter,
            replacement: Box::new(replacement),
        }
    }

    /// Create a rule that matches a single tag
    pub fn for_tag<F>(tag: &str, replacement: F) -> Self
    where
        F: Fn(&ElementRef, &str, &TurndownOptions) -> String + Send + Sync + 'static,
    {
        Self::new(Filter::tag(tag), replacement)
    }

    /// Create a rule that matches multiple tags
    pub fn for_tags<F>(tags: &[&str], replacement: F) -> Self
    where
        F: Fn(&ElementRef, &str, &TurndownOptions) -> String + Send + Sync + 'static,
    {
        Self::new(Filter::tags(tags), replacement)
    }

    /// Apply this rule's replacement
    pub fn replace(&self, element: &ElementRef, content: &str, options: &TurndownOptions) -> String {
        (self.replacement)(element, content, options)
    }
}
