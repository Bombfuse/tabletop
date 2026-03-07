mod shared;
mod views;

fn parse_optional_i64_from_input(label: &str, s: &str) -> anyhow::Result<Option<i64>> {
    let t = s.trim();
    if t.is_empty() {
        return Ok(None);
    }
    let v: i64 = t
        .parse()
        .map_err(|e| anyhow::anyhow!("{label} must be an integer (or empty): {e}"))?;
    Ok(Some(v))
}

fn parse_stat_choice(s: &str) -> anyhow::Result<app::cards::stat_modifier::Stat> {
    let t = s.trim();
    match t {
        "Strength" => Ok(app::cards::stat_modifier::Stat::Strength),
        "Focus" => Ok(app::cards::stat_modifier::Stat::Focus),
        "Intelligence" => Ok(app::cards::stat_modifier::Stat::Intelligence),
        "Knowledge" => Ok(app::cards::stat_modifier::Stat::Knowledge),
        "Agility" => Ok(app::cards::stat_modifier::Stat::Agility),
        other => Err(anyhow::anyhow!(
            "Stat must be one of Strength, Focus, Intelligence, Knowledge, Agility (got: {other})"
        )),
    }
}

fn parse_stat_operator_choice(
    s: &str,
) -> anyhow::Result<app::cards::stat_modifier::StatModifierOperator> {
    let t = s.trim();
    match t {
        "Add" => Ok(app::cards::stat_modifier::StatModifierOperator::Add),
        "Subtract" => Ok(app::cards::stat_modifier::StatModifierOperator::Subtract),
        other => Err(anyhow::anyhow!(
            "Operator must be Add or Subtract (got: {other})"
        )),
    }
}

pub fn normalize_for_match(s: &str) -> String {
    s.trim().to_lowercase()
}

pub fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.trim().is_empty() {
        return true;
    }
    normalize_for_match(haystack).contains(&normalize_for_match(needle))
}

use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use iced::widget::{button, column, container, horizontal_rule, row, text};
use iced::{Application, Command, Element, Length, Settings, Subscription, Theme};
use rusqlite::OptionalExtension;

use crate::app;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingStatModifierRow {
    pub stat: String,
    pub value: i64,
    pub operator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionAssociationKind {
    None,
    Unit,
    Item,
    Level,
}

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
    Levels,
    Actions,
    ArmorModifiers,
    HexGrids,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::Units => "Units",
            Tab::Items => "Items",
            Tab::Levels => "Levels",
            Tab::Actions => "Actions",
            Tab::ArmorModifiers => "Armor Modifiers",
            Tab::HexGrids => "Hex Grids",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveView {
    List,
    EditUnit { original_name: String },
    EditItem { original_name: String },
    EditLevel { original_name: String },
    EditAction { original_name: String },
    EditArmorModifier { id: i64 },
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    SwitchTab(Tab),

    // Refresh data
    Refresh,

    // Stat modifiers form buffers
    StatModifierValueChanged(String),
    StatModifierStatChanged(String),
    StatModifierOperatorChanged(String),

    // Queue stat modifiers during create flows (Item/Level)
    AddPendingStatModifier,
    RemovePendingStatModifier(usize),
    ClearPendingStatModifiers,

    // Create stat modifiers while editing cards
    CreateUnitStatModifier,
    CreateItemStatModifier,
    CreateLevelStatModifier,

    // Delete stat modifiers from card views
    DeleteStatModifier(i64),

    // Hex grid editor
    HexGridWidthChanged(String),
    HexGridHeightChanged(String),
    HexGridNameChanged(String),
    CreateNewHexGrid,
    SaveHexGrid,
    RefreshHexGrids,
    LoadHexGridById(i64),
    DeleteHexGridById(i64),
    HexGridApplyResize,
    HexGridTileClicked(i32, i32),
    HexGridTileClear(i32, i32),

    HexTileUnitQueryChanged(String),
    HexTileItemQueryChanged(String),
    HexTileLevelQueryChanged(String),

    HexTilePickUnitByName(String),
    HexTilePickItemByName(String),
    HexTilePickLevelByName(String),

    HexTileTypeChanged(String),

    SaveHexTileAssociations,
    ClearHexTileAssociations,

    ArmorModifierValueChanged(String),
    ArmorModifierSuitChanged(String),
    ArmorModifierDamageTypeChanged(String),

    AddPendingArmorModifier,
    RemovePendingArmorModifier(usize),
    ClearPendingArmorModifiers,

    CreateArmorModifier,

    RemoveArmorModifierLink(i64),

    EditArmorModifier(i64),
    SaveArmorModifierEdits,
    DeleteArmorModifier(i64),

    UnitNameChanged(String),
    UnitStrengthChanged(String),
    UnitFocusChanged(String),
    UnitIntelligenceChanged(String),
    UnitAgilityChanged(String),
    UnitKnowledgeChanged(String),
    CreateUnit,

    ItemNameChanged(String),
    CreateItem,

    LevelNameChanged(String),
    LevelTextChanged(String),
    CreateLevel,

    ActionNameChanged(String),
    ActionPointCostChanged(String),
    ActionTypeChanged(String),
    ActionTextChanged(String),

    CreateAction,

    UnitAssocActionNameChanged(String),
    ItemAssocActionNameChanged(String),
    LevelAssocActionNameChanged(String),

    AddUnitAssociation,
    RemoveUnitAssociation(String),

    AddItemAssociation,
    RemoveItemAssociation(String),

    AddLevelAssociation,
    RemoveLevelAssociation(String),

    CreateUnitAndMaybeAssociate,
    CreateItemAndMaybeAssociate,
    CreateLevelAndMaybeAssociate,

    AttackDamageChanged(String),
    AttackDamageTypeChanged(String),
    AttackSkillChanged(String),
    AttackTargetChanged(String),
    AttackRangeChanged(String),

    SaveAttackEdits,
    DeleteAttack,

    InteractionRangeChanged(String),
    InteractionSkillChanged(String),
    InteractionTargetChanged(String),

    SaveInteractionEdits,
    DeleteInteraction,

    EditUnit(String),
    EditItem(String),
    EditLevel(String),
    EditAction(String),

    CancelEdit,

    SaveUnitEdits,
    SaveItemEdits,
    SaveLevelEdits,
    SaveActionEdits,

    DeleteUnit(String),
    DeleteItem(String),
    DeleteLevel(String),
    DeleteAction(String),

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelRow {
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRow {
    pub name: String,
    pub action_point_cost: i64,
    pub action_type: String,
    pub text: String,

    // Optional association (exactly one of these should be Some at a time, or all None).
    pub unit_name: Option<String>,
    pub item_name: Option<String>,
    pub level_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmorModifierRow {
    pub id: i64,
    pub card_id: i64,
    pub value: i64,
    pub suit: String,
    pub damage_type: String,

    // Optional association (at most one should be Some at a time, or all None).
    pub item_name: Option<String>,
    pub level_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatModifierRow {
    pub id: i64,
    pub stat: String,
    pub value: i64,
    pub operator: String,

    // Optional association (at most one should be Some at a time, or all None).
    pub unit_name: Option<String>,
    pub item_name: Option<String>,
    pub level_name: Option<String>,
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

    // Unit -> Action association buffer (set from Unit edit view)
    pub unit_assoc_action_name: String,

    // Items form / edit buffer
    pub item_name: String,

    // Item -> Action association buffer (set from Item edit view)
    pub item_assoc_action_name: String,

    // Levels form / edit buffer
    pub level_name: String,
    pub level_text: String,

    // Level -> Action association buffer (set from Level edit view)
    pub level_assoc_action_name: String,

    // Actions form / edit buffer
    pub action_name: String,
    pub action_point_cost: String,
    pub action_type: String,
    pub action_text: String,

    // Attack subform buffer (for action edit view)
    pub attack_damage: String,
    pub attack_damage_type: String,
    pub attack_skill: String,
    pub attack_target: String,
    pub attack_range: String,

    // Interaction subform buffer (for action edit view)
    pub interaction_range: String,
    pub interaction_skill: String,
    pub interaction_target: String,

    // Armor modifiers form / edit buffer (single draft row)
    pub armor_modifier_value: String,
    pub armor_modifier_suit: String,
    pub armor_modifier_damage_type: String,

    // Stat modifiers form / edit buffer (single draft row)
    pub stat_modifier_value: String,
    pub stat_modifier_stat: String,
    pub stat_modifier_operator: String,

    // Pending stat modifiers (used by Item/Level create flows; linked on create)
    pub pending_stat_modifiers: Vec<app::cards::stat_modifier::StatModifier>,

    // Hex grid editor (UI state only)
    pub hex_grid_width: String,
    pub hex_grid_height: String,

    // Name + persisted id (for future list/edit/delete workflows)
    pub hex_grid_name: String,
    pub hex_grid_id: Option<i64>,

    // Loaded list of existing grids (for list/edit/delete UI)
    pub hex_grids: Vec<HexGridRow>,

    pub hex_grid_selected_x: Option<i32>,
    pub hex_grid_selected_y: Option<i32>,

    // Selected tile association editor buffers (UI-only; persisted on SaveHexTileAssociations)
    //
    // "Friendly" inputs: user types a name query; picks a match; we resolve to the card id.
    pub hex_tile_unit_query: String,
    pub hex_tile_item_query: String,
    pub hex_tile_level_query: String,

    // Resolved ids (derived from picked name; displayed in UI elsewhere)
    pub hex_tile_unit_id: Option<i64>,
    pub hex_tile_item_id: Option<i64>,
    pub hex_tile_level_id: Option<i64>,

    // The chosen names (for display). These are best-effort and may not be present if loaded by id only.
    pub hex_tile_unit_name: Option<String>,
    pub hex_tile_item_name: Option<String>,
    pub hex_tile_level_name: Option<String>,

    pub hex_tile_type: String,

    // Tile presence only: coordinates that currently contain a tile
    pub hex_grid_tiles_present: BTreeSet<(i32, i32)>,

    // Coordinates that have *any* persisted data/association (unit_id/item_id/level_id/type).
    // Used by the canvas to render "occupied" tiles differently.
    pub hex_grid_tiles_with_data: BTreeSet<(i32, i32)>,

    // Pending armor modifiers (used by create-card flows; linked on create)
    pub pending_armor_modifiers: Vec<app::cards::armor_modifier::ArmorModifier>,

    // loaded data
    pub units: Vec<UnitRow>,
    pub items: Vec<ItemRow>,
    pub levels: Vec<LevelRow>,
    pub actions: Vec<ActionRow>,
    pub armor_modifiers: Vec<ArmorModifierRow>,
    pub stat_modifiers: Vec<StatModifierRow>,

    // ui state
    pub status: Option<String>,
    pub active_view: ActiveView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexGridRow {
    pub id: i64,
    pub name: String,
    pub width: i32,
    pub height: i32,
}

impl Default for ToolsGui {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Robust defaults regardless of working directory:
        // - Prefer a directory at/above `cwd` that contains `migrations/` (workspace root).
        // - Fall back to `cwd` if none is found.
        let tabletop_dir = app::paths::default_tabletop_dir(&cwd).unwrap_or(cwd);

        Self {
            tab: Tab::Units,

            // Defaults are resolved under `tabletop_dir` (see `resolve_paths`).
            tabletop_dir,
            db_path: PathBuf::from("tabletop.sqlite3"),
            migrations_dir: PathBuf::from("migrations"),

            unit_name: String::new(),
            unit_strength: "0".to_string(),
            unit_focus: "0".to_string(),
            unit_intelligence: "0".to_string(),
            unit_agility: "0".to_string(),
            unit_knowledge: "0".to_string(),
            unit_assoc_action_name: String::new(),

            item_name: String::new(),
            item_assoc_action_name: String::new(),

            level_name: String::new(),
            level_text: String::new(),
            level_assoc_action_name: String::new(),

            action_name: String::new(),
            action_point_cost: "0".to_string(),
            action_type: "Interaction".to_string(),
            action_text: String::new(),

            attack_damage: "0".to_string(),
            attack_damage_type: "Physical".to_string(),
            attack_skill: "Strength".to_string(),
            attack_target: "1".to_string(),
            attack_range: "0".to_string(),

            interaction_range: "0".to_string(),
            interaction_skill: "Strength".to_string(),
            interaction_target: "".to_string(),

            armor_modifier_value: "0".to_string(),
            armor_modifier_suit: "Spades".to_string(),
            armor_modifier_damage_type: "Physical".to_string(),

            stat_modifier_value: "0".to_string(),
            stat_modifier_stat: "Strength".to_string(),
            stat_modifier_operator: "Add".to_string(),

            pending_stat_modifiers: vec![],

            hex_grid_width: "9".to_string(),
            hex_grid_height: "9".to_string(),

            hex_grid_name: "New Hex Grid".to_string(),
            hex_grid_id: None,

            hex_grids: vec![],

            hex_grid_selected_x: None,
            hex_grid_selected_y: None,

            hex_tile_unit_query: "".to_string(),
            hex_tile_item_query: "".to_string(),
            hex_tile_level_query: "".to_string(),

            hex_tile_unit_id: None,
            hex_tile_item_id: None,
            hex_tile_level_id: None,

            hex_tile_unit_name: None,
            hex_tile_item_name: None,
            hex_tile_level_name: None,

            hex_tile_type: "".to_string(),

            hex_grid_tiles_present: (|| {
                let mut s = BTreeSet::new();
                for y in 0..9_i32 {
                    for x in 0..9_i32 {
                        s.insert((x, y));
                    }
                }
                s
            })(),

            hex_grid_tiles_with_data: BTreeSet::new(),

            pending_armor_modifiers: vec![],

            units: vec![],
            items: vec![],
            levels: vec![],
            actions: vec![],
            armor_modifiers: vec![],
            stat_modifiers: vec![],

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
        let app = Self::default();

        // Run DB init/migrations as part of the startup refresh so it also runs
        // on any explicit refresh cycle.
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

                // When entering the Hex Grids tab, refresh the list.
                if self.tab == Tab::HexGrids {
                    return Command::perform(async {}, |_| Message::RefreshHexGrids);
                }

                Command::none()
            }

            // Stat modifiers form buffers
            Message::StatModifierValueChanged(v) => {
                self.stat_modifier_value = v;
                Command::none()
            }
            Message::StatModifierStatChanged(v) => {
                self.stat_modifier_stat = v;
                Command::none()
            }
            Message::StatModifierOperatorChanged(v) => {
                self.stat_modifier_operator = v;
                Command::none()
            }

            // Queue stat modifiers during create flows (Item/Level)
            Message::AddPendingStatModifier => {
                match self.create_stat_modifier_from_form() {
                    Ok(sm) => self.pending_stat_modifiers.push(sm),
                    Err(e) => self.status = Some(format!("{e:#}")),
                }
                Command::none()
            }
            Message::RemovePendingStatModifier(idx) => {
                if idx < self.pending_stat_modifiers.len() {
                    self.pending_stat_modifiers.remove(idx);
                }
                Command::none()
            }
            Message::ClearPendingStatModifiers => {
                self.pending_stat_modifiers.clear();
                Command::none()
            }

            // Create stat modifiers while editing cards
            Message::CreateUnitStatModifier => {
                if let Err(e) = self.create_stat_modifier_for_current_unit() {
                    self.status = Some(format!("{e:#}"));
                    return Command::none();
                }
                return Command::perform(async {}, |_| Message::Refresh);
            }
            Message::CreateItemStatModifier => {
                if let Err(e) = self.create_stat_modifier_for_current_item() {
                    self.status = Some(format!("{e:#}"));
                    return Command::none();
                }
                return Command::perform(async {}, |_| Message::Refresh);
            }
            Message::CreateLevelStatModifier => {
                if let Err(e) = self.create_stat_modifier_for_current_level() {
                    self.status = Some(format!("{e:#}"));
                    return Command::none();
                }
                return Command::perform(async {}, |_| Message::Refresh);
            }

            // Delete stat modifiers from card views
            Message::DeleteStatModifier(id) => {
                if let Err(e) = self.delete_stat_modifier(id) {
                    self.status = Some(format!("{e:#}"));
                    return Command::none();
                }
                return Command::perform(async {}, |_| Message::Refresh);
            }

            // Hex grid editor
            Message::HexGridWidthChanged(v) => {
                self.hex_grid_width = v;
                Command::none()
            }
            Message::HexGridHeightChanged(v) => {
                self.hex_grid_height = v;
                Command::none()
            }
            Message::HexGridNameChanged(v) => {
                self.hex_grid_name = v;
                Command::none()
            }
            Message::CreateNewHexGrid => {
                // Reset the editor to a new, unsaved grid.
                // Keep the current width/height inputs, but clear the saved id and tiles.
                let width: i32 = self.hex_grid_width.trim().parse().unwrap_or(9).max(1);
                let height: i32 = self.hex_grid_height.trim().parse().unwrap_or(9).max(1);

                self.hex_grid_id = None;
                self.hex_grid_name = "New Hex Grid".to_string();

                // Start fully populated (presence-only) for all in-bounds tiles.
                let mut s = BTreeSet::new();
                for y in 0..height {
                    for x in 0..width {
                        s.insert((x, y));
                    }
                }
                self.hex_grid_tiles_present = s;
                // Data presence is persisted per-tile in DB; refresh it when a grid is loaded/saved.
                self.hex_grid_tiles_with_data.clear();

                // Clear selection buffers
                self.hex_grid_selected_x = None;
                self.hex_grid_selected_y = None;

                // Clear selected-tile association editor buffers
                self.hex_tile_unit_query.clear();
                self.hex_tile_item_query.clear();
                self.hex_tile_level_query.clear();

                self.hex_tile_unit_id = None;
                self.hex_tile_item_id = None;
                self.hex_tile_level_id = None;

                self.hex_tile_unit_name = None;
                self.hex_tile_item_name = None;
                self.hex_tile_level_name = None;

                self.hex_tile_type.clear();

                Command::none()
            }
            Message::SaveHexGrid => {
                if let Err(e) = self.save_hex_grid_from_editor_state() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    let msg = match self.hex_grid_id {
                        Some(id) => format!("Hex grid saved (id={id})"),
                        None => "Hex grid saved".to_string(),
                    };
                    self.status = Some(msg);
                    Command::perform(async {}, |_| Message::RefreshHexGrids)
                }
            }

            // Hex grid list/load
            Message::RefreshHexGrids => {
                if let Err(e) = self.refresh_hex_grids_list() {
                    self.status = Some(format!("{e:#}"));
                }
                Command::none()
            }
            Message::LoadHexGridById(id) => {
                if let Err(e) = self.load_hex_grid_into_editor(id) {
                    self.status = Some(format!("{e:#}"));
                }
                Command::none()
            }
            Message::DeleteHexGridById(id) => {
                if let Err(e) = self.delete_hex_grid_by_id(id) {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    // If we're currently editing this grid, clear the editor id.
                    if self.hex_grid_id == Some(id) {
                        self.hex_grid_id = None;
                    }
                    Command::perform(async {}, |_| Message::RefreshHexGrids)
                }
            }

            Message::HexGridApplyResize => {
                // Keep any tiles still in-bounds; drop anything out of bounds.
                //
                // Resizing behavior:
                // - Preserve explicit deletions for tiles that were already in-bounds before.
                // - Auto-populate any *newly in-bounds* coordinates with a tile (presence-only).
                //
                // IMPORTANT: parse "old" dimensions before we overwrite the input fields.
                let old_w = self
                    .hex_grid_width
                    .trim()
                    .parse::<i32>()
                    .ok()
                    .filter(|v| *v > 0);
                let old_h = self
                    .hex_grid_height
                    .trim()
                    .parse::<i32>()
                    .ok()
                    .filter(|v| *v > 0);

                let new_w = self
                    .hex_grid_width
                    .trim()
                    .parse::<i32>()
                    .ok()
                    .filter(|v| *v > 0);
                let new_h = self
                    .hex_grid_height
                    .trim()
                    .parse::<i32>()
                    .ok()
                    .filter(|v| *v > 0);

                if let (Some(new_w), Some(new_h)) = (new_w, new_h) {
                    let old_w = old_w.unwrap_or(new_w);
                    let old_h = old_h.unwrap_or(new_h);

                    // First, prune tiles that are now out of bounds.
                    self.hex_grid_tiles_present
                        .retain(|(x, y)| *x >= 0 && *y >= 0 && *x < new_w && *y < new_h);

                    // Keep "with data" set in-bounds too.
                    self.hex_grid_tiles_with_data
                        .retain(|(x, y)| *x >= 0 && *y >= 0 && *x < new_w && *y < new_h);

                    // Then, auto-populate tiles that are newly in-bounds due to the resize.
                    // This preserves deletions for tiles that were already in-bounds.
                    for y in 0..new_h {
                        for x in 0..new_w {
                            let was_in_old_bounds = x < old_w && y < old_h;
                            if !was_in_old_bounds {
                                self.hex_grid_tiles_present.insert((x, y));
                            }
                        }
                    }

                    // If selection is now out of bounds, clear it.
                    if let (Some(x), Some(y)) = (self.hex_grid_selected_x, self.hex_grid_selected_y)
                    {
                        if x < 0 || y < 0 || x >= new_w || y >= new_h {
                            self.hex_grid_selected_x = None;
                            self.hex_grid_selected_y = None;
                        }
                    }

                    // Finally, keep the editor inputs consistent with what we applied.
                    self.hex_grid_width = new_w.to_string();
                    self.hex_grid_height = new_h.to_string();
                } else {
                    self.status = Some("Width/height must be positive integers".to_string());
                }

                Command::none()
            }
            Message::HexGridTileClicked(x, y) => {
                self.hex_grid_selected_x = Some(x);
                self.hex_grid_selected_y = Some(y);

                // Left-clicking an empty coordinate creates a tile (presence-only).
                self.hex_grid_tiles_present.insert((x, y));

                // Load associations for this tile into the editor buffers (best-effort).
                if let Some(grid_id) = self.hex_grid_id {
                    if let Ok(conn) = self.open_conn() {
                        let row: rusqlite::Result<(
                            Option<i64>,
                            Option<i64>,
                            Option<i64>,
                            Option<String>,
                        )> = conn.query_row(
                            r#"
                                SELECT unit_id, item_id, level_id, type
                                FROM hex_tiles
                                WHERE hex_grid_id = ?1 AND x = ?2 AND y = ?3
                                "#,
                            rusqlite::params![grid_id, x, y],
                            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                        );

                        match row {
                            Ok((unit_id, item_id, level_id, tile_type)) => {
                                self.hex_tile_unit_id = unit_id;
                                self.hex_tile_item_id = item_id;
                                self.hex_tile_level_id = level_id;

                                self.hex_tile_unit_name = None;
                                self.hex_tile_item_name = None;
                                self.hex_tile_level_name = None;

                                self.hex_tile_unit_query.clear();
                                self.hex_tile_item_query.clear();
                                self.hex_tile_level_query.clear();

                                self.hex_tile_type = tile_type.unwrap_or_default();
                            }
                            Err(_) => {
                                // Tile might not exist yet (unsaved/new). Keep buffers as-is.
                            }
                        }
                    }
                }

                Command::none()
            }
            Message::HexGridTileClear(x, y) => {
                // Right-click deletes a tile from the grid (make this coordinate empty).
                self.hex_grid_tiles_present.remove(&(x, y));
                self.hex_grid_tiles_with_data.remove(&(x, y));

                if self.hex_grid_selected_x == Some(x) && self.hex_grid_selected_y == Some(y) {
                    self.hex_grid_selected_x = None;
                    self.hex_grid_selected_y = None;

                    // Clear selected-tile association editor buffers
                    self.hex_tile_unit_query.clear();
                    self.hex_tile_item_query.clear();
                    self.hex_tile_level_query.clear();

                    self.hex_tile_unit_id = None;
                    self.hex_tile_item_id = None;
                    self.hex_tile_level_id = None;

                    self.hex_tile_unit_name = None;
                    self.hex_tile_item_name = None;
                    self.hex_tile_level_name = None;

                    self.hex_tile_type.clear();
                }

                Command::none()
            }

            Message::HexTileUnitQueryChanged(v) => {
                self.hex_tile_unit_query = v;

                // Enforce "only ONE of Unit/Item/Level/Type"
                if !self.hex_tile_unit_query.trim().is_empty() {
                    self.hex_tile_item_query.clear();
                    self.hex_tile_level_query.clear();
                    self.hex_tile_type.clear();

                    self.hex_tile_item_id = None;
                    self.hex_tile_level_id = None;

                    self.hex_tile_item_name = None;
                    self.hex_tile_level_name = None;
                }

                Command::none()
            }
            Message::HexTileItemQueryChanged(v) => {
                self.hex_tile_item_query = v;

                // Enforce "only ONE of Unit/Item/Level/Type"
                if !self.hex_tile_item_query.trim().is_empty() {
                    self.hex_tile_unit_query.clear();
                    self.hex_tile_level_query.clear();
                    self.hex_tile_type.clear();

                    self.hex_tile_unit_id = None;
                    self.hex_tile_level_id = None;

                    self.hex_tile_unit_name = None;
                    self.hex_tile_level_name = None;
                }

                Command::none()
            }
            Message::HexTileLevelQueryChanged(v) => {
                self.hex_tile_level_query = v;

                // Enforce "only ONE of Unit/Item/Level/Type"
                if !self.hex_tile_level_query.trim().is_empty() {
                    self.hex_tile_unit_query.clear();
                    self.hex_tile_item_query.clear();
                    self.hex_tile_type.clear();

                    self.hex_tile_unit_id = None;
                    self.hex_tile_item_id = None;

                    self.hex_tile_unit_name = None;
                    self.hex_tile_item_name = None;
                }

                Command::none()
            }

            Message::HexTilePickUnitByName(name) => {
                let picked = self
                    .units
                    .iter()
                    .find(|u| u.name == name)
                    .map(|u| u.name.clone());
                if let Some(picked) = picked {
                    // Enforce "only ONE of Unit/Item/Level/Type"
                    self.hex_tile_item_query.clear();
                    self.hex_tile_level_query.clear();
                    self.hex_tile_type.clear();

                    self.hex_tile_item_id = None;
                    self.hex_tile_level_id = None;

                    self.hex_tile_item_name = None;
                    self.hex_tile_level_name = None;

                    // NOTE: we resolve id at save time via DB lookup by name.
                    self.hex_tile_unit_name = Some(picked.clone());
                    self.hex_tile_unit_query = picked;
                }
                Command::none()
            }
            Message::HexTilePickItemByName(name) => {
                let picked = self
                    .items
                    .iter()
                    .find(|i| i.name == name)
                    .map(|i| i.name.clone());
                if let Some(picked) = picked {
                    // Enforce "only ONE of Unit/Item/Level/Type"
                    self.hex_tile_unit_query.clear();
                    self.hex_tile_level_query.clear();
                    self.hex_tile_type.clear();

                    self.hex_tile_unit_id = None;
                    self.hex_tile_level_id = None;

                    self.hex_tile_unit_name = None;
                    self.hex_tile_level_name = None;

                    self.hex_tile_item_name = Some(picked.clone());
                    self.hex_tile_item_query = picked;
                }
                Command::none()
            }
            Message::HexTilePickLevelByName(name) => {
                let picked = self
                    .levels
                    .iter()
                    .find(|l| l.name == name)
                    .map(|l| l.name.clone());
                if let Some(picked) = picked {
                    // Enforce "only ONE of Unit/Item/Level/Type"
                    self.hex_tile_unit_query.clear();
                    self.hex_tile_item_query.clear();
                    self.hex_tile_type.clear();

                    self.hex_tile_unit_id = None;
                    self.hex_tile_item_id = None;

                    self.hex_tile_unit_name = None;
                    self.hex_tile_item_name = None;

                    self.hex_tile_level_name = Some(picked.clone());
                    self.hex_tile_level_query = picked;
                }
                Command::none()
            }

            Message::HexTileTypeChanged(v) => {
                self.hex_tile_type = v;

                // Enforce "only ONE of Unit/Item/Level/Type"
                if !self.hex_tile_type.trim().is_empty() {
                    self.hex_tile_unit_query.clear();
                    self.hex_tile_item_query.clear();
                    self.hex_tile_level_query.clear();

                    self.hex_tile_unit_id = None;
                    self.hex_tile_item_id = None;
                    self.hex_tile_level_id = None;

                    self.hex_tile_unit_name = None;
                    self.hex_tile_item_name = None;
                    self.hex_tile_level_name = None;
                }

                Command::none()
            }
            Message::ClearHexTileAssociations => {
                self.hex_tile_unit_query.clear();
                self.hex_tile_item_query.clear();
                self.hex_tile_level_query.clear();

                self.hex_tile_unit_id = None;
                self.hex_tile_item_id = None;
                self.hex_tile_level_id = None;

                self.hex_tile_unit_name = None;
                self.hex_tile_item_name = None;
                self.hex_tile_level_name = None;

                self.hex_tile_type.clear();
                Command::none()
            }
            Message::SaveHexTileAssociations => {
                let Some(grid_id) = self.hex_grid_id else {
                    self.status = Some("Save the hex grid first (no grid id yet)".to_string());
                    return Command::none();
                };
                let (Some(x), Some(y)) = (self.hex_grid_selected_x, self.hex_grid_selected_y)
                else {
                    self.status = Some("Select a tile first".to_string());
                    return Command::none();
                };

                // Ensure tile exists (presence-only) before associating.
                self.hex_grid_tiles_present.insert((x, y));

                // Enforce "only ONE of Unit/Item/Level/Type"
                let unit_input = !self.hex_tile_unit_query.trim().is_empty();
                let item_input = !self.hex_tile_item_query.trim().is_empty();
                let level_input = !self.hex_tile_level_query.trim().is_empty();
                let type_input = !self.hex_tile_type.trim().is_empty();

                let chosen_count = (unit_input as i32)
                    + (item_input as i32)
                    + (level_input as i32)
                    + (type_input as i32);

                if chosen_count > 1 {
                    self.status = Some(
                        "Choose only ONE association: Unit OR Level OR Item OR Type (clear the others first)".to_string(),
                    );
                    return Command::none();
                }

                let tile_type = {
                    let t = self.hex_tile_type.trim();
                    if t.is_empty() {
                        None
                    } else {
                        Some(t.to_string())
                    }
                };

                match self.open_conn() {
                    Ok(mut conn) => {
                        let tx = match conn.transaction() {
                            Ok(tx) => tx,
                            Err(e) => {
                                self.status = Some(format!("{e:#}"));
                                return Command::none();
                            }
                        };

                        // Resolve ids by name query (exact match on canonical name).
                        // Empty query => NULL
                        let unit_id: Option<i64> = {
                            let q = self.hex_tile_unit_query.trim();
                            if q.is_empty() {
                                None
                            } else {
                                tx.query_row(
                                    r#"SELECT id FROM units WHERE name = ?1"#,
                                    rusqlite::params![q],
                                    |r| r.get(0),
                                )
                                .optional()
                                .map_err(|e| anyhow::anyhow!("{e}"))
                                .unwrap_or(None)
                            }
                        };

                        let item_id: Option<i64> = {
                            let q = self.hex_tile_item_query.trim();
                            if q.is_empty() {
                                None
                            } else {
                                tx.query_row(
                                    r#"SELECT id FROM items WHERE name = ?1"#,
                                    rusqlite::params![q],
                                    |r| r.get(0),
                                )
                                .optional()
                                .map_err(|e| anyhow::anyhow!("{e}"))
                                .unwrap_or(None)
                            }
                        };

                        let level_id: Option<i64> = {
                            let q = self.hex_tile_level_query.trim();
                            if q.is_empty() {
                                None
                            } else {
                                tx.query_row(
                                    r#"SELECT id FROM levels WHERE name = ?1"#,
                                    rusqlite::params![q],
                                    |r| r.get(0),
                                )
                                .optional()
                                .map_err(|e| anyhow::anyhow!("{e}"))
                                .unwrap_or(None)
                            }
                        };

                        let resolved_count = (unit_id.is_some() as i32)
                            + (item_id.is_some() as i32)
                            + (level_id.is_some() as i32)
                            + (tile_type.is_some() as i32);

                        if resolved_count > 1 {
                            self.status = Some(
                                "Choose only ONE association: Unit OR Level OR Item OR Type (clear the others first)".to_string(),
                            );
                            return Command::none();
                        }

                        // Ensure row exists
                        if let Err(e) = tx.execute(
                            r#"
                            INSERT INTO hex_tiles (hex_grid_id, x, y)
                            VALUES (?1, ?2, ?3)
                            ON CONFLICT(hex_grid_id, x, y)
                            DO UPDATE SET
                                updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                            "#,
                            rusqlite::params![grid_id, x, y],
                        ) {
                            self.status = Some(format!("{e:#}"));
                            return Command::none();
                        }

                        if let Err(e) = tx.execute(
                            r#"
                            UPDATE hex_tiles
                            SET unit_id = ?1,
                                item_id = ?2,
                                level_id = ?3,
                                type = ?4,
                                updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                            WHERE hex_grid_id = ?5 AND x = ?6 AND y = ?7
                            "#,
                            rusqlite::params![unit_id, item_id, level_id, tile_type, grid_id, x, y],
                        ) {
                            self.status = Some(format!("{e:#}"));
                            return Command::none();
                        }

                        if let Err(e) = tx.commit() {
                            self.status = Some(format!("{e:#}"));
                            return Command::none();
                        }

                        // Cache resolved ids in UI state
                        self.hex_tile_unit_id = unit_id;
                        self.hex_tile_item_id = item_id;
                        self.hex_tile_level_id = level_id;

                        // Track whether this tile now has any persisted association/data.
                        let has_data = unit_id.is_some()
                            || item_id.is_some()
                            || level_id.is_some()
                            || tile_type.is_some();

                        if has_data {
                            self.hex_grid_tiles_with_data.insert((x, y));
                        } else {
                            self.hex_grid_tiles_with_data.remove(&(x, y));
                        }

                        self.status = Some("Hex tile associations saved".to_string());
                        Command::none()
                    }
                    Err(e) => {
                        self.status = Some(format!("{e:#}"));
                        Command::none()
                    }
                }
            }

            Message::ArmorModifierValueChanged(v) => {
                self.armor_modifier_value = v;
                Command::none()
            }
            Message::ArmorModifierSuitChanged(v) => {
                self.armor_modifier_suit = v;
                Command::none()
            }
            Message::ArmorModifierDamageTypeChanged(v) => {
                self.armor_modifier_damage_type = v;
                Command::none()
            }

            Message::AddPendingArmorModifier => {
                if let Err(e) = self.add_pending_armor_modifier_from_form() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Armor modifier queued".to_string());
                    self.armor_modifier_value = "0".to_string();
                    self.armor_modifier_suit = "Spades".to_string();
                    self.armor_modifier_damage_type = "Physical".to_string();
                }
                Command::none()
            }
            Message::RemovePendingArmorModifier(idx) => {
                if idx < self.pending_armor_modifiers.len() {
                    self.pending_armor_modifiers.remove(idx);
                }
                Command::none()
            }
            Message::ClearPendingArmorModifiers => {
                self.pending_armor_modifiers.clear();
                Command::none()
            }

            Message::CreateArmorModifier => {
                // Create + associate in one step, based on the current edit context.
                if let Err(e) = self.create_armor_modifier_from_form() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Armor modifier created".to_string());
                    self.armor_modifier_value = "0".to_string();
                    self.armor_modifier_suit = "Spades".to_string();
                    self.armor_modifier_damage_type = "Physical".to_string();
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::RemoveArmorModifierLink(id) => {
                if let Err(e) = self.remove_armor_modifier_link(id) {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some(format!("Removed armor modifier link (id={id})"));
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }

            Message::EditArmorModifier(id) => {
                if let Err(e) = self.begin_edit_armor_modifier(id) {
                    self.status = Some(format!("{e:#}"));
                }
                Command::none()
            }

            Message::SaveArmorModifierEdits => {
                if let Err(e) = self.save_armor_modifier_edits() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Armor modifier updated".to_string());
                    self.active_view = ActiveView::List;
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }

            Message::DeleteArmorModifier(id) => {
                if let Err(e) = self.delete_armor_modifier(id) {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some(format!("Deleted armor modifier id={id}"));
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::Refresh => {
                // Ensure schema + migrations are applied on every refresh cycle,
                // including the first startup refresh.
                if let Err(e) = self.ensure_db_ready() {
                    self.status = Some(format!("DB init failed: {e:#}"));
                    return Command::none();
                }

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

            Message::UnitAssocActionNameChanged(v) => {
                self.unit_assoc_action_name = v;
                Command::none()
            }
            Message::AddUnitAssociation => {
                if let Err(e) = self.save_unit_association() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Unit association added".to_string());
                    self.unit_assoc_action_name.clear();
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }
            Message::RemoveUnitAssociation(action_name) => {
                if let Err(e) = self.remove_unit_association(&action_name) {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Unit association removed".to_string());
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }

            Message::CreateUnitAndMaybeAssociate => {
                if let Err(e) = self.create_unit_and_maybe_associate_from_form() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Unit created".to_string());
                    self.unit_name.clear();
                    self.unit_assoc_action_name.clear();
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

            Message::CreateItemAndMaybeAssociate => {
                if let Err(e) = self.create_item_and_maybe_associate_from_form() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Item created".to_string());
                    self.item_name.clear();
                    self.item_assoc_action_name.clear();
                    self.pending_armor_modifiers.clear();
                    self.pending_stat_modifiers.clear();
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::ItemAssocActionNameChanged(v) => {
                self.item_assoc_action_name = v;
                Command::none()
            }
            Message::AddItemAssociation => {
                if let Err(e) = self.save_item_association() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Item association added".to_string());
                    self.item_assoc_action_name.clear();
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }
            Message::RemoveItemAssociation(action_name) => {
                if let Err(e) = self.remove_item_association(&action_name) {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Item association removed".to_string());
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }

            Message::LevelNameChanged(v) => {
                self.level_name = v;
                Command::none()
            }
            Message::LevelTextChanged(v) => {
                self.level_text = v;
                Command::none()
            }
            Message::CreateLevel => {
                if let Err(e) = self.create_level_from_form() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Level created".to_string());
                    self.level_name.clear();
                    self.level_text.clear();
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::CreateLevelAndMaybeAssociate => {
                if let Err(e) = self.create_level_and_maybe_associate_from_form() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Level created".to_string());
                    self.level_name.clear();
                    self.level_text.clear();
                    self.level_assoc_action_name.clear();
                    self.pending_armor_modifiers.clear();
                    self.pending_stat_modifiers.clear();
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::LevelAssocActionNameChanged(v) => {
                self.level_assoc_action_name = v;
                Command::none()
            }
            Message::AddLevelAssociation => {
                if let Err(e) = self.save_level_association() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Level association added".to_string());
                    self.level_assoc_action_name.clear();
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }
            Message::RemoveLevelAssociation(action_name) => {
                if let Err(e) = self.remove_level_association(&action_name) {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Level association removed".to_string());
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }

            Message::ActionNameChanged(v) => {
                self.action_name = v;
                Command::none()
            }
            Message::ActionPointCostChanged(v) => {
                self.action_point_cost = v;
                Command::none()
            }
            Message::ActionTypeChanged(v) => {
                self.action_type = v;
                Command::none()
            }
            Message::ActionTextChanged(v) => {
                self.action_text = v;
                Command::none()
            }
            Message::CreateAction => {
                if let Err(e) = self.create_action_with_subtype_from_form() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Action created".to_string());
                    self.action_name.clear();
                    self.action_text.clear();
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::AttackDamageChanged(v) => {
                self.attack_damage = v;
                Command::none()
            }
            Message::AttackDamageTypeChanged(v) => {
                self.attack_damage_type = v;
                Command::none()
            }
            Message::AttackSkillChanged(v) => {
                self.attack_skill = v;
                Command::none()
            }
            Message::AttackTargetChanged(v) => {
                self.attack_target = v;
                Command::none()
            }
            Message::AttackRangeChanged(v) => {
                self.attack_range = v;
                Command::none()
            }
            Message::SaveAttackEdits => {
                if let Err(e) = self.save_attack_edits() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Attack saved".to_string());
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }
            Message::DeleteAttack => {
                if let Err(e) = self.delete_attack_for_current_action() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Attack deleted".to_string());
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::InteractionRangeChanged(v) => {
                self.interaction_range = v;
                Command::none()
            }
            Message::InteractionSkillChanged(v) => {
                self.interaction_skill = v;
                Command::none()
            }
            Message::InteractionTargetChanged(v) => {
                self.interaction_target = v;
                Command::none()
            }
            Message::SaveInteractionEdits => {
                if let Err(e) = self.save_interaction_edits() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Interaction saved".to_string());
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }
            Message::DeleteInteraction => {
                if let Err(e) = self.delete_interaction_for_current_action() {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some("Interaction deleted".to_string());
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

            Message::EditLevel(name) => {
                if let Err(e) = self.begin_edit_level(&name) {
                    self.status = Some(format!("{e:#}"));
                }
                Command::none()
            }

            Message::EditAction(name) => {
                // When navigating to an Action from another tab (e.g. Unit/Item/Level details),
                // make sure the Actions tab is selected so the edit view is actually visible.
                self.tab = Tab::Actions;

                if let Err(e) = self.begin_edit_action(&name) {
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

            Message::SaveLevelEdits => {
                if let Err(e) = self.save_level_edits() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Level updated".to_string());
                    self.active_view = ActiveView::List;
                    Command::perform(async {}, |_| Message::Refresh)
                }
            }

            Message::SaveActionEdits => {
                if let Err(e) = self.save_action_edits() {
                    self.status = Some(format!("{e:#}"));
                    Command::none()
                } else {
                    self.status = Some("Action updated".to_string());
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

            Message::DeleteLevel(name) => {
                if let Err(e) = self.delete_level(&name) {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some(format!("Deleted level `{name}`"));
                }
                Command::perform(async {}, |_| Message::Refresh)
            }

            Message::DeleteAction(name) => {
                if let Err(e) = self.delete_action(&name) {
                    self.status = Some(format!("{e:#}"));
                } else {
                    self.status = Some(format!("Deleted action `{name}`"));
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
            views::tab_button(self.tab, Tab::Levels),
            views::tab_button(self.tab, Tab::Actions),
            views::tab_button(self.tab, Tab::ArmorModifiers),
            views::tab_button(self.tab, Tab::HexGrids),
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
            Tab::Levels => match &self.active_view {
                ActiveView::List => views::levels::view(self),
                ActiveView::EditLevel { original_name } => {
                    views::levels::edit_view(self, original_name)
                }
                _ => views::levels::view(self),
            },
            Tab::Actions => match &self.active_view {
                ActiveView::List => views::actions::view(self),
                ActiveView::EditAction { original_name } => {
                    views::actions::edit_view(self, original_name)
                }
                _ => views::actions::view(self),
            },
            Tab::ArmorModifiers => match &self.active_view {
                ActiveView::List => views::armor_modifiers::view(self),
                ActiveView::EditArmorModifier { id } => {
                    views::armor_modifiers::edit_view(self, *id)
                }
                _ => views::armor_modifiers::view(self),
            },
            Tab::HexGrids => views::hex_grids::view(self),
        };

        container(
            column![
                header,
                tabs,
                horizontal_rule(1),
                status,
                horizontal_rule(1),
                // The Hex Grids view manages its own scrolling; wrapping it in an outer scrollable
                // causes "infinite" scroll behavior due to Fill-sized canvas/layout interactions.
                if self.tab == Tab::HexGrids {
                    content
                } else {
                    iced::widget::scrollable(content)
                        .height(Length::Fill)
                        .into()
                },
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
        // Robust path resolution regardless of the process working directory:
        // - If `tabletop_dir` is absolute, use it as-is.
        // - If `tabletop_dir` is relative, interpret it relative to a discovered workspace root
        //   (a directory at/above the current working directory that contains `migrations/`).
        let tabletop_dir = app::paths::normalize_dir(&self.tabletop_dir)
            .with_context(|| format!("Invalid tabletop dir: {}", self.tabletop_dir.display()))?;

        let db_path = app::paths::resolve_under_workspace_root(&tabletop_dir, &self.db_path)
            .with_context(|| format!("Invalid db path: {}", self.db_path.display()))?;

        let migrations_dir =
            app::paths::resolve_under_workspace_root(&tabletop_dir, &self.migrations_dir)
                .with_context(|| {
                    format!(
                        "Invalid migrations dir path: {}",
                        self.migrations_dir.display()
                    )
                })?;

        Ok((tabletop_dir, db_path, migrations_dir))
    }

    fn create_armor_modifier_from_form(&self) -> Result<()> {
        // CardId is not user-editable. It is derived from the current edit context:
        // - Item edit: card_id = items.id, and we link to the item
        // - Level edit: card_id = levels.id, and we link to the level
        //
        // For create-card flows, armor modifiers are queued via `AddPendingArmorModifier` and
        // persisted+linked when the card is created.
        let mut conn = self.open_conn()?;

        let value =
            shared::form_parsing::parse_i64_required("Value", self.armor_modifier_value.trim())?;

        let suit = app::cards::armor_modifier::Suit::parse(self.armor_modifier_suit.trim())
            .context("Invalid Suit (expected Spades, Clubs, Diamonds, or Hearts)")?;

        let damage_type =
            app::cards::armor_modifier::DamageType::parse(self.armor_modifier_damage_type.trim())
                .context("Invalid DamageType (expected Arcane or Physical)")?;

        match &self.active_view {
            ActiveView::EditItem { original_name } => {
                // Derive CardId from the DB item row id.
                let item_id: i64 = conn
                    .query_row(
                        r#"
                        SELECT id
                        FROM items
                        WHERE name = ?1
                        "#,
                        rusqlite::params![original_name],
                        |r| r.get(0),
                    )
                    .with_context(|| format!("Lookup item id for `{original_name}`"))?;

                let armor = app::cards::armor_modifier::ArmorModifier {
                    card_id: item_id,
                    value,
                    suit,
                    damage_type,
                };

                // Create + link (allows multiple per card).
                app::cards::armor_modifier::create_and_link_to_item_by_name(
                    &mut conn,
                    &armor,
                    original_name,
                )?;

                Ok(())
            }

            ActiveView::EditLevel { original_name } => {
                // Derive CardId from the DB level row id.
                let level_id: i64 = conn
                    .query_row(
                        r#"
                        SELECT id
                        FROM levels
                        WHERE name = ?1
                        "#,
                        rusqlite::params![original_name],
                        |r| r.get(0),
                    )
                    .with_context(|| format!("Lookup level id for `{original_name}`"))?;

                let armor = app::cards::armor_modifier::ArmorModifier {
                    card_id: level_id,
                    value,
                    suit,
                    damage_type,
                };

                // Create + link (allows multiple per card).
                app::cards::armor_modifier::create_and_link_to_level_by_name(
                    &mut conn,
                    &armor,
                    original_name,
                )?;

                Ok(())
            }

            _ => bail!(
                "From create-card views, queue armor modifiers with 'AddPendingArmorModifier' and they will be saved when the card is created"
            ),
        }
    }

    fn list_stat_modifiers(&self, conn: &rusqlite::Connection) -> Result<Vec<StatModifierRow>> {
        let rows = app::cards::stat_modifier::list_all(conn)?;
        Ok(rows
            .into_iter()
            .map(|r| StatModifierRow {
                id: r.id,
                stat: r.stat,
                value: r.value,
                operator: r.operator,
                unit_name: r.unit_name,
                item_name: r.item_name,
                level_name: r.level_name,
            })
            .collect())
    }

    fn create_stat_modifier_from_form(&self) -> Result<app::cards::stat_modifier::StatModifier> {
        let stat = parse_stat_choice(&self.stat_modifier_stat).with_context(
            || "Stat is required (Strength, Focus, Intelligence, Knowledge, or Agility)",
        )?;

        let operator = parse_stat_operator_choice(&self.stat_modifier_operator)
            .with_context(|| "Operator is required (Add or Subtract)")?;

        let value = parse_optional_i64_from_input("Value", &self.stat_modifier_value)?
            .ok_or_else(|| anyhow::anyhow!("Value must be a number"))?;

        Ok(app::cards::stat_modifier::StatModifier {
            stat,
            value,
            operator,
        })
    }

    fn create_stat_modifier_for_current_unit(&mut self) -> Result<()> {
        let original_name = match &self.active_view {
            ActiveView::EditUnit { original_name } => original_name.clone(),
            _ => return Err(anyhow::anyhow!("not editing a unit")),
        };

        let mut conn = self.open_conn()?;
        let sm = self.create_stat_modifier_from_form()?;

        app::cards::stat_modifier::create_and_link_to_unit_by_name(&mut conn, &sm, &original_name)?;
        Ok(())
    }

    fn create_stat_modifier_for_current_item(&mut self) -> Result<()> {
        let original_name = match &self.active_view {
            ActiveView::EditItem { original_name } => original_name.clone(),
            _ => return Err(anyhow::anyhow!("not editing an item")),
        };

        let mut conn = self.open_conn()?;
        let sm = self.create_stat_modifier_from_form()?;

        app::cards::stat_modifier::create_and_link_to_item_by_name(&mut conn, &sm, &original_name)?;
        Ok(())
    }

    fn create_stat_modifier_for_current_level(&mut self) -> Result<()> {
        let original_name = match &self.active_view {
            ActiveView::EditLevel { original_name } => original_name.clone(),
            _ => return Err(anyhow::anyhow!("not editing a level")),
        };

        let mut conn = self.open_conn()?;
        let sm = self.create_stat_modifier_from_form()?;

        app::cards::stat_modifier::create_and_link_to_level_by_name(
            &mut conn,
            &sm,
            &original_name,
        )?;
        Ok(())
    }

    fn delete_stat_modifier(&mut self, id: i64) -> Result<()> {
        let conn = self.open_conn()?;
        app::cards::stat_modifier::delete_by_id(&conn, id)?;
        Ok(())
    }

    fn begin_edit_armor_modifier(&mut self, id: i64) -> Result<()> {
        let conn = self.open_conn()?;

        let Some(row) = app::cards::armor_modifier::get_by_id(&conn, id)? else {
            bail!("Armor modifier not found (id={id})");
        };

        self.armor_modifier_value = row.value.to_string();
        self.armor_modifier_suit = row.suit.as_str().to_string();
        self.armor_modifier_damage_type = row.damage_type.as_str().to_string();

        self.active_view = ActiveView::EditArmorModifier { id };
        Ok(())
    }

    fn save_armor_modifier_edits(&self) -> Result<()> {
        let conn = self.open_conn()?;

        let ActiveView::EditArmorModifier { id } = &self.active_view else {
            bail!("Not currently editing an armor modifier");
        };

        let value =
            shared::form_parsing::parse_i64_required("Value", self.armor_modifier_value.trim())?;

        let suit = app::cards::armor_modifier::Suit::parse(self.armor_modifier_suit.trim())
            .context("Invalid Suit (expected Spades, Clubs, Diamonds, or Hearts)")?;

        let damage_type =
            app::cards::armor_modifier::DamageType::parse(self.armor_modifier_damage_type.trim())
                .context("Invalid DamageType (expected Arcane or Physical)")?;

        // Preserve the existing CardId (non-editable).
        let Some(existing) = app::cards::armor_modifier::get_by_id(&conn, *id)? else {
            bail!("Armor modifier not found (id={id})");
        };

        let armor = app::cards::armor_modifier::ArmorModifier {
            card_id: existing.card_id,
            value,
            suit,
            damage_type,
        };

        app::cards::armor_modifier::update_by_id(&conn, *id, &armor)?;
        Ok(())
    }

    fn delete_armor_modifier(&self, id: i64) -> Result<()> {
        let conn = self.open_conn()?;
        app::cards::armor_modifier::delete_by_id(&conn, id)?;
        Ok(())
    }

    fn remove_armor_modifier_link(&self, id: i64) -> Result<()> {
        let conn = self.open_conn()?;
        app::cards::armor_modifier::clear_association(&conn, id)?;
        Ok(())
    }

    fn add_pending_armor_modifier_from_form(&mut self) -> Result<()> {
        // In create-card views, we don't have an item/level id yet.
        // Queue the modifier with a placeholder CardId; it will be overwritten at persist time
        // by `create_and_link_to_*` (which derives CardId from the linked card).
        let value =
            shared::form_parsing::parse_i64_required("Value", self.armor_modifier_value.trim())?;
        let suit = app::cards::armor_modifier::Suit::parse(self.armor_modifier_suit.trim())
            .context("Invalid Suit (expected Spades, Clubs, Diamonds, or Hearts)")?;

        let damage_type =
            app::cards::armor_modifier::DamageType::parse(self.armor_modifier_damage_type.trim())
                .context("Invalid DamageType (expected Arcane or Physical)")?;

        self.pending_armor_modifiers
            .push(app::cards::armor_modifier::ArmorModifier {
                card_id: 0,
                value,
                suit,
                damage_type,
            });

        Ok(())
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
        // Ensure schema + migrations are applied before returning a connection.
        // This prevents "no such table units" if any code path opens a connection
        // without going through `Message::Refresh` first.
        self.ensure_db_ready()?;

        let (_tabletop_dir, db_path, _migrations_dir) = self.resolve_paths()?;
        app::db::open_db(&db_path).with_context(|| format!("open db: {}", db_path.display()))
    }

    fn refresh_lists(&mut self) -> Result<()> {
        let conn = self.open_conn()?;

        self.units = views::units::list_units(&conn)?;
        self.items = views::items::list_items(&conn)?;
        self.levels = views::levels::list_levels(&conn)?;
        self.actions = views::actions::list_actions(&conn)?;
        self.armor_modifiers = views::armor_modifiers::list_armor_modifiers(&conn)?;

        // Stat modifiers are displayed within Unit/Item/Level edit views, so we keep them loaded.
        self.stat_modifiers = self.list_stat_modifiers(&conn)?;

        Ok(())
    }

    fn refresh_hex_grids_list(&mut self) -> Result<()> {
        let conn = self.open_conn()?;

        // Requires migration 0009 (hex_grids.name).
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, width, height
            FROM hex_grids
            ORDER BY updated_at DESC, id DESC
            "#,
        )?;

        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(r) = rows.next()? {
            out.push(HexGridRow {
                id: r.get::<_, i64>(0)?,
                name: r.get::<_, String>(1)?,
                width: r.get::<_, i32>(2)?,
                height: r.get::<_, i32>(3)?,
            });
        }

        self.hex_grids = out;
        Ok(())
    }

    fn load_hex_grid_into_editor(&mut self, id: i64) -> Result<()> {
        let conn = self.open_conn()?;

        // Load grid metadata
        let (name, width, height): (String, i32, i32) = conn.query_row(
            r#"
            SELECT name, width, height
            FROM hex_grids
            WHERE id = ?1
            "#,
            rusqlite::params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;

        // Load tile presence + "has data" marker set
        let mut stmt = conn.prepare(
            r#"
            SELECT
                x,
                y,
                unit_id,
                item_id,
                level_id,
                type
            FROM hex_tiles
            WHERE hex_grid_id = ?1
            "#,
        )?;

        let mut present: BTreeSet<(i32, i32)> = BTreeSet::new();
        let mut with_data: BTreeSet<(i32, i32)> = BTreeSet::new();

        let mut rows = stmt.query(rusqlite::params![id])?;
        while let Some(r) = rows.next()? {
            let x: i32 = r.get(0)?;
            let y: i32 = r.get(1)?;

            let unit_id: Option<i64> = r.get(2)?;
            let item_id: Option<i64> = r.get(3)?;
            let level_id: Option<i64> = r.get(4)?;
            let tile_type: Option<String> = r.get(5)?;

            present.insert((x, y));

            let has_data = unit_id.is_some()
                || item_id.is_some()
                || level_id.is_some()
                || tile_type.as_deref().is_some_and(|t| !t.trim().is_empty());

            if has_data {
                with_data.insert((x, y));
            }
        }

        // Update editor state
        self.hex_grid_id = Some(id);
        self.hex_grid_name = name;
        self.hex_grid_width = width.to_string();
        self.hex_grid_height = height.to_string();
        self.hex_grid_tiles_present = present;
        self.hex_grid_tiles_with_data = with_data;

        // Clear selection buffers
        self.hex_grid_selected_x = None;
        self.hex_grid_selected_y = None;

        // Clear selected-tile association editor buffers
        self.hex_tile_unit_query.clear();
        self.hex_tile_item_query.clear();
        self.hex_tile_level_query.clear();

        self.hex_tile_unit_id = None;
        self.hex_tile_item_id = None;
        self.hex_tile_level_id = None;

        self.hex_tile_unit_name = None;
        self.hex_tile_item_name = None;
        self.hex_tile_level_name = None;

        self.hex_tile_type.clear();

        Ok(())
    }

    fn delete_hex_grid_by_id(&mut self, id: i64) -> Result<()> {
        let conn = self.open_conn()?;

        // Cascade deletes tiles.
        conn.execute(
            r#"DELETE FROM hex_grids WHERE id = ?1"#,
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn save_hex_grid_from_editor_state(&mut self) -> Result<()> {
        // Validate form
        let name = self.hex_grid_name.trim();
        if name.is_empty() {
            bail!("Hex grid name cannot be empty");
        }

        let width: i32 = self
            .hex_grid_width
            .trim()
            .parse()
            .with_context(|| format!("Invalid width: {}", self.hex_grid_width))?;
        let height: i32 = self
            .hex_grid_height
            .trim()
            .parse()
            .with_context(|| format!("Invalid height: {}", self.hex_grid_height))?;

        if width <= 0 || height <= 0 {
            bail!("Width/height must be > 0");
        }

        let mut conn = self.open_conn()?;

        // Use a normal transaction.
        // `unchecked_transaction()` can trigger "cannot start a transaction within a transaction"
        // in some situations (e.g. if the connection is already in a transaction).
        let tx = conn.transaction()?;

        // Save the hex grid row.
        //
        // We cannot rely on `ON CONFLICT(name)` unless the DB has a UNIQUE constraint/index on `name`.
        // To be robust across schemas, we do:
        // - If we already have `hex_grid_id`, update that row.
        // - Else, try to find an existing row by name (if the column exists), then update it.
        // - Else, insert a new row (requires `name` column to exist).
        //
        // If your DB has migration 0009 applied, `name` exists and should be unique via an index.
        let grid_id: i64 = if let Some(id) = self.hex_grid_id {
            tx.execute(
                r#"
                UPDATE hex_grids
                SET name = ?1,
                    width = ?2,
                    height = ?3,
                    updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                WHERE id = ?4
                "#,
                rusqlite::params![name, width, height, id],
            )
            .with_context(|| format!("Update hex_grids by id={id}"))?;

            id
        } else {
            let existing_id: Option<i64> = tx
                .query_row(
                    r#"SELECT id FROM hex_grids WHERE name = ?1"#,
                    rusqlite::params![name],
                    |r| r.get(0),
                )
                .optional()
                .with_context(|| "Lookup hex_grids id by name (requires migration 0009)")?;

            if let Some(id) = existing_id {
                tx.execute(
                    r#"
                    UPDATE hex_grids
                    SET width = ?1,
                        height = ?2,
                        updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                    WHERE id = ?3
                    "#,
                    rusqlite::params![width, height, id],
                )
                .with_context(|| format!("Update hex_grids by id={id}"))?;

                id
            } else {
                tx.execute(
                    r#"
                    INSERT INTO hex_grids (name, width, height)
                    VALUES (?1, ?2, ?3)
                    "#,
                    rusqlite::params![name, width, height],
                )
                .with_context(|| "Insert hex_grids row (requires migration 0009)")?;

                tx.last_insert_rowid()
            }
        };

        // Delete any tiles that are now out-of-bounds for the new size.
        tx.execute(
            r#"
            DELETE FROM hex_tiles
            WHERE hex_grid_id = ?1
              AND (x < 0 OR y < 0 OR x >= ?2 OR y >= ?3)
            "#,
            rusqlite::params![grid_id, width, height],
        )
        .with_context(|| "Delete out-of-bounds hex tiles")?;

        // Sync tile presence:
        // - For every present tile in the editor set, upsert the row.
        // - For every in-bounds coordinate missing from the set, delete the row (empty space).
        let mut seen = std::collections::HashSet::<(i32, i32)>::new();

        for &(x, y) in self.hex_grid_tiles_present.iter() {
            if x < 0 || y < 0 || x >= width || y >= height {
                continue;
            }
            seen.insert((x, y));

            tx.execute(
                r#"
                INSERT INTO hex_tiles (hex_grid_id, x, y)
                VALUES (?1, ?2, ?3)
                ON CONFLICT(hex_grid_id, x, y)
                DO UPDATE SET
                    updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                "#,
                rusqlite::params![grid_id, x, y],
            )
            .with_context(|| format!("Upsert hex tile ({x},{y})"))?;
        }

        for y in 0..height {
            for x in 0..width {
                if !seen.contains(&(x, y)) {
                    tx.execute(
                        r#"
                        DELETE FROM hex_tiles
                        WHERE hex_grid_id = ?1 AND x = ?2 AND y = ?3
                        "#,
                        rusqlite::params![grid_id, x, y],
                    )
                    .with_context(|| format!("Delete empty hex tile ({x},{y})"))?;
                }
            }
        }

        tx.commit()?;

        self.hex_grid_id = Some(grid_id);

        Ok(())
    }

    fn create_unit_from_form(&self) -> Result<()> {
        let conn = self.open_conn()?;

        let unit = app::cards::unit::Unit {
            name: self.unit_name.trim().to_string(),
            strength: shared::form_parsing::parse_i64_required("Strength", &self.unit_strength)?,
            focus: shared::form_parsing::parse_i64_required("Focus", &self.unit_focus)?,
            intelligence: shared::form_parsing::parse_i64_required(
                "Intelligence",
                &self.unit_intelligence,
            )?,
            agility: shared::form_parsing::parse_i64_required("Agility", &self.unit_agility)?,
            knowledge: shared::form_parsing::parse_i64_required("Knowledge", &self.unit_knowledge)?,
        };

        app::cards::unit::save_card(&conn, &unit)?;
        Ok(())
    }

    fn create_unit_and_maybe_associate_from_form(&self) -> Result<()> {
        let conn = self.open_conn()?;

        let unit = app::cards::unit::Unit {
            name: self.unit_name.trim().to_string(),
            strength: shared::form_parsing::parse_i64_required("Strength", &self.unit_strength)?,
            focus: shared::form_parsing::parse_i64_required("Focus", &self.unit_focus)?,
            intelligence: shared::form_parsing::parse_i64_required(
                "Intelligence",
                &self.unit_intelligence,
            )?,
            agility: shared::form_parsing::parse_i64_required("Agility", &self.unit_agility)?,
            knowledge: shared::form_parsing::parse_i64_required("Knowledge", &self.unit_knowledge)?,
        };

        let unit_name = unit.name.clone();
        app::cards::unit::save_card(&conn, &unit)?;

        let action_name = self.unit_assoc_action_name.trim();
        if !action_name.is_empty() {
            app::cards::action::set_association(
                &conn,
                action_name,
                &app::cards::action::ActionAssociation::Unit { unit_name },
            )?;
        }

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

    fn create_item_and_maybe_associate_from_form(&self) -> Result<()> {
        let mut conn = self.open_conn()?;

        let item = app::cards::item::Item {
            name: self.item_name.trim().to_string(),
        };

        let item_name = item.name.clone();
        app::cards::item::save_card(&conn, &item)?;

        let action_name = self.item_assoc_action_name.trim();
        if !action_name.is_empty() {
            app::cards::action::set_association(
                &conn,
                action_name,
                &app::cards::action::ActionAssociation::Item {
                    item_name: item_name.clone(),
                },
            )?;
        }

        // Persist any queued armor modifiers now that the item exists.
        // CardId is derived from items.id and the modifier is linked to this item.
        for pending in &self.pending_armor_modifiers {
            let _id = app::cards::armor_modifier::create_and_link_to_item_by_name(
                &mut conn, pending, &item_name,
            )?;
        }

        // Persist any queued stat modifiers now that the item exists.
        for pending in &self.pending_stat_modifiers {
            let _id = app::cards::stat_modifier::create_and_link_to_item_by_name(
                &mut conn, pending, &item_name,
            )?;
        }

        Ok(())
    }

    fn create_level_from_form(&self) -> Result<()> {
        let conn = self.open_conn()?;

        let level = app::cards::level::Level {
            name: self.level_name.trim().to_string(),
            text: self.level_text.trim().to_string(),
        };

        app::cards::level::save_card(&conn, &level)?;
        Ok(())
    }

    fn create_level_and_maybe_associate_from_form(&self) -> Result<()> {
        let mut conn = self.open_conn()?;

        let level = app::cards::level::Level {
            name: self.level_name.trim().to_string(),
            text: self.level_text.trim().to_string(),
        };

        let level_name = level.name.clone();
        app::cards::level::save_card(&conn, &level)?;

        let action_name = self.level_assoc_action_name.trim();
        if !action_name.is_empty() {
            app::cards::action::set_association(
                &conn,
                action_name,
                &app::cards::action::ActionAssociation::Level {
                    level_name: level_name.clone(),
                },
            )?;
        }

        // Persist any queued armor modifiers now that the level exists.
        // CardId is derived from levels.id and the modifier is linked to this level.
        for pending in &self.pending_armor_modifiers {
            let _id = app::cards::armor_modifier::create_and_link_to_level_by_name(
                &mut conn,
                pending,
                &level_name,
            )?;
        }

        // Persist any queued stat modifiers now that the level exists.
        for pending in &self.pending_stat_modifiers {
            let _id = app::cards::stat_modifier::create_and_link_to_level_by_name(
                &mut conn,
                pending,
                &level_name,
            )?;
        }

        Ok(())
    }

    fn create_action_with_subtype_from_form(&self) -> Result<()> {
        let conn = self.open_conn()?;

        let action_point_cost =
            shared::form_parsing::parse_i64_required("Action Point Cost", &self.action_point_cost)?;
        let action_type = match self.action_type.trim() {
            "Interaction" => app::cards::action::ActionType::Interaction,
            "Attack" => app::cards::action::ActionType::Attack,
            other => {
                anyhow::bail!("Action Type must be `Interaction` or `Attack` (got `{other}`)",)
            }
        };

        let action_name = self.action_name.trim().to_string();
        let text = self.action_text.trim().to_string();

        let action = app::cards::action::Action {
            name: action_name.clone(),
            action_point_cost,
            action_type,
            text,
        };

        // Treat Action + subtype (+ optional association) as one entity during creation.
        // 1) Create the Action row.
        app::cards::action::save_card(&conn, &action)?;

        // 2) Create exactly one subtype row, based on Action.action_type.
        match action.action_type {
            app::cards::action::ActionType::Attack => {
                let damage =
                    shared::form_parsing::parse_i64_required("Damage", &self.attack_damage)?;
                let range = shared::form_parsing::parse_i64_required("Range", &self.attack_range)?;
                let target =
                    shared::form_parsing::parse_i64_required("Target", &self.attack_target)?;

                let damage_type = match self.attack_damage_type.trim() {
                    "Arcane" => app::cards::attack::DamageType::Arcane,
                    "Physical" => app::cards::attack::DamageType::Physical,
                    other => {
                        anyhow::bail!("Damage Type must be `Arcane` or `Physical` (got `{other}`)")
                    }
                };

                let skill = match self.attack_skill.trim() {
                    "Strength" => app::cards::attack::Skill::Strength,
                    "Focus" => app::cards::attack::Skill::Focus,
                    "Intelligence" => app::cards::attack::Skill::Intelligence,
                    "Knowledge" => app::cards::attack::Skill::Knowledge,
                    "Agility" => app::cards::attack::Skill::Agility,
                    other => anyhow::bail!(
                        "Skill must be Strength/Focus/Intelligence/Knowledge/Agility (got `{other}`)"
                    ),
                };

                let atk = app::cards::attack::Attack {
                    action_name,
                    damage,
                    damage_type,
                    skill,
                    target,
                    range,
                };

                let _ = app::cards::attack::save_card(&conn, &atk)?;
            }
            app::cards::action::ActionType::Interaction => {
                let range =
                    shared::form_parsing::parse_i64_required("Range", &self.interaction_range)?;
                let target =
                    shared::form_parsing::parse_i64_optional("Target", &self.interaction_target)?;

                let skill = match self.interaction_skill.trim() {
                    "Strength" => app::cards::interaction::Skill::Strength,
                    "Focus" => app::cards::interaction::Skill::Focus,
                    "Intelligence" => app::cards::interaction::Skill::Intelligence,
                    "Knowledge" => app::cards::interaction::Skill::Knowledge,
                    "Agility" => app::cards::interaction::Skill::Agility,
                    other => anyhow::bail!(
                        "Skill must be Strength/Focus/Intelligence/Knowledge/Agility (got `{other}`)"
                    ),
                };

                let ix = app::cards::interaction::Interaction {
                    action_name,
                    range,
                    skill,
                    target,
                };

                let _ = app::cards::interaction::save_card(&conn, &ix)?;
            }
        }

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

    fn delete_level(&self, name: &str) -> Result<()> {
        let conn = self.open_conn()?;
        let _ = app::cards::level::delete_card(&conn, name)?;
        Ok(())
    }

    fn delete_action(&self, name: &str) -> Result<()> {
        let conn = self.open_conn()?;
        let _ = app::cards::action::delete_card(&conn, name)?;
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

        // Pre-fill association buffer with currently-linked action name (if any).
        //
        // We use the already-loaded `self.actions` list (refreshed via `Message::Refresh`)
        // rather than hitting the DB again here.
        self.unit_assoc_action_name = self
            .actions
            .iter()
            .find(|a| a.unit_name.as_deref() == Some(u.name.as_str()))
            .map(|a| a.name.clone())
            .unwrap_or_default();

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

        // Pre-fill association buffer with currently-linked action name (if any).
        //
        // We use the already-loaded `self.actions` list (refreshed via `Message::Refresh`)
        // rather than hitting the DB again here.
        self.item_assoc_action_name = self
            .actions
            .iter()
            .find(|a| a.item_name.as_deref() == Some(it.name.as_str()))
            .map(|a| a.name.clone())
            .unwrap_or_default();

        self.active_view = ActiveView::EditItem {
            original_name: it.name,
        };
        Ok(())
    }

    fn begin_edit_level(&mut self, name: &str) -> Result<()> {
        let conn = self.open_conn()?;
        let lv = app::cards::level::get_card(&conn, name)?
            .with_context(|| format!("Level `{name}` not found"))?;

        self.level_name = lv.name.clone();
        self.level_text = lv.text.clone();

        // Pre-fill association buffer with currently-linked action name (if any).
        //
        // We use the already-loaded `self.actions` list (refreshed via `Message::Refresh`)
        // rather than hitting the DB again here.
        self.level_assoc_action_name = self
            .actions
            .iter()
            .find(|a| a.level_name.as_deref() == Some(lv.name.as_str()))
            .map(|a| a.name.clone())
            .unwrap_or_default();

        self.active_view = ActiveView::EditLevel {
            original_name: lv.name,
        };
        Ok(())
    }

    fn begin_edit_action(&mut self, name: &str) -> Result<()> {
        let conn = self.open_conn()?;
        let a = app::cards::action::get_card(&conn, name)?
            .with_context(|| format!("Action `{name}` not found"))?;

        self.action_name = a.name.clone();
        self.action_point_cost = a.action_point_cost.to_string();
        self.action_type = match a.action_type {
            app::cards::action::ActionType::Interaction => "Interaction".to_string(),
            app::cards::action::ActionType::Attack => "Attack".to_string(),
        };
        self.action_text = a.text.clone();

        // Pre-fill subforms from existing attack/interaction rows (if any).
        if let Some(atk) = app::cards::attack::get_card(&conn, &a.name)? {
            self.attack_damage = atk.damage.to_string();
            self.attack_damage_type = match atk.damage_type {
                app::cards::attack::DamageType::Arcane => "Arcane".to_string(),
                app::cards::attack::DamageType::Physical => "Physical".to_string(),
            };
            self.attack_skill = match atk.skill {
                app::cards::attack::Skill::Strength => "Strength".to_string(),
                app::cards::attack::Skill::Focus => "Focus".to_string(),
                app::cards::attack::Skill::Intelligence => "Intelligence".to_string(),
                app::cards::attack::Skill::Knowledge => "Knowledge".to_string(),
                app::cards::attack::Skill::Agility => "Agility".to_string(),
            };
            self.attack_target = atk.target.to_string();
            self.attack_range = atk.range.to_string();
        } else {
            // Keep defaults / whatever is already in the buffer.
        }

        if let Some(ix) = app::cards::interaction::get_card(&conn, &a.name)? {
            self.interaction_range = ix.range.to_string();
            self.interaction_skill = match ix.skill {
                app::cards::interaction::Skill::Strength => "Strength".to_string(),
                app::cards::interaction::Skill::Focus => "Focus".to_string(),
                app::cards::interaction::Skill::Intelligence => "Intelligence".to_string(),
                app::cards::interaction::Skill::Knowledge => "Knowledge".to_string(),
                app::cards::interaction::Skill::Agility => "Agility".to_string(),
            };
            self.interaction_target = ix.target.map(|t| t.to_string()).unwrap_or_default();
        } else {
            // Keep defaults / whatever is already in the buffer.
        }

        self.active_view = ActiveView::EditAction {
            original_name: a.name,
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
            strength: shared::form_parsing::parse_i64_required("Strength", &self.unit_strength)?,
            focus: shared::form_parsing::parse_i64_required("Focus", &self.unit_focus)?,
            intelligence: shared::form_parsing::parse_i64_required(
                "Intelligence",
                &self.unit_intelligence,
            )?,
            agility: shared::form_parsing::parse_i64_required("Agility", &self.unit_agility)?,
            knowledge: shared::form_parsing::parse_i64_required("Knowledge", &self.unit_knowledge)?,
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

    fn save_level_edits(&self) -> Result<()> {
        let ActiveView::EditLevel { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing a level");
        };

        let conn = self.open_conn()?;

        let new_name = self.level_name.trim().to_string();
        if new_name.is_empty() {
            anyhow::bail!("Level.name must be non-empty");
        }

        let new_text = self.level_text.trim().to_string();
        if new_text.is_empty() {
            anyhow::bail!("Level.text must be non-empty");
        }

        let level = app::cards::level::Level {
            name: new_name,
            text: new_text,
        };

        let updated = app::cards::level::rename_card(&conn, original_name, &level)?
            .with_context(|| format!("Level `{}` does not exist", original_name))?;
        let _ = updated;

        Ok(())
    }

    fn save_action_edits(&self) -> Result<()> {
        let ActiveView::EditAction { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing an action");
        };

        let conn = self.open_conn()?;

        let new_name = self.action_name.trim().to_string();
        if new_name.is_empty() {
            anyhow::bail!("Action.name must be non-empty");
        }

        let action_point_cost =
            shared::form_parsing::parse_i64_required("Action Point Cost", &self.action_point_cost)?;
        let action_type = match self.action_type.trim() {
            "Interaction" => app::cards::action::ActionType::Interaction,
            "Attack" => app::cards::action::ActionType::Attack,
            other => {
                anyhow::bail!("Action Type must be `Interaction` or `Attack` (got `{other}`)",)
            }
        };

        let text = self.action_text.trim().to_string();
        if text.is_empty() {
            anyhow::bail!("Action.text must be non-empty");
        }

        let action = app::cards::action::Action {
            name: new_name.clone(),
            action_point_cost,
            action_type,
            text,
        };

        let updated = app::cards::action::rename_and_update_card(&conn, original_name, &action)?
            .with_context(|| format!("Action `{}` does not exist", original_name))?;
        let _ = updated;

        Ok(())
    }

    fn save_unit_association(&self) -> Result<()> {
        let ActiveView::EditUnit { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing a unit");
        };

        let conn = self.open_conn()?;

        let action_name = self.unit_assoc_action_name.trim();
        if action_name.is_empty() {
            anyhow::bail!("Select an Action (or use Clear) to associate with this Unit");
        }

        app::cards::action::set_association(
            &conn,
            action_name,
            &app::cards::action::ActionAssociation::Unit {
                unit_name: original_name.clone(),
            },
        )?;

        Ok(())
    }

    // NOTE: `clear_unit_association` removed.
    // Associations are now removed one-by-one via `remove_unit_association(action_name)`.

    fn remove_unit_association(&self, action_name: &str) -> Result<()> {
        let ActiveView::EditUnit { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing a unit");
        };

        let conn = self.open_conn()?;

        // Only clear if this action is actually associated with this unit.
        match app::cards::action::get_association(&conn, action_name)? {
            Some(app::cards::action::ActionAssociation::Unit { unit_name })
                if unit_name == *original_name =>
            {
                app::cards::action::clear_association(&conn, action_name)?;
                Ok(())
            }
            Some(_) => anyhow::bail!("Action `{}` is not associated with this Unit", action_name),
            None => anyhow::bail!("Action `{}` not found (or has no association)", action_name),
        }
    }

    fn save_item_association(&self) -> Result<()> {
        let ActiveView::EditItem { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing an item");
        };

        let conn = self.open_conn()?;

        let action_name = self.item_assoc_action_name.trim();
        if action_name.is_empty() {
            anyhow::bail!("Select an Action (or use Clear) to associate with this Item");
        }

        app::cards::action::set_association(
            &conn,
            action_name,
            &app::cards::action::ActionAssociation::Item {
                item_name: original_name.clone(),
            },
        )?;

        Ok(())
    }

    // NOTE: `clear_item_association` removed.
    // Associations are now removed one-by-one via `remove_item_association(action_name)`.

    fn remove_item_association(&self, action_name: &str) -> Result<()> {
        let ActiveView::EditItem { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing an item");
        };

        let conn = self.open_conn()?;

        match app::cards::action::get_association(&conn, action_name)? {
            Some(app::cards::action::ActionAssociation::Item { item_name })
                if item_name == *original_name =>
            {
                app::cards::action::clear_association(&conn, action_name)?;
                Ok(())
            }
            Some(_) => anyhow::bail!("Action `{}` is not associated with this Item", action_name),
            None => anyhow::bail!("Action `{}` not found (or has no association)", action_name),
        }
    }

    fn save_level_association(&self) -> Result<()> {
        let ActiveView::EditLevel { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing a level");
        };

        let conn = self.open_conn()?;

        let action_name = self.level_assoc_action_name.trim();
        if action_name.is_empty() {
            anyhow::bail!("Select an Action (or use Clear) to associate with this Level");
        }

        app::cards::action::set_association(
            &conn,
            action_name,
            &app::cards::action::ActionAssociation::Level {
                level_name: original_name.clone(),
            },
        )?;

        Ok(())
    }

    // NOTE: `clear_level_association` removed.
    // Associations are now removed one-by-one via `remove_level_association(action_name)`.

    fn remove_level_association(&self, action_name: &str) -> Result<()> {
        let ActiveView::EditLevel { original_name } = &self.active_view else {
            anyhow::bail!("Not currently editing a level");
        };

        let conn = self.open_conn()?;

        match app::cards::action::get_association(&conn, action_name)? {
            Some(app::cards::action::ActionAssociation::Level { level_name })
                if level_name == *original_name =>
            {
                app::cards::action::clear_association(&conn, action_name)?;
                Ok(())
            }
            Some(_) => anyhow::bail!("Action `{}` is not associated with this Level", action_name),
            None => anyhow::bail!("Action `{}` not found (or has no association)", action_name),
        }
    }

    fn current_action_name_for_subforms(&self) -> Result<&str> {
        let ActiveView::EditAction { original_name: _ } = &self.active_view else {
            anyhow::bail!("Attack/Interaction edits are only available while editing an action");
        };
        let name = self.action_name.trim();
        if name.is_empty() {
            anyhow::bail!("Action.name must be non-empty to edit Attack/Interaction");
        }
        Ok(name)
    }

    fn save_attack_edits(&self) -> Result<()> {
        let conn = self.open_conn()?;
        let action_name = self.current_action_name_for_subforms()?.to_string();

        let damage = shared::form_parsing::parse_i64_required("Damage", &self.attack_damage)?;
        let range = shared::form_parsing::parse_i64_required("Range", &self.attack_range)?;
        let target = shared::form_parsing::parse_i64_required("Target", &self.attack_target)?;

        let damage_type = match self.attack_damage_type.trim() {
            "Arcane" => app::cards::attack::DamageType::Arcane,
            "Physical" => app::cards::attack::DamageType::Physical,
            other => anyhow::bail!("Damage Type must be `Arcane` or `Physical` (got `{other}`)"),
        };

        let skill = match self.attack_skill.trim() {
            "Strength" => app::cards::attack::Skill::Strength,
            "Focus" => app::cards::attack::Skill::Focus,
            "Intelligence" => app::cards::attack::Skill::Intelligence,
            "Knowledge" => app::cards::attack::Skill::Knowledge,
            "Agility" => app::cards::attack::Skill::Agility,
            other => anyhow::bail!(
                "Skill must be Strength/Focus/Intelligence/Knowledge/Agility (got `{other}`)"
            ),
        };

        let atk = app::cards::attack::Attack {
            action_name,
            damage,
            damage_type,
            skill,
            target,
            range,
        };

        // Upsert: update if exists, otherwise create.
        if app::cards::attack::update_card(&conn, &atk)?.is_none() {
            let _ = app::cards::attack::save_card(&conn, &atk)?;
        }

        Ok(())
    }

    fn delete_attack_for_current_action(&self) -> Result<()> {
        let conn = self.open_conn()?;
        let action_name = self.current_action_name_for_subforms()?;
        let _ = app::cards::attack::delete_card(&conn, action_name)?;
        Ok(())
    }

    fn save_interaction_edits(&self) -> Result<()> {
        let conn = self.open_conn()?;
        let action_name = self.current_action_name_for_subforms()?.to_string();

        let range = shared::form_parsing::parse_i64_required("Range", &self.interaction_range)?;
        let target = shared::form_parsing::parse_i64_optional("Target", &self.interaction_target)?;

        let skill = match self.interaction_skill.trim() {
            "Strength" => app::cards::interaction::Skill::Strength,
            "Focus" => app::cards::interaction::Skill::Focus,
            "Intelligence" => app::cards::interaction::Skill::Intelligence,
            "Knowledge" => app::cards::interaction::Skill::Knowledge,
            "Agility" => app::cards::interaction::Skill::Agility,
            other => anyhow::bail!(
                "Skill must be Strength/Focus/Intelligence/Knowledge/Agility (got `{other}`)"
            ),
        };

        let ix = app::cards::interaction::Interaction {
            action_name,
            range,
            skill,
            target,
        };

        // Upsert: update if exists, otherwise create.
        if app::cards::interaction::update_card(&conn, &ix)?.is_none() {
            let _ = app::cards::interaction::save_card(&conn, &ix)?;
        }

        Ok(())
    }

    fn delete_interaction_for_current_action(&self) -> Result<()> {
        let conn = self.open_conn()?;
        let action_name = self.current_action_name_for_subforms()?;
        let _ = app::cards::interaction::delete_card(&conn, action_name)?;
        Ok(())
    }
}
