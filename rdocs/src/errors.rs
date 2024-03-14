//! A module for defining custom error types and result aliases for parsing
//!
//! This module provides custom error types for parsing and replacing operations
//! along with result aliases for convenient error handling.
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Start: `{pattern_start}`, End: `{pattern_end}`")]
    PatterNotEqual {
        pattern_start: String,
        pattern_end: String,
    },
}
#[derive(thiserror::Error, Debug)]
pub enum ReplacerError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error("Capture not found in position: {index}")]
    CaptureNotFound { index: i32 },
}

pub type ParserResult<T> = std::result::Result<T, ParseError>;
pub type ReplacerResult<T> = std::result::Result<T, ReplacerError>;
