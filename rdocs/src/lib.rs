//! # Rdocs: Code Documentation Made Simple
// 📖 #START <id:introduction>
//! ## Introduction
//! When working on open-source projects or internal projects, creating
//! effective documentation is essential. As a developer, I often faced the
//! challenge of generating and maintaining comprehensive project documentation.
//! The example below demonstrates various types of documentation I've included:
//!
//! - Examples showcasing how to use the application configuration.
//! - A README guide illustrating the usage of my CLI.
//! - Code snippets demonstrating the usage of my library in different
//!   scenarios.
//!
//! However, a common issue is ensuring that the examples provided in the
//! documentation remain accurate and up-to-date. What if the example code
//! changes? What if there's a typo in the documented code? How can we ensure
//! that the examples always reflect the current state of the codebase?
//!
//! Imagine a tool that allows you to extract code snippets, ensuring they are
//! not only reliable but also executable. What if you could easily incorporate
//! pieces of code that are known to work or leverage tested examples from
//! languages like Rust, which use `Doctest`? This tool is designed to address
//! these concerns.
// 📖 #END
//!
// 📖 #START <id:my-goal>
//! ## Goal
//! My objectives are as follows:
//!
//! - **Alignment with Code:** Ensure documentation consistently aligns with the
//!   codebase.
//! - **CI Validation:** Incorporate validation into Continuous Integration to
//!   ensure documentation validity.
//! - **User-Friendly:** Prioritize ease of use for all stakeholders.
//! - **Minimal Dependencies:** Enable documentation validity even without
//!   external tool dependencies.
// 📖 #END
//!
// 📖 #START <id:installation>
//! ## Installation
//!
//! Cargo install:
//! ```sh
//! cargo install rdocs
//! ```
//!
//! GitHub Releases:
//! https://github.com/kaplanelad/rdocs/releases/latest
// 📖 #END

#[cfg(feature = "cli")]
pub mod cli;
pub mod collect;
pub mod errors;
pub mod out;
pub mod parser;
pub mod pattern;
pub mod replacer;
