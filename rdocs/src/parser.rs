//! A module for parsing and extracting content based on patterns from files.
//!
//! This module provides functionality to parse files and extract content based
//! on specified patterns.
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{self, BufRead, Read},
    path::Path,
};

use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    /// Extract the block metadata
    static ref PARSER_INFO_RE: Regex = Regex::new(r"(?:<id:(.*?)>)").unwrap();
}

use crate::{
    collect::Collector,
    errors::{ParseError, ParserResult},
    pattern::Pattern,
};

/// Represents a parser for extracting content from files.
#[derive(Default)]
pub struct Parser {
    config: Config,
}

/// Represents configuration for the parser, including patterns to match.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Patterns used by the parser.
    patterns: Vec<Pattern>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            patterns: vec![Pattern::default()],
        }
    }
}

/// Represents content associated with a file path and a list of patterns.
#[derive(Debug)]
pub struct Content<'a> {
    path: &'a Path,
    patterns: &'a Vec<Pattern>,
    pub expected_capture_count: usize,
}

/// Represents a block of content with metadata and lines.
#[derive(Debug, Serialize)]
pub struct ContentBlock {
    pub metadata: ContentMetadata,
    pub lines: Vec<String>,
}

/// Represents metadata associated with content, including an ID.
#[derive(Debug, Serialize)]
pub struct ContentMetadata {
    pub id: String,
}

/// Represents the final results after extracting content, including metadata
/// and cleaned-up data.
#[derive(Debug, Serialize)]
pub struct ContentResults {
    pub metadata: ContentMetadata,
    pub data: String,
}

impl Parser {
    /// Creates a new instance of [`Parser`] with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Creates a new instance of [`Parser`] with the specified configuration.
    #[must_use]
    pub const fn with_config(config: Config) -> Self {
        Self { config }
    }

    /// Extracts content from files collected by the provided collector.
    #[must_use]
    pub fn extract_content(&self, collector: &Collector) -> Vec<ContentResults> {
        let files = collector.collect_files();
        files
            .par_iter()
            .flat_map(|path| {
                let span =
                    tracing::span!(tracing::Level::TRACE, "collect_file", path = %path.display());
                let _guard = span.enter();

                let parse_content = match Content::new(path.as_path(), &self.config.patterns) {
                    Ok(parse_content) => parse_content,
                    Err(err) => {
                        tracing::error!(err = %err, "could not parse file content");
                        return None;
                    }
                };

                if parse_content.expected_capture_count == 0 {
                    tracing::trace!("captures not found in file");
                    return None;
                }

                match parse_content.extract() {
                    Ok(res) => Some(res),
                    Err(err) => {
                        tracing::error!(err = %err,"could not parse file content");
                        None
                    }
                }
            })
            .flatten()
            .collect::<Vec<_>>()
    }
}

impl ContentMetadata {
    /// Creates a new instance of [`ContentMetadata`] from the specified string.
    #[must_use]
    pub fn new(str: &str) -> Option<Self> {
        let captures = PARSER_INFO_RE.captures(str)?;
        Some(Self {
            id: captures.get(1).map_or_else(
                || {
                    tracing::info!("id not found");
                    None
                },
                |m| Some(m.as_str().trim().to_string()),
            )?,
        })
    }
}

impl<'a> Content<'a> {
    /// Creates a new instance of [`Content`].
    ///
    /// # Errors
    ///
    /// when could not read the file or the cound of start pattern not equal to
    /// the end pattern
    pub fn new(path: &'a Path, patterns: &'a Vec<Pattern>) -> ParserResult<Self> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut expected_capture_count = 0;
        for pattern in patterns {
            let start_count = pattern.start.captures_iter(&content).count();
            let end_count = pattern.end.captures_iter(&content).count();
            expected_capture_count += start_count;
            if start_count != end_count {
                return Err(ParseError::PatterNotEqual {
                    pattern_start: pattern.start.to_string(),
                    pattern_end: pattern.end.to_string(),
                });
            }
        }

        Ok(Self {
            path,
            patterns,
            expected_capture_count,
        })
    }

    /// Extracts content based on the defined patterns.
    ///
    /// # Errors
    ///
    /// when could not read line content
    pub fn extract(self) -> ParserResult<Vec<ContentResults>> {
        let file = File::open(self.path)?;
        let reader = io::BufReader::new(file);

        let mut level_stack = HashMap::new();

        let mut collected_scoped_content = BTreeMap::new();

        let mut current_levels = vec![0; self.patterns.len()];
        let mut max_level = 0;

        for (line_index, line) in reader.lines().enumerate() {
            let line = line?;

            for (pattern_index, pattern) in self.patterns.iter().enumerate() {
                if pattern.start_with(&line) {
                    let Some(metadata) = ContentMetadata::new(&line) else {
                        tracing::warn!(
                            line_content = line,
                            line_index,
                            "pattern line has invalid format. invalid <id:[ID]>"
                        );
                        continue;
                    };

                    let content_block = ContentBlock {
                        metadata,
                        lines: vec![],
                    };

                    current_levels[pattern_index] += 1;
                    max_level = max_level.max(current_levels[pattern_index]);
                    level_stack.insert(pattern_index, line_index);
                    collected_scoped_content
                        .entry((pattern_index, current_levels[pattern_index]))
                        .or_insert(content_block);
                } else if pattern.end_with(&line) {
                    level_stack.remove(&pattern_index);
                } else if level_stack.contains_key(&pattern_index) {
                    if let Some(content) = collected_scoped_content
                        .get_mut(&(pattern_index, current_levels[pattern_index]))
                    {
                        content.lines.push(line.clone());
                    }
                }
            }
        }

        let mut results = vec![];
        for ((i, _), d) in collected_scoped_content {
            let match_content = d.lines.join("\n");
            let cleanup_result = if let Some(pattern) = self.patterns.get(i) {
                pattern.cleanup(&match_content)
            } else {
                tracing::debug!("skip cleanups. pattern index not found");
                match_content
            };

            results.push(ContentResults {
                metadata: d.metadata,
                data: cleanup_result.trim().to_string(),
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {

    use insta::{assert_debug_snapshot, with_settings};
    use regex::Regex;
    use tree_fs::Tree;

    use super::*;

    fn get_test_pattern() -> Vec<Pattern> {
        vec![
            Pattern {
                start: Regex::new(r".*#START").unwrap(),
                end: Regex::new(r".*#END").unwrap(),
                #[allow(clippy::trivial_regex)]
                cleanups: vec![Regex::new(r"```").unwrap()],
            },
            Pattern {
                start: Regex::new(r".*#PATTERN_2_START").unwrap(),
                end: Regex::new(r".*#PATTERN_2_END").unwrap(),
                #[allow(clippy::trivial_regex)]
                cleanups: vec![Regex::new(r"//!").unwrap()],
            },
        ]
    }

    #[test]
    fn can_create_new_content_instance() {
        let content = r"//#START
        pub fn test() {}
        //#END
        //#PATTERN_2_START
        pub fn test() bool{
           true
        }
        //#PATTERN_2_END
        #";
        let res = Tree::default().add("test.rs", content).create().unwrap();
        let patterns = get_test_pattern();
        with_settings!({
            filters => vec![
                ("path: .*","path: REDUCT")
            ]
        }, {
            assert_debug_snapshot!(Content::new(res.join("test.rs").as_path(), &patterns));
        });
    }

    #[test]
    fn patter_not_equal() {
        let content = r"//#START
        pub fn test(){}
        //#
        //#PATTERN_2_START
        pub fn test() bool{
           true
        }
        //#PATTERN_2_END
        #";
        let res = Tree::default().add("test.rs", content).create().unwrap();
        let patterns = get_test_pattern();
        with_settings!({
            filters => vec![
                ("path: .*","path: REDUCT")
            ]
        }, {
            assert_debug_snapshot!(Content::new(res.join("test.rs").as_path(), &patterns));
        });
    }

    #[test]
    fn can_extract() {
        let content = r#"//#START <id: first example>
        pub fn test() {
             another_function(5);
        }
        fn another_function(x: i32) {
            println!("The value of x is: {x}");
        }
        //#END
        //#PATTERN_2_START <id: second pattern >
        pub fn test() bool{
           true
        }
        //#PATTERN_2_END
        //#START MISSING ID
        pub fn test() {}
        //#END
        "#;
        let res = Tree::default().add("test.rs", content).create().unwrap();
        let patterns = get_test_pattern();
        let binding = res.join("test.rs");
        let c = Content::new(binding.as_path(), &patterns).unwrap();

        assert_debug_snapshot!(c.extract());
    }

    #[test]
    fn can_create_content_metadata() {
        assert_debug_snapshot!(ContentMetadata::new("<id: second pattern >"));
        assert!(ContentMetadata::new("<second pattern >").is_none());
    }
}
