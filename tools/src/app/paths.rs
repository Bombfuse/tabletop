use std::path::{Path, PathBuf};

use anyhow::Result;

/// Best-effort normalization without requiring the path to exist.
///
/// Avoids canonicalization (which fails if the path doesn't exist yet).
pub fn normalize_dir(path: &Path) -> Result<PathBuf> {
    Ok(if path.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        path.to_path_buf()
    })
}

/// Resolves `p` under `root` if it is relative; passes absolute paths through as-is.
pub fn resolve_under(root: &Path, p: &Path) -> Result<PathBuf> {
    Ok(if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    })
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
