//! Simulator entrypoint.
//!
//! Responsibilities:
//! - Apply simulator DB migrations (best-effort)
//! - Launch the `iced` application

use iced::Application;

mod app;
mod db;
mod pages;
mod types;

/// Core tabletop database (read-only for simulator UI; used to list units).
pub const CORE_DB_PATH: &str = "tabletop.sqlite3";

fn main() -> iced::Result {
    // Run simulator migrations on startup (simulator DB is separate from core DB).
    //
    // If migrations fail, we still launch the UI so the error can be surfaced again
    // when the user tries to start/continue a campaign.
    if let Err(e) = db::apply_migrations() {
        eprintln!("Failed to apply simulator migrations: {e:#}");
    }

    app::Simulator::run(iced::Settings {
        window: iced::window::Settings {
            size: iced::Size::new(900.0, 600.0),
            ..Default::default()
        },
        ..Default::default()
    })
}
