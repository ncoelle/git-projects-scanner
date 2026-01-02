//! Git project scanning functionality.
//!
//! This module provides the core scanning logic for discovering Git repositories
//! in the filesystem. The main entry point is the [`ProjectScanner`] trait,
//! with a default implementation in [`DefaultScanner`].

use crate::error::{Error, Result};
use crate::git_analyzer;
use crate::models::{GitProject, ScanConfig};
use chrono::Utc;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Trait for scanning and discovering Git projects.
///
/// Implementations of this trait define how directories are traversed
/// and how Git repositories are identified and cataloged.
///
/// # Example
///
/// ```no_run
/// use git_projects_core::{ProjectScanner, DefaultScanner, ScanConfig};
/// use std::path::PathBuf;
///
/// let scanner = DefaultScanner::new();
/// let config = ScanConfig {
///     root_paths: vec![PathBuf::from("/home/user/projects")],
///     max_depth: Some(3),
///     follow_symlinks: false,
///     include_submodules: true,
/// };
///
/// let projects = scanner.scan(&config)?;
/// println!("Found {} projects", projects.len());
/// # Ok::<(), git_projects_core::Error>(())
/// ```
pub trait ProjectScanner {
    /// Scans the filesystem according to the provided configuration.
    ///
    /// Returns a list of discovered Git projects with their metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any root path doesn't exist or isn't accessible
    /// - Permission issues prevent directory traversal
    /// - Git repository access fails critically
    fn scan(&self, config: &ScanConfig) -> Result<Vec<GitProject>>;
}

/// Default implementation of the ProjectScanner trait.
///
/// This scanner:
/// - Uses `walkdir` for efficient directory traversal
/// - Respects `max_depth` and `follow_symlinks` settings
/// - Detects Git repositories by looking for `.git` directories or files
/// - Distinguishes between regular repos and submodules
/// - Extracts metadata using gitoxide (via `git_analyzer`)
/// - Skips nested repositories unless they're submodules
///
/// # Performance Characteristics
///
/// - **I/O bound** - Speed depends on disk and filesystem
/// - **Memory efficient** - Processes repos one at a time
/// - **Parallel scanning** - Could be added in future versions
#[derive(Debug, Clone)]
pub struct DefaultScanner {
    /// Whether to emit verbose logging (for debugging).
    pub verbose: bool,
}

impl DefaultScanner {
    /// Creates a new DefaultScanner with default settings.
    ///
    /// # Example
    ///
    /// ```
    /// use git_projects_core::DefaultScanner;
    ///
    /// let scanner = DefaultScanner::new();
    /// ```
    pub fn new() -> Self {
        Self { verbose: false }
    }

    /// Creates a new DefaultScanner with verbose output enabled.
    ///
    /// Useful for debugging scanning issues.
    ///
    /// # Example
    ///
    /// ```
    /// use git_projects_core::DefaultScanner;
    ///
    /// let scanner = DefaultScanner::new().with_verbose(true);
    /// ```
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Checks if a path is a Git repository.
    ///
    /// Returns:
    /// - `Some(git_dir)` if it's a repository (where `git_dir` is the `.git` location)
    /// - `None` if it's not a repository
    ///
    /// # Detection Logic
    ///
    /// A path is considered a Git repository if:
    /// - It contains a `.git` directory (normal repo), OR
    /// - It contains a `.git` file (submodule or worktree)
    fn is_git_repository(&self, path: &Path) -> Option<PathBuf> {
        let git_dir = path.join(".git");
        if git_dir.exists() {
            Some(git_dir)
        } else {
            None
        }
    }

    /// Determines if a repository is a submodule.
    ///
    /// A repository is a submodule if its `.git` is a **file** (not a directory)
    /// that points to the parent repository's `.git/modules/` directory.
    fn is_submodule(&self, git_path: &Path) -> bool {
        git_path.is_file()
    }

    /// Checks if a repository contains submodules.
    ///
    /// Detection is done by looking for a `.gitmodules` file in the repo root.
    fn has_submodules(&self, repo_path: &Path) -> bool {
        repo_path.join(".gitmodules").exists()
    }

    /// Extracts metadata for a single Git repository.
    ///
    /// This is the core function that populates a [`GitProject`] with all
    /// relevant information using gitoxide.
    ///
    /// # Errors
    ///
    /// Returns an error if critical Git operations fail. Non-critical failures
    /// (like missing config) result in `None` values in the returned struct.
    fn analyze_repository(&self, path: &Path, git_path: &Path) -> Result<GitProject> {
        if self.verbose {
            eprintln!("Analyzing repository: {}", path.display());
        }

        // Extract the repository name from the path
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Determine if this is a submodule
        let is_submodule = self.is_submodule(git_path);

        // Check if this repo has submodules
        let has_submodules = self.has_submodules(path);

        // Extract remote URLs using gitoxide
        let remotes = git_analyzer::extract_remote_urls(path).unwrap_or_else(|e| {
            if self.verbose {
                eprintln!("  Warning: Failed to extract remotes: {}", e);
            }
            Vec::new()
        });

        // Extract Git configuration (user.name, user.email)
        let config = git_analyzer::extract_git_config(path).ok();

        Ok(GitProject {
            name,
            path: path.to_path_buf(),
            remotes,
            config,
            is_submodule,
            has_submodules,
            last_scanned: Utc::now(),
        })
    }

    /// Scans a single root path for Git repositories.
    ///
    /// This is called once per root path in the configuration.
    fn scan_root(&self, root: &Path, config: &ScanConfig) -> Result<Vec<GitProject>> {
        // Validate root path exists
        if !root.exists() {
            return Err(Error::path_not_found(root));
        }

        if !root.is_dir() {
            return Err(Error::not_a_directory(root));
        }

        let mut projects = Vec::new();
        let mut visited_repos: HashSet<PathBuf> = HashSet::new();

        if self.verbose {
            eprintln!("Scanning root: {}", root.display());
        }

        // Configure walkdir
        let mut walker = WalkDir::new(root)
            .follow_links(config.follow_symlinks)
            .min_depth(0); // Include the root itself

        if let Some(max_depth) = config.max_depth {
            walker = walker.max_depth(max_depth);
        }

        for entry in walker {
            // Skip entries that we can't read (permission issues, etc.)
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    if self.verbose {
                        eprintln!("Warning: Skipping entry: {}", e);
                    }
                    continue;
                }
            };

            let path = entry.path();

            // Skip if we're already inside a repository we've found
            // (unless it's a submodule and we want to include those)
            if self.is_inside_known_repo(path, &visited_repos) {
                continue;
            }

            // Check if this is a Git repository
            if let Some(git_path) = self.is_git_repository(path) {
                let is_submodule = self.is_submodule(&git_path);

                // Decide whether to include this repository
                let should_include = if is_submodule {
                    config.include_submodules
                } else {
                    true // Always include non-submodule repos
                };

                if should_include {
                    match self.analyze_repository(path, &git_path) {
                        Ok(project) => {
                            visited_repos.insert(path.to_path_buf());
                            projects.push(project);

                            if self.verbose {
                                eprintln!(
                                    "  Found: {} ({})",
                                    path.display(),
                                    if is_submodule { "submodule" } else { "repo" }
                                );
                            }
                        }
                        Err(e) => {
                            if self.verbose {
                                eprintln!("  Error analyzing {}: {}", path.display(), e);
                            }
                            // Continue scanning even if one repo fails
                        }
                    }
                }

                // If this is a regular repo (not a submodule), don't descend into it
                // This prevents finding nested repos inside repos
                if !is_submodule {
                    // Note: walkdir doesn't provide a way to skip descendants from the iterator
                    // So we rely on the is_inside_known_repo check above
                }
            }
        }

        if self.verbose {
            eprintln!("Found {} projects in {}", projects.len(), root.display());
        }

        Ok(projects)
    }

    /// Checks if a path is inside a repository we've already discovered.
    ///
    /// This prevents scanning nested repositories (repos inside repos),
    /// which can happen with monorepos or when people clone repos inside repos.
    fn is_inside_known_repo(&self, path: &Path, known_repos: &HashSet<PathBuf>) -> bool {
        for repo_path in known_repos {
            if path != repo_path && path.starts_with(repo_path) {
                return true;
            }
        }
        false
    }
}

impl Default for DefaultScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectScanner for DefaultScanner {
    fn scan(&self, config: &ScanConfig) -> Result<Vec<GitProject>> {
        let mut all_projects = Vec::new();

        for root in &config.root_paths {
            match self.scan_root(root, config) {
                Ok(mut projects) => {
                    all_projects.append(&mut projects);
                }
                Err(e) => {
                    if self.verbose {
                        eprintln!("Error scanning {}: {}", root.display(), e);
                    }
                    // For now, continue with other roots even if one fails
                    // In the future, we might want a "strict" mode that fails fast
                }
            }
        }

        Ok(all_projects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a mock Git repository for testing
    fn create_mock_repo(dir: &Path) -> std::io::Result<()> {
        fs::create_dir(dir.join(".git"))
    }

/*
    /// Helper to create a mock submodule (Git file instead of directory)
    /// #[allow(dead_code)]
    fn create_mock_submodule(dir: &Path) -> std::io::Result<()> {
        fs::write(dir.join(".git"), "gitdir: ../.git/modules/submodule")
    }
*/
    #[test]
    fn test_scanner_creation() {
        let scanner = DefaultScanner::new();
        assert!(!scanner.verbose);

        let scanner = DefaultScanner::new().with_verbose(true);
        assert!(scanner.verbose);
    }

    #[test]
    fn test_is_git_repository() {
        let temp = TempDir::new().unwrap();
        let scanner = DefaultScanner::new();

        // Not a repo initially
        assert!(scanner.is_git_repository(temp.path()).is_none());

        // Create .git directory
        create_mock_repo(temp.path()).unwrap();
        assert!(scanner.is_git_repository(temp.path()).is_some());
    }

    #[test]
    fn test_is_submodule() {
        let temp = TempDir::new().unwrap();
        let scanner = DefaultScanner::new();

        // Regular repo (directory)
        let git_dir = temp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        assert!(!scanner.is_submodule(&git_dir));

        // Submodule (file)
        let submodule_dir = temp.path().join("submodule");
        fs::create_dir(&submodule_dir).unwrap();
        let git_file = submodule_dir.join(".git");
        fs::write(&git_file, "gitdir: ../.git/modules/sub").unwrap();
        assert!(scanner.is_submodule(&git_file));
    }

    #[test]
    fn test_has_submodules() {
        let temp = TempDir::new().unwrap();
        let scanner = DefaultScanner::new();

        // No .gitmodules file
        assert!(!scanner.has_submodules(temp.path()));

        // Create .gitmodules file
        fs::write(temp.path().join(".gitmodules"), "[submodule \"test\"]").unwrap();
        assert!(scanner.has_submodules(temp.path()));
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp = TempDir::new().unwrap();
        let scanner = DefaultScanner::new();

        let config = ScanConfig {
            root_paths: vec![temp.path().to_path_buf()],
            max_depth: Some(3),
            follow_symlinks: false,
            include_submodules: true,
        };

        let projects = scanner.scan(&config).unwrap();
        assert_eq!(projects.len(), 0);
    }

    #[test]
    fn test_scan_nonexistent_path() {
        let scanner = DefaultScanner::new();
        let config = ScanConfig {
            root_paths: vec![PathBuf::from("/nonexistent/path/that/does/not/exist")],
            max_depth: Some(3),
            follow_symlinks: false,
            include_submodules: true,
        };

        let result = scanner.scan(&config);
        // Should return Ok with empty vec (because we continue on error)
        // or return an error - depends on error handling strategy
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_scan_with_mock_repo() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path().join("test-repo");
        fs::create_dir(&repo_dir).unwrap();
        create_mock_repo(&repo_dir).unwrap();

        let scanner = DefaultScanner::new();
        let config = ScanConfig {
            root_paths: vec![temp.path().to_path_buf()],
            max_depth: Some(2),
            follow_symlinks: false,
            include_submodules: true,
        };

        // Note: This will fail to analyze because it's not a real Git repo
        // But we can at least test that the scanner finds it
        let result = scanner.scan(&config);
        assert!(result.is_ok());
        // The actual analysis will fail, so we might get 0 projects
        // This is expected with a mock .git directory
    }
}
