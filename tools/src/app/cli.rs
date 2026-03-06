#![cfg(feature = "cli")]

use std::path::PathBuf;

use clap::{Parser, Subcommand};

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

    /// Optional command to run (if omitted, runs init + migrations only)
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Unit CRUD (by name)
    Unit {
        #[command(subcommand)]
        command: UnitCommand,
    },

    /// Item CRUD (by name)
    Item {
        #[command(subcommand)]
        command: ItemCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum UnitCommand {
    /// Save (upsert) a unit by name
    Save {
        #[arg(long)]
        name: String,

        #[arg(long)]
        strength: i64,
        #[arg(long)]
        focus: i64,
        #[arg(long)]
        intelligence: i64,
        #[arg(long)]
        agility: i64,
        #[arg(long)]
        knowledge: i64,
    },

    /// Get a unit by name
    Get {
        #[arg(long)]
        name: String,
    },

    /// List all units
    List,
}

#[derive(Debug, Subcommand)]
pub enum ItemCommand {
    /// Save (upsert) an item by name
    Save {
        #[arg(long)]
        name: String,
    },

    /// Get an item by name
    Get {
        #[arg(long)]
        name: String,
    },

    /// List all items
    List,
}

pub fn parse() -> Args {
    Args::parse()
}
