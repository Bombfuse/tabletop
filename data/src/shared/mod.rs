//! Shared helpers used across the data crate.
//!
//! Keep this module small and dependency-light; prefer putting domain-specific
//! helpers next to the domain modules.

use anyhow::Result;

/// Trim a user-provided identifier and validate it's non-empty.
///
/// Use this for things like primary keys / lookup names. Returns the trimmed
/// value on success.
pub fn require_non_empty_trimmed<'a>(field_label: &str, value: &'a str) -> Result<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("{field_label} must be non-empty");
    }
    Ok(trimmed)
}
