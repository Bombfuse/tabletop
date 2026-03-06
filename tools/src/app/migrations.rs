use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, params};

/// Loads `.sql` files from `migrations_dir`, orders them lexicographically by filename,
/// and applies any not yet recorded in the `migrations` table.
///
/// Migration format: plain SQL. Each migration is run in its own transaction and recorded on success.
pub fn apply_migrations(conn: &mut Connection, migrations_dir: &Path) -> Result<()> {
    let migrations = discover_migrations(migrations_dir)?;
    for m in migrations {
        apply_one_if_needed(conn, &m.filename, &m.path)
            .with_context(|| format!("Failed to apply migration {}", m.filename))?;
    }
    Ok(())
}

struct MigrationFile {
    filename: String,
    path: PathBuf,
}

fn discover_migrations(migrations_dir: &Path) -> Result<Vec<MigrationFile>> {
    if !migrations_dir.exists() {
        // Nothing to do.
        return Ok(vec![]);
    }
    if !migrations_dir.is_dir() {
        bail!(
            "Migrations path exists but is not a directory: {}",
            migrations_dir.display()
        );
    }

    let mut out: Vec<MigrationFile> = Vec::new();

    for entry in std::fs::read_dir(migrations_dir)
        .with_context(|| format!("Failed to read dir {}", migrations_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("sql") {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .with_context(|| format!("Non-utf8 filename in {}", migrations_dir.display()))?;

        out.push(MigrationFile { filename, path });
    }

    // Apply in deterministic order.
    out.sort_by(|a, b| a.filename.cmp(&b.filename));

    Ok(out)
}

fn apply_one_if_needed(conn: &mut Connection, filename: &str, path: &Path) -> Result<()> {
    if migration_already_applied(conn, filename)? {
        return Ok(());
    }

    let sql = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read migration {}", path.display()))?;

    let tx = conn.transaction()?;
    tx.execute_batch(&sql)
        .with_context(|| format!("Migration `{}` failed", filename))?;
    tx.execute(
        "INSERT INTO migrations (filename) VALUES (?1)",
        params![filename],
    )?;
    tx.commit()?;

    Ok(())
}

fn migration_already_applied(conn: &Connection, filename: &str) -> Result<bool> {
    let applied: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM migrations WHERE filename = ?1 LIMIT 1",
            params![filename],
            |row| row.get(0),
        )
        .optional()?;
    Ok(applied.is_some())
}
