//! Rule system for HTML to Markdown conversion.

mod commonmark;
mod rule;

pub use commonmark::commonmark_rules;
pub use rule::{Filter, Rule};

use indexmap::IndexMap;
use scraper::ElementRef;

use crate::service::TurndownOptions;
use crate::utilities::outer_html;

/// Collection of rules for conversion
pub struct Rules {
    /// Custom rules added by the user (checked first)
    custom_rules: IndexMap<String, Rule>,
    /// Keep rules (preserve as HTML)
    keep_rules: Vec<Filter>,
    /// Remove rules (remove entirely)
    remove_rules: Vec<Filter>,
    /// Built-in CommonMark rules
    commonmark_rules: Vec<Rule>,
}

impl Rules {
    /// Create a new Rules instance with CommonMark rules
    pub fn new() -> Self {
        Self {
            custom_rules: IndexMap::new(),
            keep_rules: Vec::new(),
            remove_rules: Vec::new(),
            commonmark_rules: commonmark_rules(),
        }
    }

    /// Add a custom rule
    pub fn add(&mut self, key: &str, rule: Rule) {
        self.custom_rules.insert(key.to_string(), rule);
    }

    /// Add a keep filter
    pub fn keep(&mut self, filter: Filter) {
        self.keep_rules.push(filter);
    }

    /// Add a remove filter
    pub fn remove(&mut self, filter: Filter) {
        self.remove_rules.push(filter);
    }

    /// Find the appropriate rule for an element
    pub fn for_element<'a>(
        &'a self,
        element: &ElementRef,
        options: &TurndownOptions,
    ) -> Option<&'a Rule> {
        let tag = element.value().name();

        // Check custom rules first
        for rule in self.custom_rules.values() {
            if rule.filter.matches(tag, element, options) {
                return Some(rule);
            }
        }

        // Check CommonMark rules
        for rule in &self.commonmark_rules {
            if rule.filter.matches(tag, element, options) {
                return Some(rule);
            }
        }

        None
    }

    /// Check if an element should be kept as HTML
    pub fn should_keep(&self, element: &ElementRef, options: &TurndownOptions) -> bool {
        let tag = element.value().name();

        // Don't keep if a custom or commonmark rule matches
        for rule in self.custom_rules.values() {
            if rule.filter.matches(tag, element, options) {
                return false;
            }
        }
        for rule in &self.commonmark_rules {
            if rule.filter.matches(tag, element, options) {
                return false;
            }
        }

        // Check keep rules
        for filter in &self.keep_rules {
            if filter.matches(tag, element, options) {
                return true;
            }
        }

        false
    }

    /// Check if an element should be removed
    pub fn should_remove(&self, element: &ElementRef, options: &TurndownOptions) -> bool {
        let tag = element.value().name();

        // Don't remove if keep matches
        if self.should_keep(element, options) {
            return false;
        }

        // Don't remove if a custom or commonmark rule matches
        for rule in self.custom_rules.values() {
            if rule.filter.matches(tag, element, options) {
                return false;
            }
        }
        for rule in &self.commonmark_rules {
            if rule.filter.matches(tag, element, options) {
                return false;
            }
        }

        // Check remove rules
        for filter in &self.remove_rules {
            if filter.matches(tag, element, options) {
                return true;
            }
        }

        false
    }

    /// Get the keep replacement for an element
    pub fn keep_replacement(&self, element: &ElementRef) -> String {
        outer_html(element)
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::new()
    }
}
