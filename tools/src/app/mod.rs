mod cli;
mod db;
mod migrations;
mod paths;

use anyhow::{Context, Result};

pub fn run() -> Result<()> {
    let args = cli::parse();

    let tabletop_dir = paths::normalize_dir(&args.tabletop_dir)
        .with_context(|| format!("Invalid tabletop dir: {}", args.tabletop_dir.display()))?;

    let db_path = paths::resolve_under(&tabletop_dir, &args.db_path)?;
    let migrations_dir = paths::resolve_under(&tabletop_dir, &args.migrations_dir)?;

    paths::ensure_dir(&tabletop_dir)
        .with_context(|| format!("Failed to create tabletop dir: {}", tabletop_dir.display()))?;
    paths::ensure_parent_dir(&db_path)
        .with_context(|| format!("Failed to create db parent dir for: {}", db_path.display()))?;

    let mut conn = db::open_db(&db_path)
        .with_context(|| format!("Failed to open db at {}", db_path.display()))?;

    db::init_db(&conn).context("Failed to initialize database schema")?;

    migrations::apply_migrations(&mut conn, &migrations_dir).with_context(|| {
        format!(
            "Failed to apply migrations from {}",
            migrations_dir.display()
        )
    })?;

    Ok(())
}
