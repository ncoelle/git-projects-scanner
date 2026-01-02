//! # git-projects-core
//!
//! A library for scanning and cataloging Git projects on your local file system.
//!
//! ## Features
//!
//! - **Fast scanning** of directory trees to find Git repositories
//! - **Rich metadata extraction** including remotes, configuration, and submodules
//! - **Account detection** from remote URLs (best-effort extraction)
//! - **Localization support** via Fluent (currently English and German)
//! - **JSON serialization** for all data structures
//!
//! ## Quick Start
//!
//! ```no_run
//! use git_projects_core::{ScanConfig, ProjectScanner, DefaultScanner};
//! use std::path::PathBuf;
//!
//! let config = ScanConfig {
//!     root_paths: vec![PathBuf::from("/home/user/projects")],
//!     max_depth: Some(3),
//!     follow_symlinks: false,
//!     include_submodules: true,
//! };
//!
//! let scanner = DefaultScanner::new();
//! let projects = scanner.scan(&config).expect("Failed to scan");
//!
//! for project in projects {
//!     println!("{}: {}", project.name, project.path.display());
//! }
//! ```
//!
//! ## Architecture
//!
//! The library is organized into several modules:
//!
//! - [`models`] - Core data structures (GitProject, RemoteUrl, etc.)
//! - [`scanner`] - Scanner trait and default implementation
//! - [`git_analyzer`] - Low-level Git operations using gitoxide
//! - [`error`] - Custom error types
//! - [`l10n`] - Localization utilities
//!
//! ## CLI Binary
//!
//! This crate also provides a `projects-cli` binary for command-line usage.
//! See the binary's `--help` output for details.

// Module declarations
pub mod error;
pub mod git_analyzer;
pub mod l10n;
pub mod models;
pub mod scanner;

// Re-export commonly used types for convenience
pub use error::{Error, Result};
pub use models::{ConfigScope, GitConfig, GitProject, RemoteUrl, ScanConfig};
pub use scanner::{DefaultScanner, ProjectScanner};

// Re-export key functions from git_analyzer that might be useful to library users
pub use git_analyzer::{extract_git_config, extract_remote_urls};

/// Library version, derived from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_set() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "git-projects-core");
    }
}
