use anyhow::{Context, Result};

/// Parse a required `i64` field from a GUI text input.
///
/// - Trims whitespace.
/// - Rejects empty input.
/// - Adds a human-friendly `label` to error messages.
pub fn parse_i64_required(label: &str, s: &str) -> Result<i64> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        anyhow::bail!("{label} must not be empty");
    }

    let v: i64 = trimmed
        .parse()
        .with_context(|| format!("{label} must be an integer"))?;

    Ok(v)
}

/// Parse an optional `i64` field from a GUI text input.
///
/// - Trims whitespace.
/// - Treats empty input as `None`.
/// - Adds a human-friendly `label` to error messages when non-empty but invalid.
pub fn parse_i64_optional(label: &str, s: &str) -> Result<Option<i64>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let v: i64 = trimmed
        .parse()
        .with_context(|| format!("{label} must be an integer"))?;

    Ok(Some(v))
}

/// Helper for parsing an enum-like string input with a custom validator.
///
/// Prefer this when you want consistent error context for "pick one of these strings" fields.
pub fn parse_choice<T>(label: &str, raw: &str, parse: impl FnOnce(&str) -> Result<T>) -> Result<T> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("{label} must not be empty");
    }
    parse(trimmed).with_context(|| format!("Invalid {label}"))
}
