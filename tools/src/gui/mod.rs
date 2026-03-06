mod views;

use std::path::PathBuf;

use anyhow::{Context, Result};
use iced::widget::{button, column, container, horizontal_rule, row, text};
use iced::{Application, Command, Element, Length, Settings, Subscription, Theme};

use crate::app;

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
pub enum Tab {
    Units,
    Items,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::Units => "Units",
            Tab::Items => "Items",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveView {
    List,
    EditUnit { original_name: String },
    EditItem { original_name: String },
}

#[derive(Debug, Clone)]
pub enum Message {
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

    // Edit navigation
    EditUnit(String),
    EditItem(String),
    CancelEdit,

    // Save edits
    SaveUnitEdits,
    SaveItemEdits,

    // Delete actions
    DeleteUnit(String),
    DeleteItem(String),

    // Status
    ClearStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitRow {
    pub name: String,
    pub strength: i64,
    pub focus: i64,
    pub intelligence: i64,
    pub agility: i64,
    pub knowledge: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRow {
    pub name: String,
}

pub struct ToolsGui {
    pub tab: Tab,

    pub tabletop_dir: PathBuf,
    pub db_path: PathBuf,
    pub migrations_dir: PathBuf,

    // Units form / edit buffer
    pub unit_name: String,
    pub unit_strength: String,
    pub unit_focus: String,
    pub unit_intelligence: String,
    pub unit_agility: String,
    pub unit_knowledge: String,

    // Items form / edit buffer
    pub item_name: String,

    // loaded data
    pub units: Vec<UnitRow>,
    pub items: Vec<ItemRow>,

    // ui state
    pub status: Option<String>,
    pub active_view: ActiveView,
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
            active_view: ActiveView::List,
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
        iced::time::every(std::time::Duration::from_secs(60)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => Command::none(),

            Message::SwitchTab(tab) => {
                self.tab = tab;
                self.active_view = ActiveView::List;
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

            Message::EditUnit(name) => {
                if let Err(e) = self.begin_edit_unit(&name) {
                    self.status = Some(format!("{e:#}"));
                }
                Command::none()
            }

            Message::EditItem(name) => {
                if let Err(e) = self.begin_edit_item(&name) {
                    self.status = Some(format!("{e:#}"));
                }
                Command::none()
            }

            Message::CancelEdit => {
                self.active_view = ActiveView::List;
                Command::none()
            }

            Message::SaveUnitEdits => {
                if let Err(e) = self.save_unit_edits() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Unit updated".to_string());
                    self.active_view = ActiveView::List;
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }

            Message::SaveItemEdits => {
                if let Err(e) = self.save_item_edits() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Item updated".to_string());
                    self.active_view = ActiveView::List;
                    Command::perform(async {}, |_| Message::Refresh)
                }
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
            text("Tabletop Tools").size(28),
            iced::widget::Space::with_width(Length::Fill),
            button("Refresh").on_press(Message::Refresh),
        ]
        .spacing(12);

        let tabs = row![
            views::tab_button(self.tab, Tab::Units),
            views::tab_button(self.tab, Tab::Items),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(8);

        let status = views::status_bar(self.status.as_deref());

        let content = match self.tab {
            Tab::Units => match &self.active_view {
                ActiveView::List => views::units::view(self),
                ActiveView::EditUnit { original_name } => {
                    views::units::edit_view(self, original_name)
                }
                _ => views::units::view(self),
            },
            Tab::Items => match &self.active_view {
                ActiveView::List => views::items::view(self),
                ActiveView::EditItem { original_name } => {
                    views::items::edit_view(self, original_name)
                }
                _ => views::items::view(self),
            },
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

        self.units = views::units::list_units(&conn)?;
        self.items = views::items::list_items(&conn)?;

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

    fn begin_edit_unit(&mut self, name: &str) -> Result<()> {
        let conn = self.open_conn()?;
        let u = app::cards::unit::get_card(&conn, name)?
            .with_context(|| format!("Unit `{name}` not found"))?;

        self.unit_name = u.name.clone();
        self.unit_strength = u.strength.to_string();
        self.unit_focus = u.focus.to_string();
        self.unit_intelligence = u.intelligence.to_string();
        self.unit_agility = u.agility.to_string();
        self.unit_knowledge = u.knowledge.to_string();

        self.active_view = ActiveView::EditUnit {
            original_name: u.name,
        };
        Ok(())
    }

    fn begin_edit_item(&mut self, name: &str) -> Result<()> {
        let conn = self.open_conn()?;
        let it = app::cards::item::get_card(&conn, name)?
            .with_context(|| format!("Item `{name}` not found"))?;

        self.item_name = it.name.clone();

        self.active_view = ActiveView::EditItem {
            original_name: it.name,
        };
        Ok(())
    }

    fn save_unit_edits(&self) -> Result<()> {
        let ActiveView::EditUnit { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing a unit");
        };

        let conn = self.open_conn()?;

        let new_name = self.unit_name.trim().to_string();
        if new_name.is_empty() {
            anyhow::bail!("Unit.name must be non-empty");
        }

        let unit = app::cards::unit::Unit {
            name: new_name,
            strength: Self::parse_i64_field("Strength", &self.unit_strength)?,
            focus: Self::parse_i64_field("Focus", &self.unit_focus)?,
            intelligence: Self::parse_i64_field("Intelligence", &self.unit_intelligence)?,
            agility: Self::parse_i64_field("Agility", &self.unit_agility)?,
            knowledge: Self::parse_i64_field("Knowledge", &self.unit_knowledge)?,
        };

        let updated = app::cards::unit::rename_and_update_card(&conn, original_name, &unit)?
            .with_context(|| format!("Unit `{}` does not exist", original_name))?;
        let _ = updated;

        Ok(())
    }

    fn save_item_edits(&self) -> Result<()> {
        let ActiveView::EditItem { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing an item");
        };

        let conn = self.open_conn()?;

        let new_name = self.item_name.trim().to_string();
        if new_name.is_empty() {
            anyhow::bail!("Item.name must be non-empty");
        }

        let item = app::cards::item::Item { name: new_name };

        let updated = app::cards::item::rename_card(&conn, original_name, &item)?
            .with_context(|| format!("Item `{}` does not exist", original_name))?;
        let _ = updated;

        Ok(())
    }
}
