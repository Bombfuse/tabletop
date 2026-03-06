use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Best-effort normalization without requiring the path to exist.
///
/// Avoids canonicalization (which fails if the path doesn't exist yet).
/// Path helper utilities for the tools app.
pub fn normalize_dir(path: &Path) -> Result<PathBuf> {
    Ok(if path.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        path.to_path_buf()
    })
}

/// Walk up from `start_dir` to find a directory containing `migrations/`.
///
/// This is used to make the app robust to being launched from different working dirs
/// (workspace root, `tools/`, `target/...`, etc).
pub fn find_workspace_root(start_dir: &Path) -> Result<PathBuf> {
    let mut dir = start_dir;

    loop {
        let candidate = dir.join("migrations");
        if candidate.is_dir() {
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => anyhow::bail!(
                "Could not find workspace root: no parent directory contains a `migrations` directory (started from {})",
                start_dir.display()
            ),
        }
    }
}

/// Choose a robust default tabletop dir.
///
/// Preference order:
/// 1) If `start_dir` is (or is under) the workspace root, use the workspace root.
/// 2) Otherwise, fall back to `start_dir`.
pub fn default_tabletop_dir(start_dir: &Path) -> Result<PathBuf> {
    find_workspace_root(start_dir).or_else(|_| Ok(start_dir.to_path_buf()))
}

/// Resolves `p` under `root` if it is relative; passes absolute paths through as-is.
pub fn resolve_under(root: &Path, p: &Path) -> Result<PathBuf> {
    Ok(if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    })
}

/// Like `resolve_under`, but if `root` is relative it is interpreted relative to
/// a discovered workspace root (directory containing `migrations/`) starting from
/// the current working directory.
///
/// This makes relative config values resilient regardless of where the process is launched.
pub fn resolve_under_workspace_root(root: &Path, p: &Path) -> Result<PathBuf> {
    if root.is_absolute() {
        return resolve_under(root, p);
    }

    let cwd = std::env::current_dir().with_context(|| "Failed to get current_dir")?;
    let ws = find_workspace_root(&cwd)?;
    resolve_under(&ws.join(root), p)
}

/// Creates the directory (and parents) if it doesn't exist.
pub fn ensure_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Ensures the parent directory for `path` exists (if it has a parent).
pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
