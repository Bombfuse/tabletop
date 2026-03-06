use iced::{Application, Element, Settings, Theme};

use data::cards::unit;
use rusqlite::OptionalExtension;

mod pages;

const CORE_DB_PATH: &str = "tabletop.sqlite3";
const SIMULATOR_DB_PATH: &str = "simulator.sqlite3";

const MIGRATIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS migrations (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    filename      TEXT NOT NULL UNIQUE,
    applied_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CampaignSummary {
    id: i64,
    hero_unit_name: String,
    created_at: String,
}

fn open_simulator_db() -> anyhow::Result<rusqlite::Connection> {
    use anyhow::Context;

    let conn = rusqlite::Connection::open(SIMULATOR_DB_PATH)
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
/// Migrations live in `simulator/migrations/*.sql` and are applied in lexical order
/// by filename. Each file is applied at most once (tracked in the `migrations` table).
fn apply_simulator_migrations() -> anyhow::Result<()> {
    use anyhow::Context;

    let mut conn = open_simulator_db().context("open simulator db")?;

    conn.execute_batch(MIGRATIONS_TABLE_SQL)
        .context("ensure migrations table exists in simulator db")?;

    // Load migration files embedded at compile-time.
    //
    // NOTE: If you add a migration file, you must also add it to this list.
    let migrations: &[(&str, &str)] = &[(
        "0001_campaigns.sql",
        include_str!("../migrations/0001_campaigns.sql"),
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

fn list_campaigns(conn: &rusqlite::Connection) -> anyhow::Result<Vec<CampaignSummary>> {
    use anyhow::Context;

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

fn main() -> iced::Result {
    // Run simulator migrations on startup (simulator DB is separate from core DB).
    if let Err(e) = apply_simulator_migrations() {
        eprintln!("Failed to apply simulator migrations: {e:#}");
        // Continue launching UI; DB errors will be surfaced again when starting a campaign.
    }

    Simulator::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(900.0, 600.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
enum Screen {
    MainMenu,
    CampaignSelectHero,
    CampaignContinueSelect,
    CampaignHome { campaign_id: i64 },
}

#[derive(Debug, Clone)]
enum Message {
    StartCampaign,
    ContinueCampaign,
    ExitApp,
    BackToMenu,

    SelectHero(String),
    BeginCampaign,

    SelectCampaign(i64),
}

struct Simulator {
    screen: Screen,

    units: Vec<unit::Unit>,
    selected_hero: Option<String>,
    load_error: Option<String>,

    campaign_saved: Option<String>,

    campaigns: Vec<CampaignSummary>,
}

impl Simulator {
    fn title(&self) -> String {
        match self.screen {
            Screen::MainMenu => "Tabletop Simulator".to_string(),
            Screen::CampaignSelectHero => "Tabletop Simulator — Start Campaign".to_string(),
            Screen::CampaignContinueSelect => "Tabletop Simulator — Continue Campaign".to_string(),
            Screen::CampaignHome { campaign_id } => {
                format!("Tabletop Simulator — Campaign #{campaign_id}")
            }
        }
    }
}

impl iced::Application for Simulator {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                screen: Screen::MainMenu,
                units: Vec::new(),
                selected_hero: None,
                load_error: None,
                campaign_saved: None,
                campaigns: Vec::new(),
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        self.title()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::StartCampaign => {
                self.screen = Screen::CampaignSelectHero;

                self.load_error = None;
                self.campaign_saved = None;
                self.units.clear();

                // NOTE: iced `Application::update` is sync, so we load synchronously here.
                // If this becomes slow, move DB calls to a subscription/task.
                //
                // Core tabletop DB is used ONLY for reading reference/unit data.
                let db_path = std::path::Path::new(CORE_DB_PATH);
                match data::db::open_db(db_path).and_then(|conn| unit::list_cards(&conn)) {
                    Ok(units) => self.units = units,
                    Err(e) => self.load_error = Some(e.to_string()),
                }

                iced::Command::none()
            }
            Message::ContinueCampaign => {
                self.screen = Screen::CampaignContinueSelect;

                self.load_error = None;
                self.campaign_saved = None;
                self.campaigns.clear();

                if let Err(e) = apply_simulator_migrations() {
                    self.load_error = Some(format!("Failed to apply simulator migrations: {e}"));
                    return iced::Command::none();
                }

                match open_simulator_db().and_then(|conn| list_campaigns(&conn)) {
                    Ok(campaigns) => self.campaigns = campaigns,
                    Err(e) => self.load_error = Some(e.to_string()),
                }

                iced::Command::none()
            }
            Message::SelectHero(name) => {
                self.selected_hero = Some(name);
                self.campaign_saved = None;
                iced::Command::none()
            }
            Message::BeginCampaign => {
                self.load_error = None;
                self.campaign_saved = None;

                let Some(hero_name) = self.selected_hero.clone() else {
                    return iced::Command::none();
                };

                // Ensure simulator DB schema is up-to-date before writing campaign data.
                if let Err(e) = apply_simulator_migrations() {
                    self.load_error = Some(format!("Failed to apply simulator migrations: {e}"));
                    return iced::Command::none();
                }

                match open_simulator_db().and_then(|conn| {
                    conn.execute(
                        r#"
                        INSERT INTO campaigns (hero_unit_name)
                        VALUES (?1)
                        "#,
                        rusqlite::params![hero_name],
                    )?;
                    Ok(())
                }) {
                    Ok(()) => {
                        self.campaign_saved =
                            Some("Campaign created and saved to simulator database.".to_string());
                    }
                    Err(e) => {
                        self.load_error = Some(e.to_string());
                    }
                }

                iced::Command::none()
            }
            Message::SelectCampaign(campaign_id) => {
                self.load_error = None;
                self.campaign_saved = None;
                self.screen = Screen::CampaignHome { campaign_id };
                iced::Command::none()
            }
            Message::BackToMenu => {
                self.screen = Screen::MainMenu;
                iced::Command::none()
            }
            Message::ExitApp => iced::window::close(iced::window::Id::MAIN),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self.screen {
            Screen::MainMenu => pages::main_menu::view(),
            Screen::CampaignSelectHero => pages::start_campaign::view(
                &self.units,
                self.selected_hero.as_deref(),
                self.campaign_saved.as_deref(),
                self.load_error.as_deref(),
            ),
            Screen::CampaignContinueSelect => {
                pages::continue_campaign::view(&self.campaigns, self.load_error.as_deref())
            }
            Screen::CampaignHome { campaign_id } => pages::campaign_home::view(campaign_id),
        }
    }
}
