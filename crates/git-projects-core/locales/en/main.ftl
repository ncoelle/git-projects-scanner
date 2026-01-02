# Git Projects Scanner - English Translations

# General Messages
app-name = Git Projects Scanner
app-description = Scan and catalog Git repositories on your local filesystem

# Scanning Messages
scan-started = Scanning for Git repositories...
scan-started-path = Scanning: { $path }
scan-progress = Found { $count } { $count ->
    [one] repository
    *[other] repositories
} so far...
scan-complete = Scan complete! Found { $count } { $count ->
    [one] repository
    *[other] repositories
}.
scan-no-results = No Git repositories found.

# Table Headers
header-name = Name
header-path = Path
header-remotes = Remotes
header-config = Config
header-submodule = Submodule
header-has-submodules = Has Submodules
header-last-scanned = Last Scanned
header-service = Service
header-account = Account

# Remote Information
remote-none = (none)
remote-count = { $count } { $count ->
    [one] remote
    *[other] remotes
}

# Config Information
config-local = Local
config-global = Global
config-system = System
config-user = { $name } <{ $email }>
config-name-only = { $name }
config-email-only = <{ $email }>
config-none = (not configured)

# Submodule Status
submodule-yes = Yes
submodule-no = No

# Sorting Profiles
sort-name = By Name (alphabetical)
sort-path = By Path (alphabetical)
sort-recent = By Last Scanned (newest first)
sort-service = By Service (grouped)

# Output Formats
output-table = Table format (human-readable)
output-json = JSON format (machine-readable)

# Error Messages
error-io = I/O error: { $details }
error-git-open = Failed to open Git repository at { $path }
error-git-config = Failed to read Git config for { $path }
error-path-not-found = Path does not exist: { $path }
error-not-directory = Path is not a directory: { $path }
error-invalid-locale = Invalid locale: { $locale }
error-localization = Localization error: { $details }
error-json = JSON error: { $details }
error-unknown = An error occurred: { $details }

# CLI Help Text
help-root = Root directory to scan (can be specified multiple times)
help-depth = Maximum depth to recurse (default: 3)
help-no-symlinks = Don't follow symbolic links
help-no-submodules = Don't include submodule repositories
help-sort = Sorting profile: name, path, recent, or service
help-json = Output as JSON instead of table
help-verbose = Show detailed scanning progress
help-locale = Locale for messages (e.g., en, de)

# Verbose Output
verbose-analyzing = Analyzing: { $path }
verbose-found-repo = Found repository: { $name }
verbose-found-submodule = Found submodule: { $name }
verbose-skipping = Skipping: { $path }
verbose-warning = Warning: { $message }

# Status Messages
status-ok = ✓ OK
status-warning = ⚠ Warning
status-error = ✗ Error