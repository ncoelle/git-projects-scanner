//! Core data models for Git project metadata.
//!
//! All types in this module are designed to be JSON-serializable and match
//! the schema defined in `docs/API_SCHEMA.json`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a Git project (repository) on the local file system.
///
/// This is the primary data structure returned by scanners. It contains
/// comprehensive metadata about a Git repository including its location,
/// remotes, configuration, and submodule status.
///
/// # Example
///
/// ```
/// # use git_projects_core::GitProject;
/// # use std::path::PathBuf;
/// let project = GitProject {
///     name: "my-project".to_string(),
///     path: PathBuf::from("/home/user/projects/my-project"),
///     remotes: vec![],
///     config: None,
///     is_submodule: false,
///     has_submodules: false,
///     last_scanned: chrono::Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitProject {
    /// The name of the project (typically the directory name).
    ///
    /// Derived from the last component of the path.
    /// Example: `/home/user/projects/my-repo` → `"my-repo"`
    pub name: String,

    /// Absolute path to the Git repository root.
    ///
    /// This points to the directory containing the `.git` folder (or being
    /// the `.git` folder itself for bare repositories).
    pub path: PathBuf,

    /// List of remote URLs configured for this repository.
    ///
    /// Typically includes `origin`, but may contain multiple remotes.
    /// Empty if the repository has no remotes configured.
    pub remotes: Vec<RemoteUrl>,

    /// Git configuration (user.name, user.email) with scope information.
    ///
    /// `None` if configuration could not be read or doesn't exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<GitConfig>,

    /// Whether this repository is a submodule of another repository.
    ///
    /// Detected by checking for `.git` file (pointing to parent's .git/modules)
    /// instead of a `.git` directory.
    pub is_submodule: bool,

    /// Whether this repository contains submodules.
    ///
    /// Detected by checking for `.gitmodules` file in the repository root.
    pub has_submodules: bool,

    /// Timestamp when this project was last scanned.
    ///
    /// Useful for incremental scans and cache invalidation.
    pub last_scanned: DateTime<Utc>,
}

/// Represents a Git remote URL with associated metadata.
///
/// Stores the remote name (e.g., "origin") and its URL, along with
/// best-effort extraction of the hosting service and account name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RemoteUrl {
    /// The name of the remote (e.g., "origin", "upstream").
    pub name: String,

    /// The full URL of the remote.
    ///
    /// Can be HTTP(S), SSH, or Git protocol.
    /// Examples:
    /// - `https://github.com/user/repo.git`
    /// - `git@github.com:user/repo.git`
    /// - `ssh://git@gitlab.com/user/repo.git`
    pub url: String,

    /// The hosting service, if detectable.
    ///
    /// Extracted from well-known domains:
    /// - `github.com` → `Some("github")`
    /// - `gitlab.com` → `Some("gitlab")`
    /// - `bitbucket.org` → `Some("bitbucket")`
    /// - `unknown-git-host.com` → `None`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// The account/organization name, if extractable.
    ///
    /// Best-effort extraction from URL patterns:
    /// - `github.com/user/repo` → `Some("user")`
    /// - `gitlab.com/group/subgroup/repo` → `Some("group")`
    ///
    /// `None` if the URL structure doesn't match known patterns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
}

/// Git user configuration (user.name and user.email) with scope.
///
/// Represents the identity configuration found in Git config files.
/// The scope indicates where the configuration was found (local vs global).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitConfig {
    /// User's name from git config.
    ///
    /// Corresponds to `git config user.name`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,

    /// User's email from git config.
    ///
    /// Corresponds to `git config user.email`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,

    /// The scope where this configuration was found.
    ///
    /// Indicates whether the config is repository-specific or global.
    pub scope: ConfigScope,
}

/// The scope of a Git configuration setting.
///
/// Git config can be set at different levels. This enum tracks where
/// a particular configuration value was found.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigScope {
    /// Repository-local configuration (`.git/config`).
    ///
    /// Highest priority; overrides global settings.
    Local,

    /// User-global configuration (`~/.gitconfig` or `~/.config/git/config`).
    ///
    /// Applies to all repositories for the current user.
    Global,

    /// System-wide configuration (`/etc/gitconfig`).
    ///
    /// Lowest priority; applies to all users on the system.
    System,
}

/// Configuration for scanning operations.
///
/// Defines the parameters for how the filesystem should be scanned
/// to discover Git repositories.
///
/// # Example
///
/// ```
/// # use git_projects_core::ScanConfig;
/// # use std::path::PathBuf;
/// let config = ScanConfig {
///     root_paths: vec![
///         PathBuf::from("/home/user/projects"),
///         PathBuf::from("/home/user/code"),
///     ],
///     max_depth: Some(3),
///     follow_symlinks: false,
///     include_submodules: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Root directories to start scanning from.
    ///
    /// The scanner will recursively search these directories and their
    /// subdirectories for Git repositories.
    pub root_paths: Vec<PathBuf>,

    /// Maximum depth to recurse into subdirectories.
    ///
    /// - `None` → unlimited depth (scan entire tree)
    /// - `Some(0)` → only check root_paths themselves
    /// - `Some(n)` → recurse up to n levels deep
    ///
    /// Useful for limiting scan time on deep directory structures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,

    /// Whether to follow symbolic links during scanning.
    ///
    /// - `false` (recommended) → don't follow symlinks (avoids cycles)
    /// - `true` → follow symlinks (may cause infinite loops)
    pub follow_symlinks: bool,

    /// Whether to include submodule repositories in results.
    ///
    /// - `true` → report submodules as separate projects
    /// - `false` → skip submodules (only report parent repositories)
    pub include_submodules: bool,
}

impl Default for ScanConfig {
    /// Creates a default scan configuration.
    ///
    /// Scans the user's home directory with reasonable defaults:
    /// - Max depth: 3 levels
    /// - Don't follow symlinks
    /// - Include submodules
    fn default() -> Self {
        Self {
            root_paths: vec![dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))],
            max_depth: Some(3),
            follow_symlinks: false,
            include_submodules: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_project_serialization() {
        let project = GitProject {
            name: "test-repo".to_string(),
            path: PathBuf::from("/home/user/test-repo"),
            remotes: vec![],
            config: None,
            is_submodule: false,
            has_submodules: false,
            last_scanned: Utc::now(),
        };

        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("test-repo"));

        let deserialized: GitProject = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, project.name);
    }

    #[test]
    fn test_remote_url_serialization() {
        let remote = RemoteUrl {
            name: "origin".to_string(),
            url: "https://github.com/user/repo.git".to_string(),
            service: Some("github".to_string()),
            account: Some("user".to_string()),
        };

        let json = serde_json::to_string(&remote).unwrap();
        let deserialized: RemoteUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, remote);
    }

    #[test]
    fn test_config_scope_serialization() {
        let scope = ConfigScope::Local;
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(json, "\"local\"");

        let scope = ConfigScope::Global;
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(json, "\"global\"");
    }

    #[test]
    fn test_scan_config_default() {
        let config = ScanConfig::default();
        assert_eq!(config.max_depth, Some(3));
        assert!(!config.follow_symlinks);
        assert!(config.include_submodules);
        assert!(!config.root_paths.is_empty());
    }
}
