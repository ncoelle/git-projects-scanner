# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for the git-projects-scanner project.

An ADR is a document that captures an important architectural decision made by the team, including the context, decision, rationale, and consequences.

## Format

Each ADR follows the standard format:

```
# ADR-XXXX: [Title]

## Status
[Proposed | Accepted | Deprecated | Superseded]

## Context
What problem are we trying to solve? Why is this decision needed?

## Decision
What did we decide?

## Rationale
Why this decision? What alternatives were considered?

## Consequences
What are the positive and negative impacts?

## Related Decisions
Links to other related ADRs
```

## ADRs

### Index

| # | Title | Status | Date |
|---|-------|--------|------|
| [0001](#adr-0001) | Rust Edition and MSRV | Accepted | 2025-01 |
| [0002](#adr-0002) | Git Library Selection: Gitoxide vs Git2 | Accepted | 2025-01 |
| [0003](#adr-0003) | Localization Strategy: Fluent | Accepted | 2025-01 |
| [0004](#adr-0004) | Platform Detection: Hostname String Instead of Enum | Accepted | 2025-01 |
| [0005](#adr-0005) | Swift Integration via Swift Package | Accepted | 2025-01 |

---

## ADR-0001

### Rust Edition and MSRV

**Status:** Accepted  
**Date:** January 2025

**Summary:** Use Rust Edition 2021 with MSRV 1.56+

### Context

Rust has multiple editions (2015, 2018, 2021, 2024). We need to decide which edition to use and what minimum Rust version (MSRV) to support.

**Considerations:**
- Rust 2024 Edition was recently announced
- 2021 is the current stable edition
- MSRV affects which dependencies we can use
- Users may not have the latest Rust toolchain

### Decision

- Use **Rust Edition 2021**
- Set **MSRV to 1.56+** (the edition-defining version)
- Do NOT use Rust 2024 Edition in MVP

### Rationale

1. **Stability:** Edition 2021 is widely deployed and stable
2. **Ecosystem:** All our dependencies fully support 2021
3. **Tooling:** IDEs, clippy, and linters have full support
4. **Future-proof:** Can upgrade to 2024 later (additive change)
5. **MSRV 1.56:** Good balance between modern features and compatibility

**Why not 2024?**
- Not yet broadly supported in ecosystem
- Many dependencies don't support it yet
- Breaking changes for users on older toolchains
- No killer features for our use case

### Consequences

**Positive:**
- Full ecosystem compatibility
- Stable, well-tested language features
- Easy to upgrade later

**Negative:**
- Miss out on 2024 features (syntactic sugar mostly)
- Need to maintain MSRV policy

### Related Decisions

- None directly related

---

## ADR-0002

### Git Library Selection: Gitoxide vs Git2

**Status:** Accepted  
**Date:** January 2025

**Summary:** Use `gitoxide` (pure Rust) instead of `git2` (libgit2 binding)

### Context

Two main Rust libraries for git operations:
1. **git2-rs:** Binding to libgit2 (C library), mature, widely used
2. **gitoxide:** Pure Rust implementation, newer but actively maintained

We need read-only access to:
- Remote URLs
- Local git config (user.name, user.email)
- Repository metadata

### Decision

Use **gitoxide** for all git operations.

### Rationale

1. **No C Dependencies:** Pure Rust means simpler cross-platform support
2. **Read-Only Sufficient:** gitoxide is excellent for read operations (our use case)
3. **Binary Size:** Smaller binaries without libgit2
4. **Cross-Platform:** Builds identically on macOS, Linux, Windows
5. **Active Maintenance:** Well-maintained, growing adoption

**Why not git2?**
- Requires libgit2 C library to be installed or bundled
- Adds build complexity (compilation of C code)
- Larger binaries
- Less pure Rust (FFI overhead)

### Consequences

**Positive:**
- Simpler dependency management
- Faster builds (no C compilation)
- Better cross-platform story
- Smaller binaries

**Negative:**
- Newer library (but well-maintained)
- Fewer StackOverflow answers (but improving)
- Less battle-tested than git2 (but improving)

### Migration Risk

Low risk: gitoxide API is stable for read operations. If issues arise, switching to git2 is feasible.

### Related Decisions

- ADR-0001: Rust Edition (supports gitoxide well)

---

## ADR-0003

### Localization Strategy: Fluent

**Status:** Accepted  
**Date:** January 2025

**Summary:** Use Fluent for L10N; Core returns structured data; GUIs handle translation

### Context

We need multi-language support (German, English, future: others).

**Options considered:**
1. **gettext/po:** Traditional, Linux standard
2. **JSON-based:** Simple but limited for complex strings
3. **Fluent:** Modern, used by Mozilla, good for ICU pluralization
4. **Hard-coded strings:** Not maintainable

### Decision

1. Use **Fluent** (.ftl files) for localization
2. **Core library:** Returns structured `GitProject` data (no translated strings)
3. **CLI/GUI:** Handles translation of display strings
4. **Logging:** Always in English (for debugging in field)
5. **API Parameter:** Each call takes `locale: &str` (e.g., "de", "en")

### Rationale

**Fluent:**
- Modern, maintained by Mozilla
- Used in Firefox, Thunderbird
- Excellent support for plurals, gender, context
- Extensible and readable syntax

**Core returns structured data:**
- Separation of concerns (Core ≠ GUI)
- GUIs have flexibility (can use own L10N system)
- Easier to test (no language-dependent logic)

**English logs:**
- Critical for production debugging
- Stack traces in English (standard practice)
- Developers can help remotely

**Per-call locale parameter:**
- Stateless, easy to test
- GUIs can switch language without app restart
- Better than global state

### Consequences

**Positive:**
- Modern, scalable L10N approach
- Clear separation between Core and UI
- Easy to add new languages
- GUIs have freedom in implementation

**Negative:**
- CLI/GUI must implement translation layer
- Slightly more code than hard-coded strings
- Need .ftl file management

### Example Implementation

```rust
// Core: returns structured data
pub fn scan(config: &ScanConfig) -> Result<Vec<GitProject>> {
    // Returns GitProject with is_valid_git_repo: bool, etc.
    // NO translated strings here
}

// CLI: handles translation
fn format_output(projects: &[GitProject], locale: &str) -> String {
    let i18n = load_fluent(locale)?;
    let header = i18n.get("column-name")?;
    // Format table with translated headers
}
```

### Related Decisions

- None directly related

---

## ADR-0004

### Platform Detection: Hostname String Instead of Enum

**Status:** Accepted  
**Date:** January 2025

**Summary:** Use `platform_host: String` (e.g., "github.com") instead of Enum

### Context

Git projects use different platforms:
- GitHub (github.com)
- GitLab (gitlab.com)
- Gitea (self-hosted)
- Codeberg (codeberg.org)
- Atlassian/Bitbucket
- Custom self-hosted instances

We need to represent these in `GitProject`.

**Options considered:**
1. **Enum:** `enum Platform { GitHub, GitLab, Gitea, ... }`
2. **String:** `platform_host: String` (raw hostname)
3. **Struct:** `struct Platform { host: String, platform_type: Enum }`

### Decision

Use **`platform_host: String`** (raw hostname from remote URL)

Additional fields:
- `account: Option<String>` (parsed from URL, best effort)
- Later: Can add `Platform` struct with enums if needed

### Rationale

1. **Flexibility:** Users have diverse platforms (we can't predict all)
2. **Future-proof:** Adding new platforms doesn't break schema
3. **Simplicity:** No enum mapping needed
4. **Self-hosted:** Captures hostname for internal Git servers
5. **Extensible:** Can add `platform_type: Enum` later without breaking API

**Why not Enum?**
- Restrictive (requires updating code for new platforms)
- Users with self-hosted instances would be "Other"
- Breaking change to add new variants

### Consequences

**Positive:**
- Handles any git platform (known or unknown)
- Self-hosted instances work naturally
- No breaking changes when adding platforms

**Negative:**
- Less type safety (String instead of Enum)
- GUIs must parse hostname for display names
- Can't rely on exhaustive pattern matching

### Future Enhancement

Post-MVP: Can add platform detection:
```rust
pub struct Platform {
    pub host: String,
    pub platform_type: Option<PlatformType>, // GitHub, GitLab, etc.
    pub api_url: Option<String>,
}

pub enum PlatformType {
    GitHub,
    GitLab,
    Gitea,
    Codeberg,
    Atlassian,
}
```

This is backward compatible (additive).

### Related Decisions

- ADR-0003: Localization (GUIs translate platform names)

---

## ADR-0005

### Swift Integration via Swift Package

**Status:** Accepted  
**Date:** January 2025

**Summary:** Future macOS GUI will use Swift Package (not C FFI)

### Context

The MVP is Rust core + CLI. Future work includes a native macOS GUI using Swift 6 + SwiftUI.

**Options considered:**
1. **C FFI:** Export C interface, wrap in Swift
2. **Swift Package:** Native Swift Package with Rust source
3. **Separate service:** macOS app calls CLI as subprocess
4. **Bridge library:** C++ bridge or similar

### Decision

Use **Swift Package** to integrate Rust core into macOS app.

This means:
- Rust library compiles to Swift-compatible binary
- Swift Package wraps the binary/library
- SwiftUI app uses Swift Package API

### Rationale

1. **Modern:** Swift Package is standard for iOS/macOS
2. **DX:** Better developer experience than C FFI
3. **Type-safe:** Swift types instead of C types
4. **Maintainable:** Swift Package Manager handles dependencies
5. **Native:** Feels native to Swift developers

**Why not C FFI?**
- More boilerplate (C types, memory management)
- Less idiomatic for Swift developers
- More error-prone

**Why not subprocess?**
- IPC overhead
- Not suitable for real-time interactions
- Binary distribution complexity

### Consequences

**Positive:**
- Modern, idiomatic integration
- Better tooling (Xcode integration)
- Type-safe Swift bindings

**Negative:**
- Requires learning Swift Package integration with Rust
- Adds complexity (Rust cross-compilation for macOS)
- Code signing / distribution complexity (later)

### Implementation Plan (Post-MVP)

1. Create Swift Package wrapper
2. Cross-compile Rust to macOS (Intel + Apple Silicon)
3. Create Swift bindings
4. Integrate into SwiftUI app
5. Handle code signing for distribution

### Related Decisions

- None directly related (but see docs/SWIFT_INTEGRATION.md)

---

## How to Add New ADRs

1. Create new file: `docs/ADR/XXXX-slug.md` (use next sequence number)
2. Follow the format above
3. Set Status to "Proposed"
4. Update this README's index
5. Create Pull Request for discussion
6. Once accepted, update Status to "Accepted"

## Superseded ADRs

Currently none.

## Deprecated ADRs

Currently none.

---

**Last Updated:** January 2025

See also:
- [DESIGN.md](../DESIGN.md) – Overall design
- [OPEN_DECISIONS.md](../OPEN_DECISIONS.md) – Deferred decisions
- [API_SCHEMA.json](../API_SCHEMA.json) – API contract
