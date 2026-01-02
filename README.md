# git-projects-scanner

A cross-platform Rust library to scan Git projects across your filesystem and group them by account and platform.
Includes CLI with support for macOS, Linux, and Windows.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.56+-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/Status-MVP-blue.svg)](#status)

---

## Features

- **📁 Scan Local Directories** – Discover all git projects in `~/Projects` (or any directory)
- **👥 Multi-Account Support** – Track which GitHub/GitLab/etc. account owns each project
- **🌍 Multi-Platform Support** – Works with GitHub, GitLab, Gitea, Codeberg, self-hosted, and more
- **💻 Cross-Platform** – macOS, Linux, Windows support
- **🔍 Filter & Sort** – Filter by account or platform; sort by name, platform, account, modification time, or path
- **📊 Multiple Output Formats** – Human-readable tables or JSON (for GUIs)
- **🌐 Localization** – English and German supported (extensible for more)
- **📦 Library + CLI** – Use as a Rust library or command-line tool

---

## Quick Start

### Installation

Currently: Build from source (Homebrew/package manager coming later)

```bash
# Clone and build
git clone https://github.com/ncoelle/git-projects-scanner
cd git-projects-scanner

cargo install --path crates/git-projects-core

# Verify installation
projects --version
```

### Basic Usage

```bash
# List all projects
projects

# Filter by account
projects --account champion

# Filter by platform
projects --platform github.com

# Combine filters
projects --account champion --platform github.com

# Sort differently
projects --sort=platform
projects --sort=account --reverse

# JSON output (for scripting/GUIs)
projects --json

# Verbose mode (show git config)
projects --verbose

# Change language
projects --locale de
projects --locale en
```

### Example Output

```
Name        | Local Path          | Platform   | Account    | Remote URLs
────────────────────────────────────────────────────────────────────────
StreamAPI   | Java/StreamAPI      | localhost  | -          | (no remotes)
nbt         | Go/lo/nbt           | github.com | champion   | origin: https://github.com/champion/nbt
            |                     |            |            | upstream: https://github.com/original/nbt
```

---

## Use Cases

### For Developers

- **Project discovery** – "Where did I clone that project?"
- **Account management** – See which GitHub/GitLab account each project uses
- **Organization** – Understand your project landscape at a glance
- **Automation** – JSON output for scripting and tooling

### For Teams

- **Repository audit** – Catalog all team projects
- **Migration planning** – Identify projects that need platform/account updates
- **Onboarding** – New developers understand project structure

### For Tool Builders

- **Library for GUIs** – Build macOS/Linux/Windows apps on top of the core library
- **JSON API** – Integrate with other tools

---

## Status

**MVP (v0.1.0-alpha):** Core functionality is complete and tested.

### ✅ Implemented

- Core scanning library (Rust)
- CLI binary
- JSON output
- Filtering (account, platform)
- Sorting profiles
- Localization (English, German)
- Cross-platform support

### 🔄 In Progress

- None currently

### 📋 Planned (Post-MVP)

- Configuration file support (`.git-projects.toml`)
- Caching for performance
- GUI implementations:
  - macOS (Swift 6 + SwiftUI)
  - Linux (GTK or similar)
  - Windows (GTK or similar)
- Hierarchical sorting
- Additional metadata (git hooks, branch info, etc.)

See [OPEN_DECISIONS.md](docs/OPEN_DECISIONS.md) for the full roadmap.

---

## Architecture

### High-Level Overview

```
┌─────────────────────────────────┐
│  Your Tool / GUI / Script       │
└────────────┬────────────────────┘
             │
             ↓
┌─────────────────────────────────┐
│  CLI Binary (projects-cli)      │
│  or Library (git-projects-core) │
└────────────┬────────────────────┘
             │
             ↓
┌─────────────────────────────────┐
│  Rust Core Library              │
│  ├── Scanner                    │
│  ├── GitAnalyzer (gitoxide)     │
│  ├── Models (JSON-serializable) │
│  └── L10N (Fluent)              │
└─────────────────────────────────┘
```

### Key Components

- **Library (`git-projects-core`)** – Main scanning logic, Fluent L10N
- **CLI Binary (`projects-cli`)** – Command-line interface using the library
- **Models** – `GitProject`, `RemoteUrl`, `GitConfig` (all JSON-serializable)
- **Scanner** – Recursively scans directories, finds git projects

### Design Documents

- [DESIGN.md](docs/DESIGN.md) – Architecture, data model, design decisions
- [OPEN_DECISIONS.md](docs/OPEN_DECISIONS.md) – Features deferred to post-MVP
- [docs/ADR/](docs/ADR/) – Architecture Decision Records (why decisions were made)
- [docs/API_SCHEMA.json](docs/API_SCHEMA.json) – JSON output schema (for GUIs)

---

## Development

### Getting Started

```bash
# Clone
git clone https://github.com/ncoelle/git-projects-scanner
cd git-projects-scanner

# Build
cargo build

# Run tests
cargo test --all

# Run CLI
cargo run --bin projects-cli -- --help

# Format & lint
cargo fmt
cargo clippy -- -D warnings
```

### Requirements

- Rust 1.56+ (`rustc --version`)
- Cargo
- Git

### IDE Setup

**RustRover (Recommended):**
1. Open the project in RustRover
2. Optionally install Claude plugin for AI-assisted development
3. Build runs automatically

**VS Code:**
1. Install Rust Analyzer extension
2. Follow standard Rust development setup

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Code style guidelines
- Testing requirements
- Commit message format
- Pull request process
- How to report issues

---

## Using as a Library

### Add to Your Project

```toml
[dependencies]
git-projects-core = { path = "../git-projects-scanner/crates/git-projects-core" }
```

### Example Code

```rust
use git_projects_core::{ProjectScanner, ScanConfig};
use std::path::PathBuf;

fn main() -> Result<()> {
    let config = ScanConfig {
        root_path: PathBuf::from("/Users/you/Projects"),
        settings: Default::default(),
        locale: "en",
    };

    let scanner = ProjectScanner::default();
    let projects = scanner.scan(&config)?;

    for project in projects {
        println!("{}: {} ({}, {})",
            project.name,
            project.local_path.display(),
            project.platform_host,
            project.account.unwrap_or("-".to_string())
        );
    }

    Ok(())
}
```

### JSON Output

```rust
let projects = scanner.scan(&config)?;
let json = serde_json::to_string_pretty(&projects)?;
println!("{}", json);
```

See [docs/API_SCHEMA.json](docs/API_SCHEMA.json) for the complete JSON schema.

---

## Dependencies

Core dependencies (all MIT/Apache-2.0 compatible):

| Crate                  | Purpose                              |
|------------------------|--------------------------------------|
| `gitoxide`             | Pure Rust git operations (read-only) |
| `serde` + `serde_json` | JSON serialization                   |
| `fluent`               | Localization framework               |
| `clap`                 | CLI argument parsing                 |
| `anyhow`               | Error handling                       |
| `dirs`                 | Home directory detection             |

See [Cargo.toml](Cargo.toml) for the full list and versions.

---

## Examples

### List all projects with their git config

```bash
projects --verbose
```

### Export to JSON for processing

```bash
projects --json > projects.json
```

### Find projects by account

```bash
projects --account champion
```

### See projects on specific platforms

```bash
projects --platform github.com
```

### Sort by platform

```bash
projects --sort=platform
```

---

## Performance

### Current Performance (MVP)

- **<100 projects:** <1 second
- **100–500 projects:** 1–5 seconds
- **>500 projects:** Depends on filesystem

Each run performs a full scan (no caching in MVP).

### Future Improvements

Caching planned for post-MVP:
- SQLite-based cache with invalidation
- Optional FSEvents for macOS (watch-based updates)

See [OPEN_DECISIONS.md](docs/OPEN_DECISIONS.md#performance) for details.

---

## JSON Output Format

The `--json` flag outputs an array of projects matching this schema:

```json
[
  {
    "name": "nbt",
    "local_path": "Go/lo/nbt",
    "platform_host": "github.com",
    "account": "champion",
    "remote_urls": [
      {"name": "origin", "url": "https://github.com/champion/nbt.git"},
      {"name": "upstream", "url": "https://github.com/original/nbt.git"}
    ],
    "git_config": {
      "user_name": "Champion User",
      "user_email": "champion@example.com",
      "scope": "Local"
    },
    "contains_submodules": false,
    "is_valid_git_repo": true
  }
]
```

See [docs/API_SCHEMA.json](docs/API_SCHEMA.json) for the full schema.

---

## Localization

Supported languages:
- 🇬🇧 English (en)
- 🇩🇪 German (de)

### Contributing Translations

We welcome translation contributions. Currently using manual translation; community crowdsourcing planned.

To add a new language:
1. Create `crates/git-projects-core/locales/[lang]/main.ftl`
2. Copy structure from `en/main.ftl`
3. Translate strings
4. Test with `--locale [lang]`
5. Open a PR

---

## License

MIT License – See [LICENSE](LICENSE) file

**In short:** You can use this freely, commercially, or otherwise.
Must include a licence notice.

---

## Troubleshooting

### "Command not found: projects"

Make sure it's installed and in your PATH:
```bash
cargo install --path crates/git-projects-core
echo $PATH  # Verify cargo bin directory is included
```

### "Permission denied" errors

The scanner may lack read permissions. Check:
```bash
ls -la ~/Projects/.git/config
```

### Slow scanning

Large directories with many projects may be slow.
This is normal in MVP (no caching).

For troubleshooting, enable verbose output:
```bash
RUST_LOG=debug projects --verbose
```

### JSON output doesn't match a schema

Report an issue with:
- The output JSON
- Your Rust version (`rustc --version`)
- Your OS and version

---

## Support

- **Questions:** Open a [GitHub Discussion](https://github.com/[your-username]/git-projects-scanner/discussions)
- **Bug Reports:** Open a [GitHub Issue](https://github.com/[your-username]/git-projects-scanner/issues)
- **Ideas & Features:** Discuss in [Discussions](https://github.com/[your-username]/git-projects-scanner/discussions) first

---

## Roadmap

See [OPEN_DECISIONS.md](docs/OPEN_DECISIONS.md) for the complete roadmap.

### Near-term (Q1-Q2 2025)

- [ ] Configuration file support
- [ ] Performance profiling and optimization
- [ ] Caching layer

### Medium-term (Q3-Q4 2025)

- [ ] macOS GUI (Swift 6)
- [ ] Linux GUI
- [ ] Windows GUI
- [ ] Swift Package integration

### Long-term

- [ ] Plugin system
- [ ] Web UI
- [ ] Integration with popular tools

---

## Related Projects

- [gitoxide](https://github.com/Byron/gitoxide) – Pure Rust git implementation
- [Fluent](https://projectfluent.org/) – Localization framework
- [Clap](https://github.com/clap-rs/clap) – CLI argument parser

---

## Contributing

We welcome contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Checklist

- [ ] Read [CONTRIBUTING.md](CONTRIBUTING.md)
- [ ] Fork the repository
- [ ] Create a feature branch
- [ ] Make changes and test (`cargo test --all`)
- [ ] Format code (`cargo fmt`)
- [ ] Run linter (`cargo clippy -- -D warnings`)
- [ ] Create a Pull Request

---

## Acknowledgments

Built with:
- 🦀 [Rust](https://www.rust-lang.org/)
- 📦 [Gitoxide](https://github.com/Byron/gitoxide)
- 🌐 [Fluent](https://projectfluent.org/)
- 💬 [Clap](https://github.com/clap-rs/clap)

---

## FAQ

**Q: Does this modify my git repositories?**  
A: No. It only reads configuration and metadata. Your code is never changed.

**Q: Can I use this in production?**  
A: The MVP version is stable for reading. Not production-ready until v1.0.

**Q: Will it work with Subversion or Mercurial?**  
A: Currently git-only. Support for other VCS is not planned.

**Q: How do I use this in a GUI app?**  
A: Use the Rust library in `git-projects-core`. See [Using as a Library](#using-as-a-library) section.

**Q: Is there a GUI available?**  
A: Not yet. macOS Swift GUI planned for post-MVP. You can build your own using the library.

**Q: Can I contribute translations?**  
A: Yes! Open an issue or discussion to discuss your language.

---

**Last Updated:** January 2025

Made with ❤️ for developers who love organizing their projects.

