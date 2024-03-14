//! A module for defining patterns used in parsing and extracting content.
//!
//! This module provides functionality to define patterns used to identify
//! content blocks in files.
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref DEFAULT_START: Regex = Regex::new(r"//\s*📖\s*#START").unwrap();
    static ref DEFAULT_END: Regex = Regex::new(r"//\s*📖\s*#END").unwrap();
    static ref DEFAULT_CLEANUPS: Vec<Regex> = vec![
        #[allow(clippy::trivial_regex)]
        Regex::new(r"//!").unwrap(),
    ];
}

/// Represents a pattern used for identifying content blocks in files.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pattern {
    /// The regular expression pattern to identify the start of a content block.
    #[serde(with = "serde_regex")]
    pub start: Regex,
    /// The regular expression pattern to identify the end of a content block.
    #[serde(with = "serde_regex")]
    pub end: Regex,
    /// A list of regular expression patterns used for cleanup within the
    /// content block.
    #[serde(with = "serde_regex")]
    pub cleanups: Vec<Regex>,
}

impl Default for Pattern {
    /// Creates a default instance of [`Pattern`] with predefined start, end,
    /// and cleanup patterns.
    fn default() -> Self {
        Self {
            start: DEFAULT_START.to_owned(),
            end: DEFAULT_END.to_owned(),
            cleanups: DEFAULT_CLEANUPS.to_owned(),
        }
    }
}

impl Pattern {
    /// Checks if the provided string matches the start pattern of the pattern.
    #[must_use]
    pub fn start_with(&self, str: &str) -> bool {
        self.start.is_match(str)
    }

    /// Checks if the provided string matches the end pattern of the pattern.
    #[must_use]
    pub fn end_with(&self, str: &str) -> bool {
        self.end.is_match(str)
    }

    /// Applies cleanup operations defined in the pattern to the provided text.
    #[must_use]
    pub fn cleanup(&self, text: &str) -> String {
        let mut text_result = text.to_string();

        for regex in &self.cleanups {
            text_result = regex.replace_all(&text_result, "").to_string();
        }

        text_result
    }
}

#[cfg(test)]
mod tests {

    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn default_pattern() {
        assert_debug_snapshot!(Pattern::default());
    }

    #[test]
    fn is_start_with() {
        let pattern = Pattern::default();
        assert!(pattern.start_with("   // 📖 #START"));
        assert!(pattern.start_with("// 📖   #START"));
        assert!(pattern.start_with("//📖#START <>"));
        assert!(!pattern.start_with("//📖"));
    }

    #[test]
    fn is_end_with() {
        let pattern = Pattern::default();
        assert!(pattern.end_with("   // 📖 #END"));
        assert!(pattern.end_with("// 📖   #END"));
        assert!(pattern.end_with("//📖#END <>"));
        assert!(!pattern.end_with("//📖"));
    }

    #[test]
    fn can_cleanup() {
        let pattern = Pattern::default();
        let text = ["//!"];
        // all cleanups list must be tested. if the lest is not the same, failing the
        // test.
        assert_eq!(text.len(), DEFAULT_CLEANUPS.len());
        assert_eq!(pattern.cleanup(&text.join(" ")), "");
    }
}
