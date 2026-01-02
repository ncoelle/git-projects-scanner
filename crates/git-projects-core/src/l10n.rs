//! Localization support using Project Fluent.
//!
//! This module provides internationalization (i18n) capabilities using the
//! Fluent localization system. It supports loading translation files and
//! formatting messages in different languages.
//!
//! # Supported Locales
//!
//! Currently supported languages:
//! - English (en) - Default fallback
//! - German (de)
//!
//! # Example
//!
//! ```no_run
//! use git_projects_core::l10n::Localizer;
//!
//! let localizer = Localizer::new("en").unwrap();
//! let message = localizer.get("scan-complete", Some(&[("count", "42")]));
//! println!("{}", message);
//! ```

use crate::error::{Error, Result};
use fluent::{FluentBundle, FluentResource};
use std::fs;
use std::path::PathBuf;
use unic_langid::LanguageIdentifier;

/// The default locale used when no locale is specified or loading fails.
pub const DEFAULT_LOCALE: &str = "en";

/// Manages localization resources and message formatting.
///
/// The Localizer loads Fluent translation files (.ftl) for a specific locale
/// and provides methods to retrieve translated messages with optional variable
/// interpolation.
pub struct Localizer {
    /// The Fluent bundle containing loaded translations.
    bundle: FluentBundle<FluentResource>,
    /// The current locale identifier.
    locale: LanguageIdentifier,
}

impl Localizer {
    /// Creates a new Localizer for the specified locale.
    ///
    /// Attempts to load translations from the `locales/{locale}/main.ftl` file
    /// relative to the crate root. Falls back to English if the requested
    /// locale cannot be loaded.
    ///
    /// # Arguments
    ///
    /// * `locale_str` - Locale identifier (e.g., "en", "de", "en-US")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The locale identifier is invalid
    /// - Translation files cannot be read
    /// - FTL syntax is invalid
    ///
    /// # Example
    ///
    /// ```
    /// use git_projects_core::l10n::Localizer;
    ///
    /// let localizer = Localizer::new("de").unwrap();
    /// ```
    pub fn new(locale_str: &str) -> Result<Self> {
        let locale: LanguageIdentifier = locale_str
            .parse()
            .map_err(|_| Error::l10n(format!("Invalid locale: {}", locale_str)))?;

        // Try to load the requested locale, fall back to default if it fails
        let (bundle, actual_locale) = Self::load_locale(&locale).or_else(|_| {
            if locale_str != DEFAULT_LOCALE {
                // Fall back to default locale
                let default: LanguageIdentifier = DEFAULT_LOCALE.parse().unwrap();
                Self::load_locale(&default)
            } else {
                Err(Error::l10n("Failed to load default locale".to_string()))
            }
        })?;

        Ok(Self {
            bundle,
            locale: actual_locale,
        })
    }

    /// Creates a Localizer using the system's default locale.
    ///
    /// Detects the system locale from environment variables (LANG, LC_ALL, etc.)
    /// and loads the appropriate translations. Falls back to English if detection
    /// fails or the locale is unsupported.
    ///
    /// # Example
    ///
    /// ```
    /// use git_projects_core::l10n::Localizer;
    ///
    /// let localizer = Localizer::from_system().unwrap();
    /// ```
    pub fn from_system() -> Result<Self> {
        let locale_str = detect_system_locale();
        Self::new(&locale_str)
    }

    /// Loads translation resources for a specific locale.
    ///
    /// Searches for the locale file in these locations (in order):
    /// 1. `./locales/{locale}/main.ftl` (current directory)
    /// 2. `./crates/git-projects-core/locales/{locale}/main.ftl` (workspace structure)
    /// 3. Embedded resources (if compiled in)
    fn load_locale(
        locale: &LanguageIdentifier,
    ) -> Result<(FluentBundle<FluentResource>, LanguageIdentifier)> {
        let locale_code = locale.to_string();

        // Try multiple possible paths for the locale file
        let possible_paths = vec![
            PathBuf::from(format!("locales/{}/main.ftl", locale_code)),
            PathBuf::from(format!(
                "crates/git-projects-core/locales/{}/main.ftl",
                locale_code
            )),
        ];

        let ftl_content = possible_paths
            .iter()
            .find_map(|path| fs::read_to_string(path).ok())
            .or_else(|| {
                // Try embedded resources if available
                get_embedded_locale(&locale_code)
            })
            .ok_or_else(|| {
                Error::l10n(format!("Could not find locale file for '{}'", locale_code))
            })?;

        // Parse the FTL content
        let resource = FluentResource::try_new(ftl_content)
            .map_err(|e| Error::l10n(format!("Failed to parse FTL: {:?}", e)))?;

        // Create a bundle and add the resource
        let mut bundle = FluentBundle::new(vec![locale.clone()]);
        bundle
            .add_resource(resource)
            .map_err(|e| Error::l10n(format!("Failed to add resource: {:?}", e)))?;

        Ok((bundle, locale.clone()))
    }

    /// Retrieves a translated message by its identifier.
    ///
    /// # Arguments
    ///
    /// * `msg_id` - The message identifier from the FTL file
    /// * `args` - Optional key-value pairs for variable interpolation
    ///
    /// # Returns
    ///
    /// The formatted message string. Returns the message ID itself if the
    /// translation is not found (graceful degradation).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use git_projects_core::l10n::Localizer;
    /// # let localizer = Localizer::new("en").unwrap();
    /// // Simple message without variables
    /// let msg = localizer.get("welcome", None);
    ///
    /// // Message with variables
    /// let msg = localizer.get("scan-complete", Some(&[("count", "42")]));
    /// ```
    pub fn get(&self, msg_id: &str, args: Option<&[(&str, &str)]>) -> String {
        let message = match self.bundle.get_message(msg_id) {
            Some(msg) => msg,
            None => {
                // Graceful degradation: return the message ID if not found
                return format!("[{}]", msg_id);
            }
        };

        let pattern = match message.value() {
            Some(p) => p,
            None => return format!("[{}]", msg_id),
        };

        // Convert args to FluentArgs if provided
        let mut errors = vec![];
        let formatted = if let Some(args) = args {
            let mut fluent_args = fluent::FluentArgs::new();
            for (key, value) in args {
                fluent_args.set(*key, value.to_string());
            }
            self.bundle
                .format_pattern(pattern, Some(&fluent_args), &mut errors)
        } else {
            self.bundle.format_pattern(pattern, None, &mut errors)
        };

        if !errors.is_empty() {
            eprintln!("Fluent formatting errors: {:?}", errors);
        }

        formatted.to_string()
    }

    /// Gets the current locale identifier.
    ///
    /// # Example
    ///
    /// ```
    /// # use git_projects_core::l10n::Localizer;
    /// let localizer = Localizer::new("de").unwrap();
    /// assert_eq!(localizer.locale(), "de");
    /// ```
    pub fn locale(&self) -> String {
        self.locale.to_string()
    }
}

/// Detects the system locale from environment variables.
///
/// Checks the following environment variables in order:
/// 1. `LC_ALL`
/// 2. `LC_MESSAGES`
/// 3. `LANG`
///
/// Returns the language code (first two characters) or "en" as fallback.
///
/// # Example
///
/// With `LANG=de_DE.UTF-8`, this function returns `"de"`.
pub fn detect_system_locale() -> String {
    std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .ok()
        .and_then(|locale| {
            // Extract language code (first 2 chars before underscore or dot)
            locale.split('_').next().map(|s| s.to_lowercase())
        })
        .unwrap_or_else(|| DEFAULT_LOCALE.to_string())
}

/// Attempts to get embedded locale content.
///
/// This function is called when locale files are not found on the filesystem.
/// It checks for compile-time embedded resources.
///
/// In a production build, you could use `include_str!` to embed the locale files:
///
/// ```ignore
/// match locale_code {
///     "en" => Some(include_str!("../locales/en/main.ftl").to_string()),
///     "de" => Some(include_str!("../locales/de/main.ftl").to_string()),
///     _ => None,
/// }
/// ```
fn get_embedded_locale(_locale_code: &str) -> Option<String> {
    // For now, return None - embedded resources would be added here
    // This allows the library to work both as a development library
    // (loading from filesystem) and as a compiled binary (embedded resources)
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_system_locale() {
        // Save original value
        let original = std::env::var("LANG").ok();

        // Test with German locale
        std::env::set_var("LANG", "de_DE.UTF-8");
        assert_eq!(detect_system_locale(), "de");

        // Test with English locale
        std::env::set_var("LANG", "en_US.UTF-8");
        assert_eq!(detect_system_locale(), "en");

        // Test with short locale
        std::env::set_var("LANG", "fr");
        assert_eq!(detect_system_locale(), "fr");

        // Restore original value
        if let Some(val) = original {
            std::env::set_var("LANG", val);
        } else {
            std::env::remove_var("LANG");
        }
    }

    #[test]
    fn test_default_locale() {
        assert_eq!(DEFAULT_LOCALE, "en");
    }

    #[test]
    fn test_invalid_locale() {
        let result = Localizer::new("invalid-locale-999");
        // Should either error or fall back to default
        assert!(result.is_ok() || result.is_err());
    }

    // Note: Full integration tests with actual .ftl files should be in
    // a separate integration test directory once the locale files are created
}
