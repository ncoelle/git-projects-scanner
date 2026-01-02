# git-projects-scanner – Design Document

**Status:** MVP Phase  
**Last Updated:** January 2025

---

## 1. Overview

`git-projects-scanner` is a tool for scanning and cataloging Git projects in a local directory (`~/Projects`).
It displays structured information about projects, their Git accounts, platforms, and remote URLs.

### Goals
- Inventory all local Git projects
- Multi-account/Multi-platform support
- Cross-platform (macOS, Linux, Windows)
- CLI + future GUI (macOS Swift, etc.)
- Open Source (MIT License)

### MVP Scope
- Rust Core Library (Read-Only Git Scanning)
- CLI Binary as reference implementation
- Predefined sorting profiles (not hierarchical)
- Fluent-based L10N (German, English)
- Polling only (no FSEvents in MVP)

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│          git-projects-scanner                       │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────────────────────────────────────┐   │
│  │  CLI Binary (projects-cli)                   │   │
│  │  - Command-line interface                    │   │
│  │  - Uses Core Library                         │   │
│  │  - Clap for argument parsing                 │   │
│  └──────────────────────────────────────────────┘   │
│                         ↓                           │
│  ┌──────────────────────────────────────────────┐   │
│  │  git-projects-core (Library)                 │   │
│  │  ├── Scanner (ProjectScanner)                │   │
│  │  ├── Models (GitProject, RemoteUrl, etc.)    │   │
│  │  ├── GitAnalyzer (gitoxide wrapper)          │   │
│  │  └── L10N (Fluent Integration)               │   │
│  └──────────────────────────────────────────────┘   │
│           ↓            ↓            ↓               │
│      gitoxide      Fluent      serde_json           │
│    (Git Ops)    (Localization) (Serialization)      │
│                                                     │
└─────────────────────────────────────────────────────┘

Future (Post-MVP):
├── macOS Swift App (uses Core via Swift Package)
├── Linux GTK App (via FFI/C Bindings)
└── Windows App (via FFI/C Bindings)
```

---

## 3. Data Model

### GitProject (Core Data Structure)

```rust
pub struct GitProject {
    pub name: String,                  // Project name (e.g. "StreamAPI")
    pub local_path: PathBuf,           // Local path (e.g., ~/Projects/Java/StreamAPI)
    pub platform_host: String,         // Hostname (e.g., "github.com", "gitlab.com")
    pub account: Option<String>,       // Account/User (e.g., "champion")
    pub remote_urls: Vec<RemoteUrl>,   // origin, upstream, etc.
    pub git_config: Option<GitConfig>, // user.name, user.email, scope
    pub contains_submodules: bool,     // Has submodules?
    pub is_valid_git_repo: bool,       // Is it a valid git repository?
}

pub struct RemoteUrl {
    pub name: String,                  // "origin", "upstream", etc.
    pub url: String,                   // Full URL
}

pub struct GitConfig {
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub scope: ConfigScope,            // Local, Global, System
}

pub enum ConfigScope {
    Local,
    Global,
    System,
}
```

### ScanConfig (Configuration Parameters)

```rust
pub struct ScanConfig {
    pub root_path: PathBuf,                    // Root directory to scan
    pub settings: HashMap<String, String>,     // Extensible settings
    pub locale: String,                        // "de" or "en"
}
```

---

## 4. Scanner Logic

### ProjectScanner Trait

```rust
pub trait ProjectScanner {
    fn scan(&self, config: &ScanConfig) -> Result<Vec<GitProject>>;
    fn scan_with_filter(&self, config: &ScanConfig, filter: &ScanFilter) 
        -> Result<Vec<GitProject>>;
}

pub struct ScanFilter {
    pub account: Option<String>,
    pub platform_host: Option<String>,
}
```

### Scanning Process

```
1. Iterate root directory (~/Projects)
   └─ For each subdirectory:

2. Is it a git repository?
   └─ .git directory present?
   └─ If no:
      ├─ In verbose mode: add as "not a git repo"
      └─ Otherwise: ignore

3. Extract git data
   ├─ Remote URLs: git config --get-regexp remote\..*\.url
   ├─ User config: git config user.name (local + global)
   ├─ Submodules: .gitmodules present?
   └─ Account: Parse from URL (best effort)

4. Scan for nested repositories
   └─ After finding first .git, don't search inside it
   └─ But: continue searching in subdirectories
   └─ Example: ~/Projects/Java/StreamAPI/.git + 
              ~/Projects/Java/StreamAPI/Example/.git = 2 entries

5. Collect results and return
```

**Why:** Gitoxide is ideal for read-only operations (pure Rust, no C dependencies)

---

## 5. Sorting & Output

### Sorting Profiles (MVP)

```
--sort=name              # Name A-Z (default)
--sort=platform          # Platform, then Account, then Name
--sort=account           # Account, then Platform, then Name
--sort=modified          # By last modification time
--sort=path              # By local path
```

Each profile can be reversed with `--reverse`:
```bash
projects --sort=platform --reverse
```

**Why:** Simple to implement, covers 80% of use cases.
Hierarchical sorting deferred to post-MVP.

### Output Formats

**Table (Default):**
```
Name        | Local Path          | Platform   | Account    | Remote URLs
────────────────────────────────────────────────────────────────────────
StreamAPI   | Java/StreamAPI      | Local      | -          | -
NBT         | Go/lo/nbt           | github.com | champion   | origin: https://github.com/champion/nbt
            |                     |            |            | upstream: https://github.com/original/nbt
```

**JSON (for GUIs):**

See `docs/API_SCHEMA.json` for the complete JSON schema.

**Status:** v0.1.0-alpha (unstable, may change before MVP completion)

---

## 6. Localization (L10N/I18N)

### Strategy

- **Format:** Fluent (.ftl files)
- **Languages (MVP):** German, English
- **Log Level:** Always English (for debugging)
- **Parameter:** Each API call takes `locale: &str`

### Fluent Structure

```
locales/
├── de/
│   └── main.ftl
└── en/
    └── main.ftl
```

**Example main.ftl (English):**
```
column-name = Name
column-path = Local Path
column-platform = Platform
column-account = Account
column-remotes = Remote URLs

error-not-git-repo = Not a valid git repository
error-invalid-path = Invalid path
status-contains-submodules = Contains submodules
status-is-local = Local (no remote)
```

### API Integration

```rust
pub fn scan_with_config(config: &ScanConfig) -> Result<Vec<GitProject>> {
    // config.locale is used internally for error/status strings
}
```

**Important:** Core returns structured data; GUI/CLI handles translation:
```rust
// In CLI: table headers translated based on locale
let header = i18n.get("column-name", locale)?;
```

---

## 7. CLI Interface (projects-cli Binary)

### Usage Examples

```bash
# List all projects
projects

# Filter by account
projects --account champion

# Filter by platform
projects --platform github.com

# Combined filters
projects --account champion --platform github.com

# Sorting
projects --sort=platform
projects --sort=account --reverse

# Output formats
projects --json                  # JSON instead of table
projects --sort=name --verbose   # Show user.name/email

# Language
projects --locale de
projects --locale en
```

### Exit Codes

```
0  - Success
1  - General error
2  - Invalid arguments
3  - Scan error (permission denied, etc.)
```

---

## 8. Workspace Structure

```
git-projects-scanner/
├── Cargo.toml                         # Workspace root
├── Cargo.lock
├── README.md
├── LICENSE (MIT)
├── CONTRIBUTING.md
│
├── crates/
│   └── git-projects-core/            # Main library
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs               # Public API
│       │   ├── models.rs            # Data structures
│       │   ├── scanner.rs           # Scanner trait & impl
│       │   ├── git_analyzer.rs      # Gitoxide wrapper
│       │   ├── l10n.rs              # Localization
│       │   ├── error.rs             # Error types
│       │   ├── bin/
│       │   │   └── projects-cli.rs  # CLI binary
│       │   └── tests/
│       │       ├── scanner_tests.rs
│       │       └── l10n_tests.rs
│       └── locales/
│           ├── de/
│           │   └── main.ftl
│           └── en/
│               └── main.ftl
│
├── docs/
│   ├── API_SCHEMA.json
│   ├── DESIGN.md (this file)
│   ├── OPEN_DECISIONS.md
│   ├── ADR/
│   │   ├── README.md
│   │   ├── 0001-rust-edition-and-msrv.md
│   │   ├── 0002-gitoxide-vs-git2.md
│   │   ├── 0003-l10n-strategy.md
│   │   ├── 0004-platform-detection.md
│   │   └── 0005-swift-integration-via-package.md
│   └── SWIFT_INTEGRATION.md (later)
│
├── .github/
│   └── workflows/
│       ├── test.yml
│       ├── build.yml
│       └── release.yml
│
└── Makefile
```

---

## 9. Dependencies (MVP)

| Crate        | Version  | Why                      | License        |
|--------------|----------|--------------------------|----------------|
| `gitoxide`   | ^0.35    | Pure Rust git operations | MIT            |
| `serde`      | ^1.0     | Serialization framework  | MIT/Apache-2.0 |
| `serde_json` | ^1.0     | JSON output              | MIT/Apache-2.0 |
| `fluent`     | ^0.16    | L10N framework           | MIT            |
| `clap`       | ^4.5     | CLI argument parsing     | MIT/Apache-2.0 |
| `anyhow`     | ^1.0     | Error handling           | MIT/Apache-2.0 |
| `dirs`       | ^5.0     | Home directory handling  | MIT/Apache-2.0 |

**Why these?**
- All under MIT/Apache-2.0 (compatible with MIT licence)
- Well-maintained, large ecosystems
- Minimal dependencies (avoid "dependency hell")

---

## 10. Testing Strategy

### Unit Tests
```rust
#[test]
fn test_parse_remote_url() { }

#[test]
fn test_extract_account_from_github_url() { }

#[test]
fn test_scan_empty_directory() { }
```

### Integration Tests
```rust
#[test]
fn test_scan_real_project() { }

#[test]
fn test_l10n_de() { }

#[test]
fn test_cli_output_json() { }
```

### How to Run
```bash
cargo test --all              # All tests
cargo test --lib             # Library only
cargo test --bin projects-cli # Binary only
cargo test -- --nocapture    # With output
```

---

## 11. Future Features (Post-MVP)

- [ ] Hierarchical sorting (`--sort-by platform,account,name`)
- [ ] Config file support (`.git-projects.toml`)
- [ ] Caching with SQLite
- [ ] FSEvents for macOS (performance optimization)
- [ ] Custom platform detection
- [ ] GUI implementations (macOS, Linux, Windows)
- [ ] Swift Package integration
- [ ] C FFI Bindings
- [ ] Code signing support
- [ ] Multiple OS account detection (in shared directories)

---

## 12. References

- Rust Edition 2021: https://doc.rust-lang.org/edition-guide/rust-2021/
- Gitoxide: https://github.com/Byron/gitoxide
- Fluent: https://projectfluent.org/
- Clap CLI Builder: https://docs.rs/clap/latest/clap/
- See CONTRIBUTING.md for developer setup

---
