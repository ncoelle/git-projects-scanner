### Project Guidelines

#### Build & Configuration
- **Rust Version**: This project requires Rust 1.70 or newer (due to `gitoxide` dependencies).
- **Tooling**: Uses standard Cargo workspaces.
- **Key Dependencies**:
  - `gix` (gitoxide): High-performance Git operations.
  - `fluent`: Localization (i18n) support.
  - `clap`: CLI argument parsing.
  - `anyhow`/`thiserror`: Error handling.

#### Testing Guidelines
- **Unit Tests**: Most logic is tested via `#[cfg(test)]` modules within the same file or in a `tests` module.
- **Mocking Repositories**: Since deep Git analysis requires real repositories, unit tests often mock simple repos by creating a `.git` directory using `tempfile`.
- **Running Tests**:
  ```bash
  cargo test --workspace
  ```
- **Example Test**:
  To test the scanner, you can create a temporary directory and a mock Git repo:
  ```rust
  use crate::{ScanConfig, DefaultScanner, ProjectScanner};
  use tempfile::TempDir;
  use std::fs;

  #[test]
  fn test_scanner_basic() {
      let temp = TempDir::new().unwrap();
      let repo_dir = temp.path().join("my-project");
      fs::create_dir(&repo_dir).unwrap();
      fs::create_dir(repo_dir.join(".git")).unwrap();

      let config = ScanConfig {
          root_paths: vec![temp.path().to_path_buf()],
          max_depth: Some(2),
          ..Default::default()
      };

      let scanner = DefaultScanner::new();
      let projects = scanner.scan(&config).unwrap();
      assert!(!projects.is_empty());
  }
  ```

#### Development & Code Style
- **Documentation**: Use `//!` for module-level and `///` for symbol-level documentation. Include `# Example` blocks in doc-comments whenever possible; these are verified via doc-tests.
- **Localization**:
  - All user-facing strings should be localized using the `Localizer`.
  - Translations are stored in `crates/git-projects-core/locales/{locale}/main.ftl`.
  - Use `clean_fluent_string` (found in `projects-cli.rs`) to remove Unicode isolation marks from Fluent output if needed for plain text display.
- **Error Handling**: 
  - Prefer `anyhow` for application logic (CLI) and `thiserror` for library errors (`git-projects-core`).
  - Wrap external errors (IO, Git) with context-rich custom errors defined in `error.rs`.
- **Git Operations**: Use `git_analyzer` for operations involving `gitoxide`. Avoid direct filesystem manipulation for Git-specific metadata if `gitoxide` can handle it.
