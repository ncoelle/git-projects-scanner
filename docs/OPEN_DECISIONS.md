# Open Decisions & Future Features

**Last Updated:** January 2025

This document tracks architectural decisions that have been intentionally deferred and features planned for post-MVP phases.

---

## 1. Explicitly Deferred Decisions

### 1.1 Hierarchical Sorting

**Status:** Deferred to Post-MVP

**Context:**
User may want complex sorting like: "sort by platform, then by account, then by project name" with different directions per level.

**Current Solution (MVP):**
- Predefined profiles: `--sort=name`, `--sort=platform`, `--sort=account`, `--sort=modified`, `--sort=path`
- Single `--reverse` flag applies to all levels

**Future Approach:**
- Implement: `--sort-by platform,account,name --order asc,asc,desc`
- Or: Named profiles for common patterns

**Why Deferred:**
- MVP covers 80% of use cases
- Adds complexity to argument parsing and sorting logic
- Can be added without breaking API (additive feature)

**Decision Owner:** Core Team

---

### 1.2 Configuration File Support

**Status:** Deferred to Post-MVP

**Context:**
Users may want persistent configuration (ignore patterns, default sorting, custom aliases).

**Current Solution (MVP):**
- Everything via CLI parameters
- `ScanConfig` with extensible HashMap for future settings

**Future Approach:**
- `.git-projects.toml` or `.git-projects.yaml`
- Per-project overrides
- Global config in `~/.config/git-projects/`

**Example (future):**
```toml
[scan]
ignore_patterns = ["**/vendor", "**/node_modules"]
default_sort = "platform"

[projects.StreamAPI]
tags = ["personal", "java"]
ignore = true
```

**Why Deferred:**
- MVP doesn't need persistence
- File format decision hasn’t yet been made (TOML vs YAML)
- Core API already supports parameter passing

**Decision Owner:** TBD (post-MVP)

---

### 1.3 Caching & Performance Optimization

**Status:** Deferred to Post-MVP

**Context:**
With 100+ projects, repeated scans may be slow. Caching could help.

**Current Solution (MVP):**
- Sequential scanning with no state
- Each invocation is a fresh scan
- Polling-based (no FSEvents/inotify)

**Future Approaches:**
1. **In-memory cache:** Cache results for 5 minutes
2. **SQLite cache:** Persistent cache with invalidation
3. **FSEvents (macOS):** Watch for .git changes
4. **Parallel scanning:** Use Rayon for concurrent directory traversal

**Considerations:**
- Filesystem is the bottleneck, not CPU
- Too many parallel FS calls may hurt performance
- Need benchmarks before implementing

**Why Deferred:**
- MVP is fast enough for typical use cases (10–100 projects)
- Adds complexity without clear performance metrics
- Architecture already supports trait-based swapping

**Decision Owner:** Core Team (with benchmarks)

---

### 1.4 Custom Platform Detection

**Status:** Deferred to Post-MVP

**Context:**
Currently: `platform_host = String` (e.g., "github.com", "gitlab.com", "codeberg.org")

Some users may want:
- Custom Git server detection
- Platform name mapping ("GitHub" → "gh")
- Platform-specific metadata

**Current Solution (MVP):**
- Raw hostname from remote URL
- Best-effort account extraction from URL structure

**Future Approach:**
- Platform registry/database
- Custom URL pattern matching
- Platform-specific parsing rules

**Example (future):**
```rust
pub enum PlatformType {
    GitHub,
    GitLab,
    Gitea,
    Codeberg,
    Atlassian,
    Custom { host: String },
}

pub struct Platform {
    pub host: String,
    pub platform_type: PlatformType,
    pub api_url: Option<String>,
}
```

**Why Deferred:**
- MVP: hostname and account extraction sufficient
- Platform-specific features require research
- Can add without breaking the existing data model

**Decision Owner:** TBD

---

### 1.5 Multiple OS Account Detection

**Status:** Deferred to Post-MVP

**Context:**
In shared directories (e.g., `/shared/projects`), multiple OS accounts may own subdirectories.

**Current Assumption (MVP):**
- Single OS account (home-directory-based)
- All projects belong to one user

**Future Approach:**
- Optional: scan `/shared` with OS account detection
- Add to GitProject: `os_account: Option<String>`

**Why Deferred:**
- MVP assumes single-user (home directory)
- Requires OS-specific code (getpwuid on Unix, etc.)
- Feature request can be addressed later

**Decision Owner:** User feedback

---

## 2. Conscious Design Decisions (Finalized)

### 2.1 Rust Edition & MSRV

**Decision:** Rust Edition 2021, MSRV = 1.56+

**Rationale:**
- 2021 is stable and widely supported
- Rust 2024 is too new (not yet broadly supported)
- Can upgrade to 2024 later (additive change)

**Reference:** ADR-0001

---

### 2.2 Git Library: Gitoxide

**Decision:** Use `gitoxide` (pure Rust) over `git2` (libgit2 binding)

**Rationale:**
- Pure Rust implementation (no C dependencies)
- Read-only operations sufficient for MVP
- Better cross-platform story
- Smaller binary size

**Trade-off:** Newer library, but well-maintained and actively developed

**Reference:** ADR-0002

---

### 2.3 L10N Strategy: Fluent

**Decision:** 
- Use Fluent (.ftl files) for localization
- L10N handled by CLI/GUI, not Core
- `locale: &str` parameter passed to API calls
- Log messages always in English

**Rationale:**
- Fluent is modern and used by Mozilla/Firefox
- Core returns structured data (not strings)
- GUIs can choose their own L10N approach
- Testable logs (important for support)

**Reference:** ADR-0003

---

### 2.4 Platform Detection: Hostname String

**Decision:** 
- `platform_host: String` (e.g., "github.com")
- Not an enum
- Account extraction: "best effort"

**Rationale:**
- Users have diverse platforms (GitHub, GitLab, Codeberg, Atlassian, self-hosted, etc.)
- Enum would be too restrictive
- String + URL parsing is flexible
- Can add custom platform detection later without breaking API

**Reference:** ADR-0004

---

### 2.5 Swift Integration: Swift Package

**Decision:** 
- Future macOS GUI will use Swift Package
- Not C FFI at this time

**Rationale:**
- Modern approach (Swift native)
- Better DX for Swift developers
- FFI can be added later if needed

**Reference:** ADR-0005

---

### 2.6 Sorting: Predefined Profiles

**Decision:** 
- `--sort=name|platform|account|modified|path`
- Single `--reverse` flag
- Not hierarchical sorting (MVP)

**Rationale:**
- Simpler to implement
- Covers 80% of use cases
- Can add hierarchical later (backward compatible)

**Reference:** DESIGN.md Section 5

---

### 2.7 Scanning: Nested Repos

**Decision:** 
- After finding `.git`, don't search inside it
- But: continue searching in sibling directories
- Example: `Projekte/Java/StreamAPI/.git` + `Projekte/Java/StreamAPI/Example/.git` = 2 entries

**Rationale:**
- Avoids scanning Git internals
- Finds legitimate nested projects
- Works with monorepos and mixed structures

**Reference:** DESIGN.md Section 4

---

### 2.8 Submodules Handling

**Decision:** 
- Mark projects with submodules with `contains_submodules: bool`
- Don't recursively scan submodules as separate projects
- Treat as metadata

**Rationale:**
- Simpler implementation
- Submodules are dependencies, not separate projects
- Can be enhanced later if needed

**Reference:** DESIGN.md Section 3

---

### 2.9 License: MIT

**Decision:** Publish under MIT License

**Rationale:**
- Liberal, permissive license
- Compatible with all chosen dependencies
- Good for open source / community contributions

**Reference:** All crates compatible with MIT/Apache-2.0

---

### 2.10 Output: JSON + Table

**Decision:** 
- Default: Human-readable table
- `--json` flag: Machine-readable JSON
- JSON schema defined in `docs/API_SCHEMA.json`

**Rationale:**
- Table good for CLI users
- JSON good for GUI/automation
- Schema ensures consistency

**Reference:** DESIGN.md Section 5

---

## 3. Post-MVP Features (Backlog)

### High Priority

- [ ] **Config file support** – `.git-projects.toml`
  - Ignore patterns
  - Project tags/groups
  - Custom aliases
  
- [ ] **GUI Implementations**
  - macOS (Swift 6 + SwiftUI)
  - Linux (GTK4 via FFI or Rust GUI)
  - Windows (GTK4 via FFI or Rust GUI)

- [ ] **Caching** – Performance optimization
  - SQLite-based cache
  - Invalidation strategy
  - Benchmarks are needed first

### Medium Priority

- [ ] **Hierarchical Sorting** – `--sort-by platform,account,name`

- [ ] **Platform Detection** – Custom platform types beyond the hostname

- [ ] **Swift Package Integration** – For macOS app

- [ ] **Code Signing Support** – For macOS binary/app distribution

- [ ] **FSEvents for macOS** – Monitor changes instead of polling

- [ ] **Additional Metadata**
  - Git hooks detection
  - GPG signing config
  - Branch information
  - Dirty status (uncommitted changes)

### Lower Priority

- [ ] **Multiple OS Account Detection** – For shared directories

- [ ] **SQLite Schema Versioning** – For cache migrations

- [ ] **Plugin System** – Custom scanners?

- [ ] **Web UI** – Browser-based interface

- [ ] **CI Integration** – GitHub Actions, GitLab CI examples

---

## 4. Known Limitations (MVP)

### What's NOT Supported (v0.1.0)

1. **No persistence** – Every run re-scans everything
2. **No config files** – CLI parameters only
3. **No caching** – Suitable for <1000 projects
4. **No FSEvents** – Polling only (maybe slow on large directories)
5. **No GUI** – CLI only
6. **Single locale per run** – `--locale` per invocation
7. **No custom patterns** – Scans all subdirectories
8. **No platform-specific APIs** – Cross-platform only (basic)

### Performance Expectations

- **<100 projects:** <1 second
- **100–500 projects:** 1–5 seconds (depending on filesystem)
- **>500 projects:** May need caching (post-MVP)

---

## 5. Testing Assumptions

### What We'll Test (MVP)

- ✅ Scanning valid git repos
- ✅ Detecting submodules
- ✅ Parsing remote URLs (GitHub, GitLab, generic)
- ✅ Extracting user config (local + global)
- ✅ JSON schema validation
- ✅ CLI argument parsing
- ✅ L10N (German and English strings)
- ✅ Sorting profiles
- ✅ Filter logic (by account, platform)

### What We'll NOT Test (MVP)

- ✗ Performance benchmarks (deferred)
- ✗ Very large directories (1000+ projects)
- ✗ Exotic git configurations
- ✗ Windows-specific path handling (basic testing only)
- ✗ Code signing workflows
- ✗ GUI integration

---

## 6. Decision Process Going Forward

### How to Make New Decisions

1. **Identify** the decision (in issue or discussion)
2. **Create ADR** in `docs/ADR/` (see ADR template)
3. **Document** pros/cons
4. **Link** from this file
5. **Review** with the team
6. **Finalize** and reference from DESIGN.md

### When to Defer

- Feature requires significant implementation
- Not needed for MVP
- Can be added later without breaking API
- Blocked by other decisions

---

## 7. References

- See `docs/ADR/` for Architecture Decision Records
- See `docs/DESIGN.md` for design overview
- See `docs/API_SCHEMA.json` for JSON contract

---
