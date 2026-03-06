use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui;
use rusqlite::OptionalExtension;

fn main() -> eframe::Result<()> {
    // On startup, ensure we have a sqlite database in the repository root (../tabletop)
    // and apply any pending SQL migrations from tools/migrations.
    //
    // We treat DB/migration failures as fatal for now to keep startup behavior explicit.
    init_database_or_panic();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Tools",
        native_options,
        Box::new(|cc| Ok(Box::new(ToolsApp::new(cc)))),
    )
}

struct ToolsApp {
    last_action: Option<&'static str>,
    auto_close_deadline: Option<Instant>,
}

impl ToolsApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let auto_close_deadline = read_autoclose_deadline_from_env();
        Self {
            last_action: None,
            auto_close_deadline,
        }
    }
}

impl eframe::App for ToolsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // If AUTOCLOSE_TIMEOUT is set, close after that many seconds.
        if let Some(deadline) = self.auto_close_deadline {
            if Instant::now() >= deadline {
                // eframe 0.30: request the window/viewport to close via egui.
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }

            // Keep repainting so the countdown progresses even when idle.
            let remaining = deadline.saturating_duration_since(Instant::now());
            ctx.request_repaint_after(remaining.min(Duration::from_millis(100)));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tools");
            ui.add_space(8.0);

            if let Some(deadline) = self.auto_close_deadline {
                let remaining = deadline.saturating_duration_since(Instant::now());
                ui.label(format!(
                    "Auto-close in: {:.1}s (set via AUTOCLOSE_TIMEOUT)",
                    remaining.as_secs_f32()
                ));
                ui.add_space(8.0);
            } else {
                ui.label("Choose an option:");
                ui.add_space(8.0);
            }

            ui.horizontal(|ui| {
                if ui.button("Create Card").clicked() {
                    self.last_action = Some("Create Card");
                }
                if ui.button("Create Campaign").clicked() {
                    self.last_action = Some("Create Campaign");
                }
            });

            ui.add_space(12.0);

            if let Some(action) = self.last_action {
                ui.separator();
                ui.label(format!("Last action: {action}"));
            }
        });
    }
}

fn read_autoclose_deadline_from_env() -> Option<Instant> {
    let raw = std::env::var("AUTOCLOSE_TIMEOUT").ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Accept integers or floats (seconds).
    let seconds: f32 = match trimmed.parse() {
        Ok(v) => v,
        Err(_) => return None,
    };

    if !seconds.is_finite() || seconds <= 0.0 {
        return None;
    }

    Some(Instant::now() + Duration::from_secs_f32(seconds))
}

/// Initialize the sqlite database file in the tabletop repo root and apply any
/// migration SQL files in `tools/migrations` that haven't been applied yet.
fn init_database_or_panic() {
    if let Err(e) = init_database() {
        panic!("database initialization failed: {e}");
    }
}

fn init_database() -> Result<(), Box<dyn std::error::Error>> {
    // When running `cargo run` from `tabletop/tools`, CWD is typically `.../tabletop/tools`.
    // The repo root "tabletop" folder is one level up.
    let repo_root = std::env::current_dir()?.join("..");
    let db_path = repo_root.join("tabletop.sqlite3");
    let migrations_dir = std::env::current_dir()?.join("migrations");

    let mut conn = rusqlite::Connection::open(&db_path)?;

    // Apply migrations in a transaction to keep the DB consistent.
    let tx = conn.transaction()?;

    // Ensure the migrations table exists even if the "0001" file is missing.
    tx.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL UNIQUE,
            sha256 TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );
        "#,
    )?;

    let migration_files = list_sql_files_sorted(&migrations_dir)?;
    for path in migration_files {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("migration filename is not valid utf-8")?
            .to_string();

        let sql = std::fs::read_to_string(&path)?;
        let sha256 = sha256_hex(sql.as_bytes());

        // If already applied, skip. If applied but the content hash changed, fail loudly.
        let existing: Option<(String,)> = tx
            .query_row(
                "SELECT sha256 FROM schema_migrations WHERE filename = ?1",
                rusqlite::params![filename],
                |row| Ok((row.get(0)?,)),
            )
            .optional()?;

        if let Some((existing_sha,)) = existing {
            if existing_sha != sha256 {
                return Err(format!(
                    "migration {filename} was already applied but its contents changed (db sha256={existing_sha}, file sha256={sha256})"
                )
                .into());
            }
            continue;
        }

        tx.execute_batch(&sql)?;

        tx.execute(
            "INSERT INTO schema_migrations (filename, sha256) VALUES (?1, ?2)",
            rusqlite::params![filename, sha256],
        )?;
    }

    tx.commit()?;
    Ok(())
}

fn list_sql_files_sorted(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();

    // If the directory doesn't exist, treat as "no migrations" (non-fatal).
    if !dir.exists() {
        return Ok(out);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("sql") {
            out.push(path);
        }
    }

    out.sort_by(|a, b| {
        a.file_name()
            .and_then(|s| s.to_str())
            .cmp(&b.file_name().and_then(|s| s.to_str()))
    });

    Ok(out)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let digest = sha2::Sha256::digest(bytes);
    hex::encode(digest)
}
