//! A module for exporting content
//!
//! This module provides functionality to export content to different formats
//! and destinations.

use std::{fs::File, io::prelude::*, path::PathBuf};

use crate::parser::ContentResults;

/// Constant holding the default file name used when exporting to a file without
/// a specified name.
const DEFAULT_FILE_NAME: &str = "rdocs";

/// Enum representing different export options.
pub enum Content {
    /// Export only the content in the patter
    Only(Output),
    /// Export all extractor data and metadata
    All(Output, Format),
}

/// Enum representing different output options.
pub enum Output {
    /// Export to a file with the specified path.
    Path(PathBuf),
    /// Export to STDOUT.
    Stdout,
}

/// Enum representing different export formats.
#[derive(clap::ValueEnum, Clone)]
pub enum Format {
    /// Export in JSON format.
    Json,
    /// Export in YAML format.
    Yaml,
}

impl Format {
    /// Get the file extension based on the export format.
    #[must_use]
    pub const fn get_extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
        }
    }
}

impl Output {
    /// Create a new `Output` instance based on the provided path option.
    #[must_use]
    pub fn new(path: Option<PathBuf>) -> Self {
        path.map_or(Self::Stdout, Self::Path)
    }
}

impl Content {
    /// Export content results based on the specified export options.
    ///
    /// # Errors
    ///
    /// When have io errors like read/write file or create directory or could
    /// not parse [`ContentResults`] to a given format
    pub fn export(&self, results: Vec<ContentResults>) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Only(output) => {
                for result in results {
                    match output {
                        Output::Path(path) => {
                            let file_path = path.join(result.metadata.id);
                            if let Some(parent) = file_path.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                            let mut file = File::create(file_path)?;
                            file.write_all(result.data.as_bytes())?;
                        }
                        Output::Stdout => println!("{}", result.data),
                    }
                }
            }
            Self::All(output, format) => {
                let content = match format {
                    Format::Json => serde_json::to_string_pretty(&results)?,
                    Format::Yaml => serde_yaml::to_string(&results)?,
                };
                match output {
                    Output::Path(path) => {
                        let file_path = if path.extension().is_some() {
                            path.clone()
                        } else {
                            let mut file_path = path.join(DEFAULT_FILE_NAME);
                            file_path.set_extension(format.get_extension());
                            file_path
                        };
                        if let Some(parent) = file_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        let mut file = File::create(file_path)?;
                        file.write_all(content.as_bytes())?;
                    }
                    Output::Stdout => println!("{content}"),
                }
            }
        }
        Ok(())
    }
}
