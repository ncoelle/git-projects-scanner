//! Command-line interface for Git Projects Scanner.
//!
//! This binary provides a user-friendly CLI for scanning and cataloging
//! Git repositories on the local filesystem.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use git_projects_core::{
    l10n::Localizer,
    ConfigScope, DefaultScanner, GitProject, ProjectScanner, ScanConfig,
};
use std::path::PathBuf;

/// Git Projects Scanner - Catalog your local Git repositories
#[derive(Parser, Debug)]
#[command(
    name = "projects-cli",
    version,
    about = "Scan and catalog Git repositories on your local filesystem",
    long_about = None
)]
struct Cli {
    /// Root directories to scan (can be specified multiple times)
    #[arg(
        short = 'r',
        long = "root",
        value_name = "PATH",
        help = "Root directory to scan"
    )]
    roots: Vec<PathBuf>,

    /// Maximum depth to recurse into subdirectories
    #[arg(
        short = 'd',
        long = "depth",
        value_name = "N",
        help = "Maximum recursion depth (default: 3)"
    )]
    max_depth: Option<usize>,

    /// Don't follow symbolic links during scanning
    #[arg(long = "no-symlinks", help = "Don't follow symbolic links")]
    no_symlinks: bool,

    /// Don't include submodule repositories in results
    #[arg(long = "no-submodules", help = "Don't include submodule repositories")]
    no_submodules: bool,

    /// Sorting profile for results
    #[arg(
        short = 's',
        long = "sort",
        value_enum,
        default_value_t = SortProfile::Name,
        help = "Sort results by: name, path, recent, or service"
    )]
    sort: SortProfile,

    /// Output as JSON instead of a table
    #[arg(short = 'j', long = "json", help = "Output as JSON")]
    json: bool,

    /// Show detailed scanning progress
    #[arg(short = 'v', long = "verbose", help = "Show verbose output")]
    verbose: bool,

    /// Locale for messages (e.g., en, de)
    #[arg(
        short = 'l',
        long = "locale",
        value_name = "LOCALE",
        help = "Locale for messages (e.g., en, de)"
    )]
    locale: Option<String>,
}

/// Sorting profiles for organizing results
#[derive(Debug, Clone, Copy, ValueEnum)]
enum SortProfile {
    /// Sort alphabetically by repository name
    Name,
    /// Sort alphabetically by full path
    Path,
    /// Sort by last scanned time (newest first)
    Recent,
    /// Group by hosting service (GitHub, GitLab, etc.)
    Service,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize localizer
    let localizer = if let Some(locale) = &cli.locale {
        Localizer::new(locale).with_context(|| format!("Failed to load locale: {}", locale))?
    } else {
        Localizer::from_system()
            .unwrap_or_else(|_| Localizer::new("en").expect("Failed to load default locale"))
    };

    // Build scan configuration
    let config = build_scan_config(&cli)?;

    // Create scanner
    let scanner = DefaultScanner::new().with_verbose(cli.verbose);

    // Show start message
    if !cli.json && cli.verbose {
        eprintln!(
            "{}",
            clean_fluent_string(&localizer.get("scan-started", None))
        );
        for root in &config.root_paths {
            let path_str = root.display().to_string();
            eprintln!(
                "{}",
                clean_fluent_string(
                    &localizer.get("scan-started-path", Some(&[("path", path_str.as_str())]))
                )
            );
        }
    }

    // Perform the scan
    let mut projects = scanner
        .scan(&config)
        .context("Failed to scan for Git repositories")?;

    // Sort the results
    sort_projects(&mut projects, cli.sort);

    // Show completion message
    if !cli.json && cli.verbose {
        let count = projects.len().to_string();
        eprintln!(
            "{}",
            clean_fluent_string(&localizer.get("scan-complete", Some(&[("count", &count)])))
        );
    }

    // Output results
    if cli.json {
        output_json(&projects)?;
    } else {
        output_table(&projects, &localizer)?;
    }

    Ok(())
}

/// Builds a ScanConfig from CLI arguments
fn build_scan_config(cli: &Cli) -> Result<ScanConfig> {
    // Determine root paths
    let root_paths = if cli.roots.is_empty() {
        // Default to home directory if no roots specified
        vec![dirs::home_dir().context("Could not determine home directory")?]
    } else {
        cli.roots.clone()
    };

    // Validate that all root paths exist
    for path in &root_paths {
        if !path.exists() {
            anyhow::bail!("Path does not exist: {}", path.display());
        }
        if !path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", path.display());
        }
    }

    Ok(ScanConfig {
        root_paths,
        max_depth: cli.max_depth.or(Some(3)), // Default to 3 if not specified
        follow_symlinks: !cli.no_symlinks,
        include_submodules: !cli.no_submodules,
    })
}

/// Sorts projects according to the specified profile
fn sort_projects(projects: &mut [GitProject], profile: SortProfile) {
    match profile {
        SortProfile::Name => {
            projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }
        SortProfile::Path => {
            projects.sort_by(|a, b| a.path.cmp(&b.path));
        }
        SortProfile::Recent => {
            // Sort by last_scanned, newest first
            projects.sort_by(|a, b| b.last_scanned.cmp(&a.last_scanned));
        }
        SortProfile::Service => {
            // Sort by service, then by account, then by name
            projects.sort_by(|a, b| {
                let a_service = a
                    .remotes
                    .first()
                    .and_then(|r| r.service.as_deref())
                    .unwrap_or("");
                let b_service = b
                    .remotes
                    .first()
                    .and_then(|r| r.service.as_deref())
                    .unwrap_or("");

                match a_service.cmp(b_service) {
                    std::cmp::Ordering::Equal => {
                        let a_account = a
                            .remotes
                            .first()
                            .and_then(|r| r.account.as_deref())
                            .unwrap_or("");
                        let b_account = b
                            .remotes
                            .first()
                            .and_then(|r| r.account.as_deref())
                            .unwrap_or("");

                        match a_account.cmp(b_account) {
                            std::cmp::Ordering::Equal => a.name.cmp(&b.name),
                            other => other,
                        }
                    }
                    other => other,
                }
            });
        }
    }
}

/// Outputs projects as JSON to stdout
fn output_json(projects: &[GitProject]) -> Result<()> {
    let json =
        serde_json::to_string_pretty(projects).context("Failed to serialize projects to JSON")?;
    println!("{}", json);
    Ok(())
}

/// Outputs projects as a formatted table to stdout
fn output_table(projects: &[GitProject], localizer: &Localizer) -> Result<()> {
    if projects.is_empty() {
        println!(
            "{}",
            clean_fluent_string(&localizer.get("scan-no-results", None))
        );
        return Ok(());
    }

    // Calculate column widths
    let name_width = projects
        .iter()
        .map(|p| p.name.len())
        .max()
        .unwrap_or(10)
        .max(localizer.get("header-name", None).len());

    let path_width = projects
        .iter()
        .map(|p| p.path.display().to_string().len())
        .max()
        .unwrap_or(20)
        .max(localizer.get("header-path", None).len())
        .min(60); // Cap at 60 chars for readability

    let remote_width = 30;
    let config_width = 35;

    // Print header
    println!(
        "{:<name_width$}  {:<path_width$}  {:<remote_width$}  {:<config_width$}  {}  {}",
        localizer.get("header-name", None),
        localizer.get("header-path", None),
        localizer.get("header-remotes", None),
        localizer.get("header-config", None),
        localizer.get("header-submodule", None),
        localizer.get("header-has-submodules", None),
        name_width = name_width,
        path_width = path_width,
        remote_width = remote_width,
        config_width = config_width,
    );

    // Print separator
    println!(
        "{}",
        "=".repeat(name_width + path_width + remote_width + config_width + 20)
    );

    // Print each project
    for project in projects {
        let name = truncate(&project.name, name_width);
        let path = truncate(&project.path.display().to_string(), path_width);
        let remote = format_remotes(project, localizer);
        let config = format_config(project, localizer);
        let is_submodule = if project.is_submodule {
            localizer.get("submodule-yes", None)
        } else {
            localizer.get("submodule-no", None)
        };
        let has_submodules = if project.has_submodules {
            localizer.get("submodule-yes", None)
        } else {
            localizer.get("submodule-no", None)
        };

        println!(
            "{:<name_width$}  {:<path_width$}  {:<remote_width$}  {:<config_width$}  {:<3}  {}",
            name,
            path,
            truncate(&remote, remote_width),
            truncate(&config, config_width),
            is_submodule,
            has_submodules,
            name_width = name_width,
            path_width = path_width,
            remote_width = remote_width,
            config_width = config_width,
        );
    }

    // Print summary
    println!();
    let count = projects.len().to_string();
    println!(
        "{}",
        clean_fluent_string(&localizer.get("scan-complete", Some(&[("count", &count)])))
    );
    Ok(())
}

/// Formats remote information for display
fn format_remotes(project: &GitProject, localizer: &Localizer) -> String {
    if project.remotes.is_empty() {
        return clean_fluent_string(&localizer.get("remote-none", None));
    }

    let first = &project.remotes[0];
    let mut result = String::new();

    // Add service if available
    if let Some(service) = &first.service {
        result.push_str(service);
        if let Some(account) = &first.account {
            result.push('/');
            result.push_str(account);
        }
    } else {
        // Fallback to remote name
        result.push_str(&first.name);
    }

    // Add count if multiple remotes
    if project.remotes.len() > 1 {
        let count = project.remotes.len().to_string();
        let remote_count =
            clean_fluent_string(&localizer.get("remote-count", Some(&[("count", &count)])));
        result.push_str(&format!(" (+{})", remote_count));
    }

    result
}

/// Formats Git config for display
fn format_config(project: &GitProject, localizer: &Localizer) -> String {
    match &project.config {
        Some(config) => {
            let scope = match config.scope {
                ConfigScope::Local => clean_fluent_string(&localizer.get("config-local", None)),
                ConfigScope::Global => clean_fluent_string(&localizer.get("config-global", None)),
                ConfigScope::System => clean_fluent_string(&localizer.get("config-system", None)),
            };

            match (&config.user_name, &config.user_email) {
                (Some(name), Some(email)) => {
                    format!("{} <{}> [{}]", name, email, scope)
                }
                (Some(name), None) => {
                    format!("{} [{}]", name, scope)
                }
                (None, Some(email)) => {
                    format!("<{}> [{}]", email, scope)
                }
                (None, None) => {
                    format!("[{}]", scope)
                }
            }
        }
        None => clean_fluent_string(&localizer.get("config-none", None)),
    }
}

/// Removes Unicode control characters that Fluent might add
fn clean_fluent_string(s: &str) -> String {
    s.chars()
        .filter(|c| {
            !matches!(
                *c,
                '\u{2068}' |  // FIRST STRONG ISOLATE
            '\u{2069}' |  // POP DIRECTIONAL ISOLATE
            '\u{202A}' |  // LEFT-TO-RIGHT EMBEDDING
            '\u{202B}' |  // RIGHT-TO-LEFT EMBEDDING
            '\u{202C}' |  // POP DIRECTIONAL FORMATTING
            '\u{202D}' |  // LEFT-TO-RIGHT OVERRIDE
            '\u{202E}' // RIGHT-TO-LEFT OVERRIDE
            )
        })
        .collect()
}

/// Truncates a string to a maximum width, adding "..." if truncated
/// Unicode-safe version that respects character boundaries
fn truncate(s: &str, max_width: usize) -> String {
    let char_count = s.chars().count();

    if char_count <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        "...".to_string()
    } else {
        // Use char indices instead of byte indices
        s.chars().take(max_width - 3).collect::<String>() + "..."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 2), "hi");

        // Test with Unicode
        assert_eq!(truncate("café", 10), "café");
        assert_eq!(truncate("hello 世界", 10), "hello 世界");

        // truncate doesn't clean control characters - that's done by clean_fluent_string
        // Just test that it doesn't panic on them
        let result = truncate("test\u{2068}123\u{2069}", 10);
        assert_eq!(result.chars().count(), 9); // 4 + 3 + 2 = 9 chars total
    }

    #[test]
    fn test_clean_fluent_string() {
        // Test that control characters are removed
        assert_eq!(clean_fluent_string("test\u{2068}123\u{2069}"), "test123");
        assert_eq!(clean_fluent_string("hello"), "hello");
        assert_eq!(clean_fluent_string("\u{2068}wrapped\u{2069}"), "wrapped");
    }

    #[test]
    fn test_sort_by_name() {
        let mut projects = vec![
            create_test_project("zebra"),
            create_test_project("alpha"),
            create_test_project("beta"),
        ];

        sort_projects(&mut projects, SortProfile::Name);

        assert_eq!(projects[0].name, "alpha");
        assert_eq!(projects[1].name, "beta");
        assert_eq!(projects[2].name, "zebra");
    }

    #[test]
    fn test_sort_by_path() {
        let mut projects = vec![
            create_test_project_with_path("project", "/z/path"),
            create_test_project_with_path("project", "/a/path"),
            create_test_project_with_path("project", "/m/path"),
        ];

        sort_projects(&mut projects, SortProfile::Path);

        assert_eq!(projects[0].path, PathBuf::from("/a/path"));
        assert_eq!(projects[1].path, PathBuf::from("/m/path"));
        assert_eq!(projects[2].path, PathBuf::from("/z/path"));
    }

    fn create_test_project(name: &str) -> GitProject {
        GitProject {
            name: name.to_string(),
            path: PathBuf::from(format!("/test/{}", name)),
            remotes: vec![],
            config: None,
            is_submodule: false,
            has_submodules: false,
            last_scanned: Utc::now(),
        }
    }

    fn create_test_project_with_path(name: &str, path: &str) -> GitProject {
        GitProject {
            name: name.to_string(),
            path: PathBuf::from(path),
            remotes: vec![],
            config: None,
            is_submodule: false,
            has_submodules: false,
            last_scanned: Utc::now(),
        }
    }
}
