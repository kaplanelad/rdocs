//! A module for performing content replacement in files based on specified
//! patterns.
//!
//! This module provides functionality to replace content between specified
//! start and end patterns in files.
use std::{
    fmt,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;

use crate::{
    collect::Collector,
    errors::{ReplacerError, ReplacerResult},
    parser,
};

lazy_static! {
    static ref DEFAULT_START_PATTERN: &'static str = r"(<!--\s*📖(ID)\s*-->)";
    static ref DEFAULT_END_PATTERN: &'static str = r"(<!--\s*(ID)📖\s*-->)";
}

/// Enum representing the status of a content replacement operation.
#[derive(Debug)]
pub enum ReplaceStatus {
    Error(String),
    NotFound(String),
    Equal(String),
    Replaced(String, String, String),
}

impl fmt::Display for ReplaceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error(_) => write!(f, "error"),
            Self::NotFound(_) => write!(f, "not found"),
            Self::Equal(_) => write!(f, "equal"),
            Self::Replaced(_, _, _) => write!(f, "replaced"),
        }
    }
}

/// Struct representing the start and end patterns used for replacement.
pub struct Replace {
    pub start: String,
    pub end: String,
}

/// Struct representing the result of a content replacement operation.
#[derive(Debug)]
pub struct ReplaceResult {
    pub path: PathBuf,
    pub status: ReplaceStatus,
}

impl Default for Replace {
    /// Creates a default instance of [`Replace`] with predefined start and end
    /// patterns.
    fn default() -> Self {
        Self {
            start: DEFAULT_START_PATTERN.to_string(),
            end: DEFAULT_END_PATTERN.to_string(),
        }
    }
}

impl Replace {
    /// Replaces content in files based on the provided collector and parsed
    /// contents.
    #[must_use]
    pub fn replace_content(
        &self,
        collector: &Collector,
        parse_contents: &Vec<parser::ContentResults>,
    ) -> Vec<ReplaceResult> {
        let files = collector.collect_files();
        files
            .par_iter()
            .flat_map(|path| {
                let span =
                    tracing::span!(tracing::Level::TRACE, "replace_content", path = %path.display());
                let _guard = span.enter();

                match self.replace_with_save(path, parse_contents) {
                    Ok(status) => Some(status),
                    Err(err) => {
                        tracing::error!(err = %err, "could not replace content");
                        Some(vec![ReplaceResult{ path: path.clone(), status: ReplaceStatus::Error(err.to_string()) }])
                    }
                }

            })
            .flatten()
            .collect::<Vec<_>>()
    }

    /// Retrieves statistics about content replacement in files based on the
    /// provided collector and parsed contents.
    #[must_use]
    pub fn stats(
        &self,
        collector: &Collector,
        parse_contents: &Vec<parser::ContentResults>,
    ) -> Vec<ReplaceResult> {
        let files = collector.collect_files();
        files
            .par_iter()
            .flat_map(|path| {
                let span =
                    tracing::span!(tracing::Level::TRACE, "replace_content", path = %path.display());
                let _guard = span.enter();

                match self.replace(path, parse_contents) {
                    Ok((_, status)) => Some(status),
                    Err(err) => {
                        tracing::error!(err = %err, "could not replace content");
                        Some(vec![ReplaceResult{ path: path.clone(), status: ReplaceStatus::Error(err.to_string()) }])
                    }
                }

            })
            .flatten()
            .collect::<Vec<_>>()
    }

    /// Execute replace block content and save the new content to the given
    /// path.
    ///
    /// # Errors
    /// When exec return an error or could not save the new content to the given
    /// path
    pub fn replace_with_save(
        &self,
        path: &Path,
        parse_contents: &Vec<parser::ContentResults>,
    ) -> ReplacerResult<Vec<ReplaceResult>> {
        let (new_content, status) = self.replace(path, parse_contents)?;

        let is_changed = status
            .iter()
            .any(|s| matches!(s.status, ReplaceStatus::Replaced(_, _, _)));

        if is_changed {
            let mut file = File::create(path)?;
            file.write_all(new_content.as_bytes())?;
            Ok(status)
        } else {
            Ok(status)
        }
    }

    /// Execute replace block content and save the new content to the given
    /// path.
    ///
    /// # Errors
    /// When when could not read the file or could not capture the pattern
    /// replacement regex
    pub fn replace(
        &self,
        path: &Path,
        parse_contents: &Vec<parser::ContentResults>,
    ) -> ReplacerResult<(String, Vec<ReplaceResult>)> {
        let mut content = std::fs::read_to_string(path)?;
        let mut results = vec![];
        for parse_content in parse_contents {
            let status = self.find_and_replace(&content, parse_content)?;
            if let ReplaceStatus::Replaced(_, all_content, _) = &status {
                content = all_content.to_string();
            }
            results.push(ReplaceResult {
                path: path.to_path_buf(),
                status,
            });
        }

        Ok((content, results))
    }

    /// Find and replace the content between two patterns based on capturing
    /// details.
    ///
    /// This function takes gets [`parser::ContentResults`] collected data,
    /// which contains the results of parsing the content. It searches for a
    /// pattern match in the given content and replaces the content between
    /// specified capturing groups.
    ///
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// * When a match is found, but the capturing details are invalid, leading
    ///   to an unsuccessful replacement.
    fn find_and_replace(
        &self,
        content: &str,
        parse_content: &parser::ContentResults,
    ) -> ReplacerResult<ReplaceStatus> {
        let start_re_pattern = self.start.replace("ID", &parse_content.metadata.id);
        let end_re_pattern = self.end.replace("ID", &parse_content.metadata.id);
        let re = Regex::new(&format!("(?s){start_re_pattern}(.*){end_re_pattern}"))?;

        if let Some(capture) = re.captures(content) {
            if capture
                .get(3)
                .ok_or(ReplacerError::CaptureNotFound { index: 3 })?
                .as_str()
                .trim()
                == parse_content.data
            {
                return Ok(ReplaceStatus::Equal(parse_content.metadata.id.to_string()));
            }

            let keep_start = capture
                .get(1)
                .ok_or(ReplacerError::CaptureNotFound { index: 1 })?
                .as_str();
            let keep_end = capture
                .get(4)
                .ok_or(ReplacerError::CaptureNotFound { index: 4 })?
                .as_str();

            let replace = format!("{}\n{}\n{}", keep_start, &parse_content.data, keep_end);
            return Ok(ReplaceStatus::Replaced(
                parse_content.metadata.id.to_string(),
                re.replace_all(content, &replace).to_string(),
                parse_content.data.to_string(),
            ));
        }

        Ok(ReplaceStatus::NotFound(
            parse_content.metadata.id.to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use insta::{assert_debug_snapshot, with_settings};

    use super::*;

    pub fn get_mock_data() -> PathBuf {
        let content = r"files:
  - path: README.md
    content: |
        some text
        some text
        <!-- 📖REPLACE-1 -->
        R
        D
        O
        C
        S
        <!-- REPLACE-1📖 -->
        some text
        some text
        <!-- 📖REPLACE-2 -->
        R
        D
        O
        C
        S
        <!-- REPLACE-2📖 -->()
  ";
        tree_fs::from_yaml_str(content).unwrap()
    }

    #[test]
    fn can_replace() {
        let replacer = Replace::default();
        let contents: Vec<parser::ContentResults> = vec![
            parser::ContentResults {
                metadata: parser::ContentMetadata {
                    id: "REPLACE-1".to_string(),
                },
                data: "NEW CONTENT1".to_string(),
            },
            parser::ContentResults {
                metadata: parser::ContentMetadata {
                    id: "REPLACE-2".to_string(),
                },
                data: "NEW CONTENT2".to_string(),
            },
        ];

        let data = get_mock_data();

        with_settings!({
            filters => vec![
                ("path: .*","path: REDUCT")
            ]
        }, {
            assert_debug_snapshot!(replacer.replace(data.join("README.md").as_path(), &contents));
        });
    }

    #[test]
    fn replace_with_save() {
        let replacer = Replace::default();
        let contents: Vec<parser::ContentResults> = vec![
            parser::ContentResults {
                metadata: parser::ContentMetadata {
                    id: "REPLACE-1".to_string(),
                },
                data: "NEW CONTENT1".to_string(),
            },
            parser::ContentResults {
                metadata: parser::ContentMetadata {
                    id: "REPLACE-2".to_string(),
                },
                data: "NEW CONTENT2".to_string(),
            },
        ];

        let data = get_mock_data();

        with_settings!({
            filters => vec![
                ("path: .*","path: REDUCT")
            ]
        }, {
        assert_debug_snapshot!(
            replacer.replace_with_save(data.join("README.md").as_path(), &contents)
        );
        });
        assert_debug_snapshot!(std::fs::read_to_string(data.join("README.md")).unwrap());
    }
}
