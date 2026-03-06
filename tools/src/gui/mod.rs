mod shared;
mod views;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use iced::widget::{button, column, container, horizontal_rule, row, text};
use iced::{Application, Command, Element, Length, Settings, Subscription, Theme};

use crate::app;

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
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::Units => "Units",
            Tab::Items => "Items",
            Tab::Levels => "Levels",
            Tab::Actions => "Actions",
            Tab::ArmorModifiers => "Armor Modifiers",
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

    Refresh,

    // Armor modifiers form (single "draft" row editor)
    ArmorModifierValueChanged(String),
    ArmorModifierSuitChanged(String),
    ArmorModifierDamageTypeChanged(String),

    // Pending armor modifiers (used by create-card flows)
    AddPendingArmorModifier,
    RemovePendingArmorModifier(usize),
    ClearPendingArmorModifiers,

    // Create an armor modifier in the context of the current view:
    // - If editing an Item, it will be linked to that Item.
    // - If editing a Level, it will be linked to that Level.
    // - Otherwise, creation is rejected (prevents unassociated modifiers).
    CreateArmorModifier,

    // De-link an armor modifier from its associated card (item/level).
    // The armor modifier row remains, but the association link row is removed.
    RemoveArmorModifierLink(i64),

    // Armor modifiers edit navigation
    EditArmorModifier(i64),

    // Armor modifiers save/delete
    SaveArmorModifierEdits,
    DeleteArmorModifier(i64),

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

    // Levels form
    LevelNameChanged(String),
    LevelTextChanged(String),
    CreateLevel,

    // Actions form
    ActionNameChanged(String),
    ActionPointCostChanged(String),
    ActionTypeChanged(String),
    ActionTextChanged(String),

    CreateAction,

    // Optional associations set from Unit/Item/Level views
    UnitAssocActionNameChanged(String),
    ItemAssocActionNameChanged(String),
    LevelAssocActionNameChanged(String),

    AddUnitAssociation,
    RemoveUnitAssociation(String),

    AddItemAssociation,
    RemoveItemAssociation(String),

    AddLevelAssociation,
    RemoveLevelAssociation(String),

    // Create + associate in one step (from create forms)
    CreateUnitAndMaybeAssociate,
    CreateItemAndMaybeAssociate,
    CreateLevelAndMaybeAssociate,

    // Attack subform (used for create + edit when ActionType = Attack)
    AttackDamageChanged(String),
    AttackDamageTypeChanged(String),
    AttackSkillChanged(String),
    AttackTargetChanged(String),
    AttackRangeChanged(String),
    SaveAttackEdits,
    DeleteAttack,

    // Interaction subform (used for create + edit when ActionType = Interaction)
    InteractionRangeChanged(String),
    InteractionSkillChanged(String),
    InteractionTargetChanged(String), // empty string => NULL
    SaveInteractionEdits,
    DeleteInteraction,

    // Edit navigation
    EditUnit(String),
    EditItem(String),
    EditLevel(String),
    EditAction(String),
    CancelEdit,

    // Save edits
    SaveUnitEdits,
    SaveItemEdits,
    SaveLevelEdits,
    SaveActionEdits,

    // Delete actions
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

    // Pending armor modifiers (used by create-card flows; linked on create)
    pub pending_armor_modifiers: Vec<app::cards::armor_modifier::ArmorModifier>,

    // loaded data
    pub units: Vec<UnitRow>,
    pub items: Vec<ItemRow>,
    pub levels: Vec<LevelRow>,
    pub actions: Vec<ActionRow>,
    pub armor_modifiers: Vec<ArmorModifierRow>,

    // ui state
    pub status: Option<String>,
    pub active_view: ActiveView,
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

            pending_armor_modifiers: vec![],

            units: vec![],
            items: vec![],
            levels: vec![],
            actions: vec![],
            armor_modifiers: vec![],

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
                Command::none()
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
        };

        container(
            column![
                header,
                tabs,
                horizontal_rule(1),
                status,
                horizontal_rule(1),
                iced::widget::scrollable(content).height(Length::Fill),
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
