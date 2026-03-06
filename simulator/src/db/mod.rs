//! Simulator database module.
//!
//! This database is separate from the core tabletop database.
//! It is intended to persist simulator-only state like campaigns.
//!
//! Migrations are stored in `simulator/migrations/*.sql` and are embedded at
//! compile time via `include_str!`.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

pub const SIMULATOR_DB_PATH: &str = "simulator.sqlite3";

const MIGRATIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS migrations (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    filename      TEXT NOT NULL UNIQUE,
    applied_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
"#;

/// A minimal summary of a campaign persisted in the simulator DB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignSummary {
    pub id: i64,
    pub hero_unit_name: String,
    pub created_at: String,
}

/// Opens the simulator DB and applies common PRAGMAs.
///
/// Note: this does **not** automatically run migrations. Call
/// [`apply_migrations`] on startup (and before writes if you want extra safety).
pub fn open() -> Result<Connection> {
    let conn = Connection::open(SIMULATOR_DB_PATH)
        .with_context(|| format!("open simulator db at `{SIMULATOR_DB_PATH}`"))?;

    // Reasonable defaults for application DBs.
    conn.pragma_update(None, "foreign_keys", "ON")
        .context("enable foreign_keys for simulator db")?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .context("set journal_mode=WAL for simulator db")?;

    Ok(conn)
}

/// Applies all simulator migrations to the simulator database.
///
/// Migrations are applied in lexical order by filename.
/// Each migration file is applied at most once (tracked in the `migrations` table).
pub fn apply_migrations() -> Result<()> {
    let mut conn = open().context("open simulator db")?;

    conn.execute_batch(MIGRATIONS_TABLE_SQL)
        .context("ensure migrations table exists in simulator db")?;

    // Embedded migration files.
    //
    // NOTE: If you add a migration file, you must also add it to this list.
    let migrations: &[(&str, &str)] = &[(
        "0001_campaigns.sql",
        include_str!("../../migrations/0001_campaigns.sql"),
    )];

    for (filename, sql) in migrations {
        let already_applied: bool = conn
            .query_row(
                "SELECT 1 FROM migrations WHERE filename = ?1",
                rusqlite::params![filename],
                |_row| Ok(true),
            )
            .optional()
            .context("check whether migration is already applied")?
            .unwrap_or(false);

        if already_applied {
            continue;
        }

        let tx = conn
            .transaction()
            .with_context(|| format!("begin transaction for migration `{filename}`"))?;

        tx.execute_batch(sql)
            .with_context(|| format!("apply migration `{filename}`"))?;

        tx.execute(
            "INSERT INTO migrations (filename) VALUES (?1)",
            rusqlite::params![filename],
        )
        .with_context(|| format!("record migration `{filename}`"))?;

        tx.commit()
            .with_context(|| format!("commit migration `{filename}`"))?;
    }

    Ok(())
}

/// Inserts a new campaign into the simulator DB.
pub fn create_campaign(conn: &Connection, hero_unit_name: &str) -> Result<()> {
    let hero_unit_name = hero_unit_name.trim();
    anyhow::ensure!(
        !hero_unit_name.is_empty(),
        "hero_unit_name must be non-empty"
    );

    conn.execute(
        r#"
        INSERT INTO campaigns (hero_unit_name)
        VALUES (?1)
        "#,
        rusqlite::params![hero_unit_name],
    )
    .with_context(|| format!("insert campaign for hero `{hero_unit_name}`"))?;

    Ok(())
}

/// Lists existing campaigns (newest first).
pub fn list_campaigns(conn: &Connection) -> Result<Vec<CampaignSummary>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, hero_unit_name, created_at
            FROM campaigns
            ORDER BY datetime(created_at) DESC, id DESC
            "#,
        )
        .context("prepare list campaigns query")?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CampaignSummary {
                id: row.get(0)?,
                hero_unit_name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .context("query campaigns")?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.context("read campaign row")?);
    }
    Ok(out)
}
