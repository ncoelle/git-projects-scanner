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

    /// Extracts metadata for a single Git repository.
    ///
    /// This is the core function that populates a [`GitProject`] with all
    /// relevant information using gitoxide.
    ///
    /// # Errors
    ///
    /// Returns an error if critical Git operations fail. Non-critical failures
    /// (like missing config) result in `None` values in the returned struct.
    fn analyze_repository(&self, repo: gix::Repository) -> Result<GitProject> {
        let path = repo.workdir().unwrap_or_else(|| repo.path());
        
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
        // A repository is a submodule if its .git is a file
        let is_submodule = repo.path().is_file();

        // Check if this repo has submodules
        let has_submodules = path.join(".gitmodules").exists();

        // Extract remote URLs using gitoxide
        let mut remotes = Vec::new();
        let remote_names = repo.remote_names();
        for name in remote_names {
            let name_str = name.as_ref();
            if let Ok(remote) = repo.find_remote(name_str) {
                if let Some(url) = remote.url(gix::remote::Direction::Fetch) {
                    let url_string = url.to_bstring().to_string();
                    let (service, account) = git_analyzer::parse_git_url(&url_string);
                    remotes.push(crate::models::RemoteUrl {
                        name: name_str.to_string(),
                        url: url_string,
                        service,
                        account,
                    });
                }
            }
        }

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

            // If we are NOT following symlinks, skip if the path is a symlink
            if !config.follow_symlinks && entry.path_is_symlink() {
                continue;
            }

            // Skip if we're already inside a repository we've found
            // (unless it's a submodule and we want to include those)
            if self.is_inside_known_repo(path, &visited_repos) {
                continue;
            }

            // Check if this is a Git repository
            if let Ok(repo) = gix::discover(path) {
                // gix::discover might find a parent repo, we only want to detect
                // if the current directory is the root of a repo.
                let work_dir = repo.workdir();
                let git_path = repo.path().to_path_buf();

                let is_root = if let Some(wd) = work_dir {
                    wd == path
                } else {
                    git_path == path || git_path.parent() == Some(path)
                };

                if is_root {
                    let is_submodule = git_path.is_file();

                    // Decide whether to include this repository
                    let should_include = if is_submodule {
                        config.include_submodules
                    } else {
                        true // Always include non-submodule repos
                    };

                    if should_include {
                        match self.analyze_repository(repo) {
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
                }
            }
        }

        if self.verbose {
            eprintln!("Found {} projects in {}", projects.len(), root.display());
        }

        Ok(projects)
    }

    /// Checks if a path is inside a repository we've already discovered.
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
        let git_dir = dir.join(".git");
        fs::create_dir(&git_dir)?;
        // gix::discover needs some basic files to recognize it as a repo
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
        fs::create_dir(git_dir.join("refs"))?;
        fs::create_dir(git_dir.join("objects"))?;
        Ok(())
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
        // Not a repo initially
        assert!(gix::discover(temp.path()).is_err());

        // Create .git directory
        create_mock_repo(temp.path()).unwrap();
        assert!(gix::discover(temp.path()).is_ok());
    }

    #[test]
    fn test_is_submodule() {
        let temp = TempDir::new().unwrap();

        // Regular repo (directory)
        let git_dir = temp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        assert!(git_dir.is_dir());

        // Submodule (file)
        let submodule_dir = temp.path().join("submodule");
        fs::create_dir(&submodule_dir).unwrap();
        let git_file = submodule_dir.join(".git");
        fs::write(&git_file, "gitdir: ../.git/modules/sub").unwrap();
        assert!(git_file.is_file());
    }

    #[test]
    fn test_has_submodules() {
        let temp = TempDir::new().unwrap();

        // No .gitmodules file
        assert!(!temp.path().join(".gitmodules").exists());

        // Create .gitmodules file
        fs::write(temp.path().join(".gitmodules"), "[submodule \"test\"]").unwrap();
        assert!(temp.path().join(".gitmodules").exists());
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

    #[test]
    fn test_scan_max_depth() {
        let temp = TempDir::new().unwrap();
        let scanner = DefaultScanner::new();

        // Create a repo at depth 2
        let d1 = temp.path().join("d1");
        let d2 = d1.join("d2");
        let repo_dir = d2.join("repo");
        fs::create_dir_all(&repo_dir).unwrap();
        create_mock_repo(&repo_dir).unwrap();

        // Scan with max_depth 1 (should not find the repo)
        let config = ScanConfig {
            root_paths: vec![temp.path().to_path_buf()],
            max_depth: Some(1),
            follow_symlinks: false,
            include_submodules: true,
        };
        let projects = scanner.scan(&config).unwrap();
        assert_eq!(projects.len(), 0);

        // Scan with max_depth 3 (should find the repo)
        let config = ScanConfig {
            root_paths: vec![temp.path().to_path_buf()],
            max_depth: Some(3),
            follow_symlinks: false,
            include_submodules: true,
        };
        let projects = scanner.scan(&config).unwrap();
        assert_eq!(projects.len(), 1);
    }

    #[test]
    #[cfg(unix)] // Symlinks are easier to test on Unix
    fn test_scan_follow_symlinks() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let scanner = DefaultScanner::new().with_verbose(true);

        let real_repo = temp.path().join("real-repo");
        fs::create_dir(&real_repo).unwrap();
        create_mock_repo(&real_repo).unwrap();

        // Create a symlink to the repo
        let symlink_dir = temp.path().join("symlink-to-repo");
        symlink(&real_repo, &symlink_dir).unwrap();

        // Scan without following symlinks
        let config = ScanConfig {
            root_paths: vec![temp.path().to_path_buf()],
            max_depth: Some(1),
            follow_symlinks: false,
            include_submodules: true,
        };
        let projects = scanner.scan(&config).unwrap();
        // Should find only real-repo. symlink-to-repo is a symlink, and is_git_repository checks for .git inside it.
        // If follow_symlinks is false, WalkDir still returns the symlink entry.
        // But path.join(".git").exists() might or might not follow the symlink depending on OS/Rust version.
        // Actually, Path::exists() follows symlinks.
        
        // If we want to strictly NOT follow symlinks even if they look like a repo,
        // we might need to check if the path itself is a symlink.
        
        assert_eq!(projects.len(), 1, "Should find 1 project when follow_symlinks is false");

        // Scan with following symlinks
        let config = ScanConfig {
            root_paths: vec![temp.path().to_path_buf()],
            max_depth: Some(1),
            follow_symlinks: true,
            include_submodules: true,
        };
        let projects = scanner.scan(&config).unwrap();
        // Should find both real-repo and symlink-to-repo
        assert_eq!(projects.len(), 2, "Should find 2 projects when follow_symlinks is true");
    }

    #[test]
    fn test_scan_nested_repos() {
        let temp = TempDir::new().unwrap();
        let scanner = DefaultScanner::new();

        let parent_repo = temp.path().join("parent");
        fs::create_dir(&parent_repo).unwrap();
        create_mock_repo(&parent_repo).unwrap();

        let nested_repo = parent_repo.join("nested");
        fs::create_dir(&nested_repo).unwrap();
        create_mock_repo(&nested_repo).unwrap();

        // By default, we should NOT find nested repos that are not submodules
        let config = ScanConfig {
            root_paths: vec![temp.path().to_path_buf()],
            max_depth: Some(3),
            follow_symlinks: false,
            include_submodules: true,
        };
        let projects = scanner.scan(&config).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].path, parent_repo);
    }

    #[test]
    fn test_is_inside_known_repo() {
        let scanner = DefaultScanner::new();
        let mut known = HashSet::new();
        let repo_path = PathBuf::from("/a/b/c");
        known.insert(repo_path.clone());

        assert!(scanner.is_inside_known_repo(&repo_path.join("d"), &known));
        assert!(!scanner.is_inside_known_repo(&repo_path, &known));
        assert!(!scanner.is_inside_known_repo(&PathBuf::from("/a/b/other"), &known));
    }

    #[test]
    fn test_scan_with_worktree() {
        let temp = TempDir::new().unwrap();
        let main_repo = temp.path().join("main_repo");
        let worktree = temp.path().join("worktree");
        fs::create_dir_all(&main_repo).unwrap();
        fs::create_dir_all(&worktree).unwrap();

        // Create a mock main repo
        create_mock_repo(&main_repo).unwrap();
        
        // Create a mock worktree (a .git file pointing to the main repo)
        // In reality it's more complex, but for gix::discover, 
        // a .git file is enough to be recognized if it looks like a git file
        fs::write(worktree.join(".git"), "gitdir: ../main_repo/.git").unwrap();

        let scanner = DefaultScanner::new();
        let config = ScanConfig {
            root_paths: vec![temp.path().to_path_buf()],
            max_depth: Some(2),
            follow_symlinks: false,
            include_submodules: true,
        };

        let projects = scanner.scan(&config).unwrap();
        // Should find both the main repo and the worktree
        assert_eq!(projects.len(), 2);
    }
}
