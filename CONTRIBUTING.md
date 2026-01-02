# Contributing to git-projects-scanner

Thank you for your interest in contributing to git-projects-scanner! This document provides guidelines and instructions for contributors.

## Code of Conduct

Be respectful, inclusive, and constructive. We're building a welcoming community.

---

## Getting Started

### Prerequisites

- Rust 1.56+ (check with `rustc --version`)
- Cargo (comes with Rust)
- Git
- For macOS development: Xcode Command Line Tools

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/[your-username]/git-projects-scanner.git
cd git-projects-scanner

# Verify Rust installation
rustc --version
cargo --version

# Build the project
cargo build

# Run tests
cargo test --all

# Run the CLI
cargo run --bin projects-cli -- --help
```

### Using RustRover (Recommended)

1. Open project in JetBrains RustRover
2. Install Claude plugin (if developing with AI assistance)
3. Right-click project → "Add to Claude Project" (if supported)

---

## Development Workflow

### Branch Naming

```
feature/description          # New feature
fix/issue-description        # Bug fix
docs/what-changed            # Documentation
refactor/what-changed        # Code refactoring
test/what-tested            # Test additions
```

Example:
```bash
git checkout -b feature/json-output-format
git checkout -b fix/remote-url-parsing
git checkout -b docs/api-schema
```

### Making Changes

1. Create a feature branch from `main`
2. Make your changes
3. Test locally (`cargo test --all`)
4. Format code (`cargo fmt`)
5. Run linter (`cargo clippy -- -D warnings`)
6. Commit with clear messages
7. Push to your fork
8. Create a Pull Request

### Code Style

#### Formatting

Run before committing:
```bash
cargo fmt
```

This ensures consistent formatting across the codebase.

#### Linting

Run before committing:
```bash
cargo clippy -- -D warnings
```

Fix all warnings and errors. This keeps code quality high.

#### Naming Conventions

Follow Rust conventions:
- **Functions:** `snake_case` – `parse_remote_url()`
- **Types:** `PascalCase` – `struct GitProject`, `enum ConfigScope`
- **Constants:** `SCREAMING_SNAKE_CASE` – `const MAX_DEPTH: u32 = 10`
- **Modules:** `snake_case` – `mod git_analyzer`
- **Visibility:** Use `pub` only what's necessary (prefer private)

#### Comments

Add comments for "why", not "what":

```rust
// Good: explains the decision
// Skip scanning inside .git to avoid traversing repository internals
if entry.file_name() == ".git" {
    continue;
}

// Avoid: just restates the code
// Skip .git directory
if entry.file_name() == ".git" {
    continue;
}
```

#### Documentation Comments

Use doc comments for public APIs:

```rust
/// Scans the given directory for git projects.
///
/// # Arguments
/// * `config` - Scan configuration including root path and locale
///
/// # Returns
/// A vector of discovered `GitProject` instances
///
/// # Errors
/// Returns an error if the root path is invalid or inaccessible
pub fn scan(config: &ScanConfig) -> Result<Vec<GitProject>> {
    // implementation
}
```

---

## Testing

### Running Tests

```bash
# All tests
cargo test --all

# Library tests only
cargo test --lib

# Binary tests only
cargo test --bin projects-cli

# With output (for debugging)
cargo test -- --nocapture

# Single test
cargo test test_parse_remote_url

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

### Writing Tests

Place unit tests in the same file as code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_remote_url_https_github() {
        let url = "https://github.com/champion/nbt.git";
        let (host, account) = parse_remote_url(url).unwrap();
        
        assert_eq!(host, "github.com");
        assert_eq!(account, Some("champion"));
    }

    #[test]
    fn test_parse_remote_url_ssh() {
        let url = "git@github.com:champion/nbt.git";
        let (host, account) = parse_remote_url(url).unwrap();
        
        assert_eq!(host, "github.com");
        assert_eq!(account, Some("champion"));
    }
}
```

### Test Coverage Expectations

- Aim for >80% code coverage
- Test public APIs thoroughly
- Include both happy path and error cases
- Test edge cases (empty strings, None values, etc.)

---

## Commit Messages

Follow Conventional Commits format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code refactoring without feature changes
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, etc.

### Scope

Optional, but helpful:
- `scanner` – ProjectScanner changes
- `cli` – CLI binary changes
- `l10n` – Localization changes
- `models` – Data structure changes
- `git-analyzer` – Git operations changes

### Examples

```
feat(scanner): add nested repository detection

- Recursively scan subdirectories for .git
- Avoid scanning inside .git directories
- Closes #42

fix(git-analyzer): handle SSH URLs without username

This handles git@host:path URLs where no username is present.

docs(api): update JSON schema for v0.1.0

test(scanner): add tests for nested repos
```

---

## Pull Requests

### Before Creating a PR

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Run `cargo test --all` (all tests must pass)
6. Update relevant documentation

### PR Template

When creating a PR, include:

```markdown
## Description
Brief description of the changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Related Issues
Closes #123

## Testing
Describe how you tested the changes.

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No new warnings from clippy
- [ ] Commits follow conventional commits format
```

### Review Process

- One approval required before merging
- Automated checks (CI/CD) must pass
- Address review comments promptly
- Be respectful of reviewer feedback

---

## Documentation

### When to Document

Update docs when:
- Adding new features
- Changing public APIs
- Fixing subtle bugs
- Adding complex logic

### Documentation Files

- **DESIGN.md** – Architecture and design decisions
- **OPEN_DECISIONS.md** – Deferred decisions and future features
- **docs/ADR/*** – Architecture Decision Records
- **README.md** – Project overview and quick start
- **Code comments** – Why decisions were made (not what code does)

### Writing Documentation

- Use clear, concise English
- Include examples where helpful
- Link to related sections
- Keep docs in sync with code

---

## Reporting Issues

### Bug Reports

Include:
- Clear description of the bug
- Steps to reproduce
- Expected vs. actual behavior
- Rust version (`rustc --version`)
- OS and version
- Relevant logs or error messages

### Feature Requests

Include:
- Clear description of the feature
- Use case / motivation
- Possible implementation approach (optional)
- Related issues or discussions

### Discussions

Use GitHub Discussions for:
- Questions about usage
- Ideas before opening an issue
- General feedback

---

## Architecture & Design Decisions

Before making major changes:

1. Read [DESIGN.md](docs/DESIGN.md) – Understand the architecture
2. Check [OPEN_DECISIONS.md](docs/OPEN_DECISIONS.md) – See what's deferred
3. Review [docs/ADR/](docs/ADR/) – See past decisions
4. Consider creating an ADR for significant changes

### Creating an Architecture Decision Record (ADR)

For significant architectural changes:

1. Read [docs/ADR/README.md](docs/ADR/README.md)
2. Create a new file: `docs/ADR/NNNN-slug.md`
3. Follow the ADR template
4. Include in PR description
5. Link from OPEN_DECISIONS.md if appropriate

---

## Localization (L10N) Changes

### Adding New Strings

When adding new user-facing strings:

1. Add to `crates/git-projects-core/locales/en/main.ftl`
2. Add to `crates/git-projects-core/locales/de/main.ftl`
3. Reference in code using L10N API
4. Test both English and German

### Translation Process

Current translations: English, German (by maintainers)

Future: Community contributions welcome. We use:
- [Weblate](https://weblate.org/) (for community translations)
- Manual review before merging

---

## Performance Considerations

### When Optimizing

1. Measure first (use benchmarks)
2. Avoid premature optimization
3. Document performance changes
4. Test on different platforms (macOS, Linux, Windows)

### Current Performance Goals (MVP)

- <100 projects: <1 second
- 100-500 projects: 1-5 seconds
- Don't optimize for >500 projects yet (caching deferred)

---

## Release Process

(Maintained by project leads)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` (if created)
3. Create git tag: `git tag v0.1.0`
4. Push tag: `git push origin v0.1.0`
5. GitHub Actions builds and publishes

---

## Getting Help

- **Questions:** Open a Discussion on GitHub
- **Bug reports:** Open an Issue
- **Ideas:** Discuss in Issues or Discussions
- **Chat:** See README for community channels

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Cargo Guide](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Conventional Commits](https://www.conventionalcommits.org/)

---

## License

By contributing, you agree that your contributions are licensed under the MIT License (same as the project).

---

**Thank you for contributing! 🎉**

Questions? Open an issue or discussion. We're here to help!

**Last Updated:** January 2025

