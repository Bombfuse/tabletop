mod cards;
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

    if let Some(cmd) = args.command {
        handle_command(&conn, cmd)?;
    }

    Ok(())
}

fn handle_command(conn: &rusqlite::Connection, cmd: cli::Command) -> Result<()> {
    match cmd {
        cli::Command::Unit { command } => match command {
            cli::UnitCommand::Save {
                name,
                strength,
                focus,
                intelligence,
                agility,
                knowledge,
            } => {
                let unit = cards::unit::Unit {
                    name,
                    strength,
                    focus,
                    intelligence,
                    agility,
                    knowledge,
                };
                let saved = cards::unit::save_card(conn, &unit)?;
                println!("{saved:?}");
                Ok(())
            }
            cli::UnitCommand::Get { name } => {
                let got = cards::unit::get_card(conn, &name)?;
                println!("{got:?}");
                Ok(())
            }
            cli::UnitCommand::List => {
                anyhow::bail!(
                    "unit list is not implemented after refactor (no list API in cards::unit)"
                );
            }
        },

        cli::Command::Item { command } => match command {
            cli::ItemCommand::Save { name } => {
                let item = cards::item::Item { name };
                let saved = cards::item::save_card(conn, &item)?;
                println!("{saved:?}");
                Ok(())
            }
            cli::ItemCommand::Get { name } => {
                let got = cards::item::get_card(conn, &name)?;
                println!("{got:?}");
                Ok(())
            }
            cli::ItemCommand::List => {
                anyhow::bail!(
                    "item list is not implemented after refactor (no list API in cards::item)"
                );
            }
        },
    }
}
