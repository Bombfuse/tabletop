use std::path::PathBuf;

use clap::Parser;

/// Tabletop tools: initializes and migrates the local SQLite database.
#[derive(Debug, Parser)]
#[command(name = "tools", version, about)]
pub struct Args {
    /// Path to the tabletop folder (defaults to `..` so running from `tabletop/tools` targets the project root)
    #[arg(long, default_value = "..")]
    pub tabletop_dir: PathBuf,

    /// Path to the SQLite database file (relative to `--tabletop-dir` by default)
    #[arg(long, default_value = "tabletop.sqlite3")]
    pub db_path: PathBuf,

    /// Directory containing `.sql` migration files (relative to `--tabletop-dir` by default)
    #[arg(long, default_value = "migrations")]
    pub migrations_dir: PathBuf,
}

pub fn parse() -> Args {
    Args::parse()
}
