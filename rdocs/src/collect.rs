//! A module for collecting files based on include and exclude patterns.
//!
//! This module provides functionality to collect files from a specified folder
//! while respecting configured include and exclude patterns.
//!
//! # Examples
//!
//! ```
//! use rdocs::collect::Collector;
//! use std::path::Path;
//!
//! let folder = Path::new("./fixtures");
//! let collector = Collector::new(folder).expect("Failed to create collector instance");
//!
//! let files = collector.collect_files();
//! println!("Collected files: {:?}", files);
//! ```
//!
//! ```
//! use rdocs::collect::{Collector, Config};
//! use std::path::Path;
//! use regex::Regex;
//!
//! let folder = Path::new("./fixtures");
//! let config = Config{
//!     includes: vec![],
//!     excludes: vec![Regex::new("exclude.rs").unwrap()].into()
//! };
//! let collector = Collector::from_config(folder, &config).expect("Failed to create collector instance");
//!
//! let files = collector.collect_files();
//! println!("Collected files: {:?}", files);
//! ```
use std::{
    io,
    path::{Path, PathBuf},
    sync::mpsc,
};

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Represents a file collector configured with include and exclude patterns.
#[derive(Debug)]
pub struct Collector {
    /// The base folder from which files are collected.
    pub folder: PathBuf,
    config: Config,
}

/// Represents configuration for the file collector.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Patterns to include files.
    #[serde(with = "serde_regex", default)]
    pub includes: Vec<Regex>,
    /// Patterns to exclude files.
    #[serde(with = "serde_regex", default)]
    pub excludes: Vec<Regex>,
}

impl Collector {
    /// Creates a new instance of [`Collector`] with the specified base folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided folder path is invalid.
    pub fn new(folder: &Path) -> io::Result<Self> {
        Ok(Self {
            folder: folder.canonicalize()?,
            config: Config::default(),
        })
    }

    /// Create [`Collector`] instance from the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided folder path is invalid.
    pub fn from_config(folder: &Path, config: &Config) -> io::Result<Self> {
        Ok(Self {
            folder: folder.canonicalize()?,
            config: config.clone(),
        })
    }

    /// Checks if a file should be excluded based on configured exclude
    /// patterns.
    fn should_exclude(&self, path: &Path) -> bool {
        let path = path
            .strip_prefix(&self.folder)
            .unwrap()
            .display()
            .to_string();

        for exclude in &self.config.excludes {
            if exclude.is_match(&path) {
                tracing::trace!("file excluded from configurations");
                return true;
            }
        }
        false
    }

    /// Checks if a file should be included based on configured include
    /// patterns.
    fn should_include(&self, path: &Path) -> bool {
        let path = path
            .strip_prefix(&self.folder)
            .unwrap()
            .display()
            .to_string();

        if self.config.includes.is_empty() {
            return true;
        }

        for include in &self.config.includes {
            if include.is_match(&path) {
                tracing::trace!("file excluded from configurations");
                return true;
            }
        }
        tracing::debug!("file should not be included");
        false
    }

    /// Collects files in the specified folder, respecting exclude and include
    /// patterns.
    #[must_use]
    pub fn collect_files(&self) -> Vec<PathBuf> {
        let (tx, rx) = mpsc::channel();
        WalkBuilder::new(&self.folder)
            .build_parallel()
            .run(move || {
                let tx = tx.clone();
                Box::new(move |result| {
                    result.map_or_else(
                        |err| {
                            tracing::error!(err = %err,"dir entry error ");
                        },
                        |entry| {
                            if entry.path().is_file() {
                                let path = entry.path().to_owned();
                                if !self.should_exclude(path.as_path()) && self.should_include(path.as_path()){
                                    if let Err(err) = tx.send(path.clone()) {
                                        tracing::error!(err = %err,path = %path.display(),"error sending path to tx ");
                                    }
                                }
                            }
                        },
                    );
                    ignore::WalkState::Continue
                })
            });

        rx.into_iter().collect::<Vec<_>>()
    }
}
