use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use iced::alignment;
use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Application, Command, Element, Length, Settings, Subscription, Theme};

use crate::app;

/// Run the GUI application.
///
/// Note: the GUI currently opens the DB and performs queries synchronously on the UI thread.
/// For small local DBs this is usually fine. If you notice stutters when listing many rows,
/// we can move DB work onto a background task.
pub fn run() -> iced::Result {
    let settings = Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1000.0, 720.0),
            ..Default::default()
        },
        ..Default::default()
    };

    ToolsGui::run(settings)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Units,
    Items,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Tab::Units => "Units",
            Tab::Items => "Items",
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    SwitchTab(Tab),

    Refresh,

    // Units form
    UnitNameChanged(String),
    UnitStrengthChanged(String),
    UnitFocusChanged(String),
    UnitIntelligenceChanged(String),
    UnitAgilityChanged(String),
    UnitKnowledgeChanged(String),
    CreateUnit,

    // Items form
    ItemNameChanged(String),
    CreateItem,

    // Delete actions
    DeleteUnit(String),
    DeleteItem(String),

    // Status
    ClearStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnitRow {
    name: String,
    strength: i64,
    focus: i64,
    intelligence: i64,
    agility: i64,
    knowledge: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ItemRow {
    name: String,
}

struct ToolsGui {
    tab: Tab,

    tabletop_dir: PathBuf,
    db_path: PathBuf,
    migrations_dir: PathBuf,

    // form state
    unit_name: String,
    unit_strength: String,
    unit_focus: String,
    unit_intelligence: String,
    unit_agility: String,
    unit_knowledge: String,

    item_name: String,

    // loaded data
    units: Vec<UnitRow>,
    items: Vec<ItemRow>,

    // ui state
    status: Option<String>,
}

impl Default for ToolsGui {
    fn default() -> Self {
        Self {
            tab: Tab::Units,

            // Match CLI defaults: tabletop_dir="..", db_path="tabletop.sqlite3", migrations_dir="migrations"
            tabletop_dir: PathBuf::from(".."),
            db_path: PathBuf::from("tabletop.sqlite3"),
            migrations_dir: PathBuf::from("migrations"),

            unit_name: String::new(),
            unit_strength: "0".to_string(),
            unit_focus: "0".to_string(),
            unit_intelligence: "0".to_string(),
            unit_agility: "0".to_string(),
            unit_knowledge: "0".to_string(),

            item_name: String::new(),

            units: vec![],
            items: vec![],

            status: None,
        }
    }
}

impl Application for ToolsGui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut app = Self::default();

        // Best-effort: init schema + apply migrations before first refresh.
        if let Err(e) = app.ensure_db_ready() {
            app.status = Some(format!("DB init failed: {e:#}"));
        }

        (app, Command::perform(async {}, |_| Message::Refresh))
    }

    fn title(&self) -> String {
        "Tabletop Tools".to_string()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // A light periodic tick can be used for future enhancements (auto-refresh, etc.).
        // For now it's effectively unused other than being available.
        iced::time::every(std::time::Duration::from_secs(60)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => Command::none(),

            Message::SwitchTab(tab) => {
                self.tab = tab;
                Command::none()
            }

            Message::Refresh => {
                if let Err(e) = self.refresh_lists() {
                    self.status = Some(format!("{e:#}"));
                }
                Command::none()
            }

            Message::UnitNameChanged(v) => {
                self.unit_name = v;
                Command::none()
            }
            Message::UnitStrengthChanged(v) => {
                self.unit_strength = v;
                Command::none()
            }
            Message::UnitFocusChanged(v) => {
                self.unit_focus = v;
                Command::none()
            }
            Message::UnitIntelligenceChanged(v) => {
                self.unit_intelligence = v;
                Command::none()
            }
            Message::UnitAgilityChanged(v) => {
                self.unit_agility = v;
                Command::none()
            }
            Message::UnitKnowledgeChanged(v) => {
                self.unit_knowledge = v;
                Command::none()
            }
            Message::CreateUnit => {
                if let Err(e) = self.create_unit_from_form() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Unit created".to_string());
                    self.unit_name.clear();
                    // keep stat inputs as-is
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::ItemNameChanged(v) => {
                self.item_name = v;
                Command::none()
            }
            Message::CreateItem => {
                if let Err(e) = self.create_item_from_form() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Item created".to_string());
                    self.item_name.clear();
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::DeleteUnit(name) => {
                if let Err(e) = self.delete_unit(&name) {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some(format!("Deleted unit `{name}`"));
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::DeleteItem(name) => {
                if let Err(e) = self.delete_item(&name) {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some(format!("Deleted item `{name}`"));
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::ClearStatus => {
                self.status = None;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let header = row![
            text("Tabletop Tools")
                .size(28)
                .horizontal_alignment(alignment::Horizontal::Left),
            iced::widget::Space::with_width(Length::Fill),
            button("Refresh").on_press(Message::Refresh),
        ]
        .spacing(12);

        let tabs = row![
            tab_button(self.tab, Tab::Units),
            tab_button(self.tab, Tab::Items),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(8);

        let status = if let Some(s) = &self.status {
            row![
                text(s.clone()).size(14),
                iced::widget::Space::with_width(Length::Fill),
                button("Dismiss").on_press(Message::ClearStatus)
            ]
            .spacing(12)
        } else {
            row![text("")] // keep layout stable
        };

        let content = match self.tab {
            Tab::Units => self.view_units_tab(),
            Tab::Items => self.view_items_tab(),
        };

        container(
            column![
                header,
                tabs,
                horizontal_rule(1),
                status,
                horizontal_rule(1),
                content
            ]
            .spacing(12),
        )
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn tab_button(current: Tab, tab: Tab) -> iced::widget::Button<'static, Message> {
    let label = tab.label();
    let mut b = button(label);
    if current != tab {
        b = b.on_press(Message::SwitchTab(tab));
    }
    b
}

impl ToolsGui {
    fn resolve_paths(&self) -> Result<(PathBuf, PathBuf, PathBuf)> {
        let tabletop_dir = app::paths::normalize_dir(&self.tabletop_dir)
            .with_context(|| format!("Invalid tabletop dir: {}", self.tabletop_dir.display()))?;

        let db_path = app::paths::resolve_under(&tabletop_dir, &self.db_path)?;
        let migrations_dir = app::paths::resolve_under(&tabletop_dir, &self.migrations_dir)?;

        Ok((tabletop_dir, db_path, migrations_dir))
    }

    fn ensure_db_ready(&self) -> Result<()> {
        let (tabletop_dir, db_path, migrations_dir) = self.resolve_paths()?;

        app::paths::ensure_dir(&tabletop_dir).with_context(|| {
            format!("Failed to create tabletop dir: {}", tabletop_dir.display())
        })?;
        app::paths::ensure_parent_dir(&db_path).with_context(|| {
            format!("Failed to create db parent dir for: {}", db_path.display())
        })?;

        let mut conn = app::db::open_db(&db_path)
            .with_context(|| format!("Failed to open db at {}", db_path.display()))?;

        app::db::init_db(&conn).context("Failed to initialize database schema")?;

        app::migrations::apply_migrations(&mut conn, &migrations_dir).with_context(|| {
            format!(
                "Failed to apply migrations from {}",
                migrations_dir.display()
            )
        })?;

        Ok(())
    }

    fn open_conn(&self) -> Result<rusqlite::Connection> {
        let (_tabletop_dir, db_path, _migrations_dir) = self.resolve_paths()?;
        app::db::open_db(&db_path).with_context(|| format!("open db: {}", db_path.display()))
    }

    fn refresh_lists(&mut self) -> Result<()> {
        let conn = self.open_conn()?;

        self.units = list_units(&conn)?;
        self.items = list_items(&conn)?;

        Ok(())
    }

    fn parse_i64_field(label: &str, s: &str) -> Result<i64> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            anyhow::bail!("{label} must not be empty");
        }
        let v: i64 = trimmed
            .parse()
            .with_context(|| format!("{label} must be an integer"))?;
        Ok(v)
    }

    fn create_unit_from_form(&self) -> Result<()> {
        let conn = self.open_conn()?;

        let unit = app::cards::unit::Unit {
            name: self.unit_name.trim().to_string(),
            strength: Self::parse_i64_field("Strength", &self.unit_strength)?,
            focus: Self::parse_i64_field("Focus", &self.unit_focus)?,
            intelligence: Self::parse_i64_field("Intelligence", &self.unit_intelligence)?,
            agility: Self::parse_i64_field("Agility", &self.unit_agility)?,
            knowledge: Self::parse_i64_field("Knowledge", &self.unit_knowledge)?,
        };

        app::cards::unit::save_card(&conn, &unit)?;
        Ok(())
    }

    fn create_item_from_form(&self) -> Result<()> {
        let conn = self.open_conn()?;
        let item = app::cards::item::Item {
            name: self.item_name.trim().to_string(),
        };
        app::cards::item::save_card(&conn, &item)?;
        Ok(())
    }

    fn delete_unit(&self, name: &str) -> Result<()> {
        let conn = self.open_conn()?;
        let _ = app::cards::unit::delete_card(&conn, name)?;
        Ok(())
    }

    fn delete_item(&self, name: &str) -> Result<()> {
        let conn = self.open_conn()?;
        let _ = app::cards::item::delete_card(&conn, name)?;
        Ok(())
    }

    fn view_units_tab(&self) -> Element<'_, Message> {
        let form = column![
            text("Create Unit").size(18),
            row![
                labeled_input("Name", &self.unit_name, Message::UnitNameChanged),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12),
            row![
                labeled_input(
                    "Strength",
                    &self.unit_strength,
                    Message::UnitStrengthChanged
                ),
                labeled_input("Focus", &self.unit_focus, Message::UnitFocusChanged),
                labeled_input(
                    "Intelligence",
                    &self.unit_intelligence,
                    Message::UnitIntelligenceChanged
                ),
            ]
            .spacing(12),
            row![
                labeled_input("Agility", &self.unit_agility, Message::UnitAgilityChanged),
                labeled_input(
                    "Knowledge",
                    &self.unit_knowledge,
                    Message::UnitKnowledgeChanged
                ),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12),
            row![button("Create").on_press(Message::CreateUnit)].spacing(12),
        ]
        .spacing(10);

        let list_header = row![
            text("Units").size(18),
            iced::widget::Space::with_width(Length::Fill),
            text(format!("{} total", self.units.len())).size(14),
        ];

        let mut list_col = column![list_header].spacing(8);

        for u in &self.units {
            let stats = format!(
                "STR {}  FOC {}  INT {}  AGI {}  KNO {}",
                u.strength, u.focus, u.intelligence, u.agility, u.knowledge
            );

            let row_el = row![
                column![text(u.name.clone()).size(16), text(stats).size(12)].spacing(2),
                iced::widget::Space::with_width(Length::Fill),
                button("Delete").on_press(Message::DeleteUnit(u.name.clone())),
            ]
            .spacing(12);

            list_col = list_col.push(container(row_el).padding(8));
        }

        let list = scrollable(list_col).height(Length::Fill);

        column![form, horizontal_rule(1), list]
            .spacing(12)
            .height(Length::Fill)
            .into()
    }

    fn view_items_tab(&self) -> Element<'_, Message> {
        let form = column![
            text("Create Item").size(18),
            row![
                labeled_input("Name", &self.item_name, Message::ItemNameChanged),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12),
            row![button("Create").on_press(Message::CreateItem)].spacing(12),
        ]
        .spacing(10);

        let list_header = row![
            text("Items").size(18),
            iced::widget::Space::with_width(Length::Fill),
            text(format!("{} total", self.items.len())).size(14),
        ];

        let mut list_col = column![list_header].spacing(8);

        for it in &self.items {
            let row_el = row![
                text(it.name.clone()).size(16),
                iced::widget::Space::with_width(Length::Fill),
                button("Delete").on_press(Message::DeleteItem(it.name.clone())),
            ]
            .spacing(12);

            list_col = list_col.push(container(row_el).padding(8));
        }

        let list = scrollable(list_col).height(Length::Fill);

        column![form, horizontal_rule(1), list]
            .spacing(12)
            .height(Length::Fill)
            .into()
    }
}

fn labeled_input<'a>(
    label: &'static str,
    value: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    // iced's TextInput requires placeholders; we use the label as placeholder for simplicity.
    // The label text is rendered above the input to be explicit.
    column![
        text(label).size(12),
        text_input(label, value)
            .on_input(on_change)
            .padding(8)
            .width(Length::Fixed(220.0)),
    ]
    .spacing(4)
    .into()
}

fn list_units(conn: &rusqlite::Connection) -> Result<Vec<UnitRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT name, strength, focus, intelligence, agility, knowledge
        FROM units
        ORDER BY name ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(UnitRow {
            name: row.get(0)?,
            strength: row.get(1)?,
            focus: row.get(2)?,
            intelligence: row.get(3)?,
            agility: row.get(4)?,
            knowledge: row.get(5)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn list_items(conn: &rusqlite::Connection) -> Result<Vec<ItemRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT name
        FROM items
        ORDER BY name ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| Ok(ItemRow { name: row.get(0)? }))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

// Small helper so we can keep referencing app::paths without exposing file paths here.
mod _path_helpers {
    use super::*;

    #[allow(dead_code)]
    pub fn display_path(p: &Path) -> String {
        p.display().to_string()
    }
}
