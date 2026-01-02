//! Error types for git-projects-core.
//!
//! This module defines a custom error type using `thiserror` for the library's
//! public API, while using `anyhow` internally for error propagation in the CLI.

use std::path::PathBuf;
use thiserror::Error;

/// A specialized Result type for git-projects-core operations.
///
/// This is a convenience alias that uses our custom [`Error`] type.
///
/// # Example
///
/// ```
/// use git_projects_core::Result;
///
/// fn scan_project() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during Git project scanning and analysis.
///
/// This enum covers all error cases that can happen when scanning
/// directories, reading Git repositories, and extracting metadata.
#[derive(Error, Debug)]
pub enum Error {
    /// An I/O error occurred while accessing the filesystem.
    ///
    /// This can happen during directory traversal, file reading, etc.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to open or read a Git repository.
    ///
    /// Includes the path that was being accessed and the underlying gitoxide error.
    #[error("Failed to open Git repository at {path}: {source}")]
    GitOpen {
        /// The path to the repository that couldn't be opened.
        path: PathBuf,
        /// The underlying gitoxide error.
        #[source]
        source: gix::open::Error,
    },

    /// Failed to discover a Git repository in the given path.
    ///
    /// This occurs when trying to find the `.git` directory starting from a path.
    #[error("Failed to discover Git repository from {path}: {source}")]
    GitDiscover {
        /// The path where discovery was attempted.
        path: PathBuf,
        /// The underlying gitoxide error.
        #[source]
        source: gix::discover::Error,
    },

    /// Failed to read Git configuration.
    ///
    /// This can happen when trying to extract user.name or user.email.
    #[error("Failed to read Git config for {path}: {source}")]
    GitConfig {
        /// The repository path where config reading failed.
        path: PathBuf,
        /// The underlying gitoxide config error.
        #[source]
        source: gix::config::Error,
    },

    /// Failed to access remote configuration.
    ///
    /// This occurs when trying to list or read remote URLs.
    #[error("Failed to access remotes for {path}: {message}")]
    GitRemote {
        /// The repository path where remote access failed.
        path: PathBuf,
        /// A descriptive error message.
        message: String,
    },

    /// A required path does not exist.
    ///
    /// Used when a specified scan root or target path is invalid.
    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),

    /// The specified path is not a directory.
    ///
    /// Used when trying to scan a file instead of a directory.
    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),

    /// Failed to parse a URL.
    ///
    /// Occurs when trying to extract service/account information from malformed URLs.
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    /// Localization system error.
    ///
    /// This covers errors in loading or using Fluent translation files.
    #[error("Localization error: {0}")]
    L10n(String),

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A generic error with a custom message.
    ///
    /// Used for miscellaneous errors that don't fit other categories.
    #[error("{0}")]
    Other(String),
}

// Helper constructors for common error cases
impl Error {
    /// Creates a GitOpen error.
    ///
    /// # Example
    ///
    /// ```ignore
    /// return Err(Error::git_open(path, err));
    /// ```
    pub fn git_open(path: impl Into<PathBuf>, source: gix::open::Error) -> Self {
        Error::GitOpen {
            path: path.into(),
            source,
        }
    }

    /// Creates a GitDiscover error.
    pub fn git_discover(path: impl Into<PathBuf>, source: gix::discover::Error) -> Self {
        Error::GitDiscover {
            path: path.into(),
            source,
        }
    }

    /// Creates a GitConfig error.
    pub fn git_config(path: impl Into<PathBuf>, source: gix::config::Error) -> Self {
        Error::GitConfig {
            path: path.into(),
            source,
        }
    }

    /// Creates a GitRemote error.
    pub fn git_remote(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Error::GitRemote {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Creates a PathNotFound error.
    pub fn path_not_found(path: impl Into<PathBuf>) -> Self {
        Error::PathNotFound(path.into())
    }

    /// Creates a NotADirectory error.
    pub fn not_a_directory(path: impl Into<PathBuf>) -> Self {
        Error::NotADirectory(path.into())
    }

    /// Creates an InvalidUrl error.
    pub fn invalid_url(url: impl Into<String>) -> Self {
        Error::InvalidUrl(url.into())
    }

    /// Creates an L10n error.
    pub fn l10n(message: impl Into<String>) -> Self {
        Error::L10n(message.into())
    }

    /// Creates an Other error.
    pub fn other(message: impl Into<String>) -> Self {
        Error::Other(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::path_not_found("/nonexistent/path");
        assert_eq!(err.to_string(), "Path does not exist: /nonexistent/path");

        let err = Error::not_a_directory("/some/file.txt");
        assert_eq!(err.to_string(), "Path is not a directory: /some/file.txt");

        let err = Error::invalid_url("not a url");
        assert_eq!(err.to_string(), "Invalid URL format: not a url");
    }

    #[test]
    fn test_error_constructors() {
        let err = Error::git_remote("/path/to/repo", "Could not fetch");
        match err {
            Error::GitRemote { path, message } => {
                assert_eq!(path, PathBuf::from("/path/to/repo"));
                assert_eq!(message, "Could not fetch");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        assert_eq!(returns_result().unwrap(), 42);
    }
}
