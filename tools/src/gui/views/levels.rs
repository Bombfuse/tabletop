use anyhow::Result;
use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::gui::views::shared::associated_actions;
use crate::gui::{LevelRow, Message, ToolsGui};

#[derive(Debug, Clone, PartialEq, Eq)]
struct SuitChoice(String);

impl std::fmt::Display for SuitChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn suit_choices() -> Vec<SuitChoice> {
    vec![
        SuitChoice("Spades".to_string()),
        SuitChoice("Clubs".to_string()),
        SuitChoice("Diamonds".to_string()),
        SuitChoice("Hearts".to_string()),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DamageTypeChoice(String);

impl std::fmt::Display for DamageTypeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn damage_type_choices() -> Vec<DamageTypeChoice> {
    vec![
        DamageTypeChoice("Physical".to_string()),
        DamageTypeChoice("Arcane".to_string()),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StatChoice(String);

impl std::fmt::Display for StatChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn stat_choices() -> Vec<StatChoice> {
    vec![
        StatChoice("Strength".to_string()),
        StatChoice("Focus".to_string()),
        StatChoice("Intelligence".to_string()),
        StatChoice("Knowledge".to_string()),
        StatChoice("Agility".to_string()),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OperatorChoice(String);

impl std::fmt::Display for OperatorChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn operator_choices() -> Vec<OperatorChoice> {
    vec![
        OperatorChoice("Add".to_string()),
        OperatorChoice("Subtract".to_string()),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingArmorModifierRow {
    pub value: i64,
    pub suit: String,
    pub damage_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingStatModifierRow {
    pub stat: String,
    pub value: i64,
    pub operator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActionNameChoice(String);

impl std::fmt::Display for ActionNameChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn normalize_for_match(s: &str) -> String {
    s.trim().to_lowercase()
}

pub fn view(app: &ToolsGui) -> Element<'_, Message> {
    // Create form: include optional "Add Action" association input.
    // The association is applied when creating the level (if an action name is selected/typed).
    let filter = normalize_for_match(&app.level_assoc_action_name);

    let mut action_choices: Vec<ActionNameChoice> = app
        .actions
        .iter()
        .map(|a| a.name.clone())
        .filter(|name| {
            if filter.is_empty() {
                true
            } else {
                normalize_for_match(name).contains(&filter)
            }
        })
        .map(ActionNameChoice)
        .collect();
    action_choices.sort_by(|a, b| a.0.cmp(&b.0));

    let selected_action = {
        let trimmed = app.level_assoc_action_name.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(ActionNameChoice(trimmed.to_string()))
        }
    };

    let add_assoc_row = row![
        text("Add Action").width(Length::Fixed(140.0)),
        pick_list(
            action_choices,
            selected_action,
            |choice: ActionNameChoice| Message::LevelAssocActionNameChanged(choice.0)
        )
        .placeholder("Select...")
        .width(Length::Fixed(320.0)),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let add_assoc_filter_row = row![
        text("Filter").width(Length::Fixed(140.0)),
        text_input("type to filter", &app.level_assoc_action_name)
            .on_input(Message::LevelAssocActionNameChanged)
            .padding(8)
            .width(Length::Fixed(320.0)),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let pending: Vec<PendingArmorModifierRow> = app
        .pending_armor_modifiers
        .iter()
        .map(|am| PendingArmorModifierRow {
            value: am.value,
            suit: am.suit.as_str().to_string(),
            damage_type: am.damage_type.as_str().to_string(),
        })
        .collect();

    let mut pending_col = column![
        text(format!(
            "Pending armor modifiers ({} queued)",
            pending.len()
        ))
        .size(12)
    ]
    .spacing(6);

    if pending.is_empty() {
        pending_col = pending_col.push(text("(none)").size(12));
    } else {
        for (idx, p) in pending.iter().enumerate() {
            pending_col = pending_col.push(
                row![
                    text(format!(
                        "value={} suit={} damage_type={}",
                        p.value, p.suit, p.damage_type
                    ))
                    .size(12),
                    iced::widget::Space::with_width(Length::Fill),
                    button("Remove").on_press(Message::RemovePendingArmorModifier(idx)),
                ]
                .spacing(12)
                .align_items(Alignment::Center),
            );
        }

        pending_col = pending_col.push(
            row![
                button("Clear queued").on_press(Message::ClearPendingArmorModifiers),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12),
        );
    }

    let pending_stat: Vec<PendingStatModifierRow> = app
        .pending_stat_modifiers
        .iter()
        .map(|sm| PendingStatModifierRow {
            stat: sm.stat.as_str().to_string(),
            value: sm.value,
            operator: sm.operator.as_str().to_string(),
        })
        .collect();

    let mut pending_stat_col = column![
        text(format!(
            "Pending stat modifiers ({} queued)",
            pending_stat.len()
        ))
        .size(12)
    ]
    .spacing(6);

    if pending_stat.is_empty() {
        pending_stat_col = pending_stat_col.push(text("(none)").size(12));
    } else {
        for (idx, p) in pending_stat.iter().enumerate() {
            pending_stat_col = pending_stat_col.push(
                row![
                    text(format!("{} {} {}", p.operator, p.value, p.stat)).size(12),
                    iced::widget::Space::with_width(Length::Fill),
                    button("Remove").on_press(Message::RemovePendingStatModifier(idx)),
                ]
                .spacing(12)
                .align_items(Alignment::Center),
            );
        }

        pending_stat_col = pending_stat_col.push(
            row![
                button("Clear queued").on_press(Message::ClearPendingStatModifiers),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12),
        );
    }

    let selected_stat = {
        let trimmed = app.stat_modifier_stat.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(StatChoice(trimmed.to_string()))
        }
    };

    let selected_operator = {
        let trimmed = app.stat_modifier_operator.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(OperatorChoice(trimmed.to_string()))
        }
    };

    let form = column![
        text("Create Level").size(18),
        row![
            labeled_input("Name", &app.level_name, Message::LevelNameChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        row![
            labeled_multiline_input("Text", &app.level_text, Message::LevelTextChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        horizontal_rule(1),
        add_assoc_row,
        add_assoc_filter_row,
        row![button("Create").on_press(Message::CreateLevelAndMaybeAssociate)].spacing(12),
        horizontal_rule(1),
        text("Optional Stat Modifiers").size(16),
        row![
            column![
                text("Stat").size(12),
                pick_list(stat_choices(), selected_stat, |c: StatChoice| {
                    Message::StatModifierStatChanged(c.0)
                })
                .placeholder("Select...")
                .width(Length::Fixed(220.0)),
            ]
            .spacing(4),
            column![
                text("Operator").size(12),
                pick_list(
                    operator_choices(),
                    selected_operator,
                    |c: OperatorChoice| Message::StatModifierOperatorChanged(c.0)
                )
                .placeholder("Select...")
                .width(Length::Fixed(220.0)),
            ]
            .spacing(4),
            column![
                text("Value").size(12),
                text_input("Value", &app.stat_modifier_value)
                    .on_input(Message::StatModifierValueChanged)
                    .padding(8)
                    .width(Length::Fixed(220.0)),
            ]
            .spacing(4),
            button("Queue Stat Modifier").on_press(Message::AddPendingStatModifier),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::End),
        pending_stat_col,
        horizontal_rule(1),
        text("Optional Armor Modifiers").size(16),
        row![
            labeled_input(
                "Value",
                &app.armor_modifier_value,
                Message::ArmorModifierValueChanged
            ),
            labeled_pick_list(
                "Suit",
                suit_choices(),
                &app.armor_modifier_suit,
                Message::ArmorModifierSuitChanged
            ),
            labeled_pick_list_damage_type(
                "Damage Type",
                damage_type_choices(),
                &app.armor_modifier_damage_type,
                Message::ArmorModifierDamageTypeChanged
            ),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::End),
        row![button("Queue Armor Modifier").on_press(Message::AddPendingArmorModifier)].spacing(12),
        pending_col,
    ]
    .spacing(10);

    let list_header = row![
        text("Levels").size(18),
        iced::widget::Space::with_width(Length::Fill),
        text(format!("{} total", app.levels.len())).size(14),
    ];

    let mut list_col = column![list_header].spacing(8);

    for lv in &app.levels {
        let preview = preview_text(&lv.text);

        let row_el = row![
            column![text(lv.name.clone()).size(16), text(preview).size(12),].spacing(2),
            iced::widget::Space::with_width(Length::Fill),
            button("Edit").on_press(Message::EditLevel(lv.name.clone())),
            button("Delete").on_press(Message::DeleteLevel(lv.name.clone())),
        ]
        .spacing(12);

        list_col = list_col.push(container(row_el).padding(8));
    }
    // Let the parent/root scroll view measure this content naturally.
    // Forcing `Length::Fill` here can fight the root scroll container and feel "infinite".
    column![form, horizontal_rule(1), list_col]
        .spacing(12)
        .into()
}

pub fn edit_view<'a>(app: &'a ToolsGui, original_name: &'a str) -> Element<'a, Message> {
    let header = row![
        text(format!("Edit Level: {original_name}")).size(18),
        iced::widget::Space::with_width(Length::Fill),
        button("Cancel").on_press(Message::CancelEdit),
        button("Save").on_press(Message::SaveLevelEdits),
    ]
    .spacing(12);

    // Add association: pick an Action name (dropdown) and click "Add".
    //
    // We filter choices using whatever the user has typed into the association box.
    // This approximates an "autocomplete dropdown" without a dedicated widget.
    let filter = normalize_for_match(&app.level_assoc_action_name);

    let mut action_choices: Vec<ActionNameChoice> = app
        .actions
        .iter()
        .map(|a| a.name.clone())
        .filter(|name| {
            if filter.is_empty() {
                true
            } else {
                normalize_for_match(name).contains(&filter)
            }
        })
        .map(ActionNameChoice)
        .collect();
    action_choices.sort_by(|a, b| a.0.cmp(&b.0));

    let selected_action = {
        let trimmed = app.level_assoc_action_name.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(ActionNameChoice(trimmed.to_string()))
        }
    };

    let add_assoc_row = row![
        text("Add Action").width(Length::Fixed(140.0)),
        pick_list(
            action_choices,
            selected_action,
            |choice: ActionNameChoice| Message::LevelAssocActionNameChanged(choice.0)
        )
        .placeholder("Select...")
        .width(Length::Fixed(320.0)),
        button("Add").on_press(Message::AddLevelAssociation),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let add_assoc_filter_row = row![
        text("Filter").width(Length::Fixed(140.0)),
        text_input("type to filter", &app.level_assoc_action_name)
            .on_input(Message::LevelAssocActionNameChanged)
            .padding(8)
            .width(Length::Fixed(320.0)),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    // Fully list all associated actions at the bottom (shared component).
    let associated = associated_actions::collect_for_level(&app.actions, original_name);
    let associated_list =
        associated_actions::view(&associated, |name| Message::RemoveLevelAssociation(name));

    // List armor modifiers associated with this level.
    let mut armor_mods_col = column![text("Armor Modifiers").size(16)].spacing(8);
    let mut any = false;

    for am in &app.armor_modifiers {
        if am.level_name.as_deref() == Some(original_name) {
            any = true;
            armor_mods_col = armor_mods_col.push(
                row![
                    text(format!(
                        "id={} value={} suit={} damage_type={}",
                        am.id, am.value, am.suit, am.damage_type
                    ))
                    .size(12),
                    iced::widget::Space::with_width(Length::Fill),
                    button("Edit").on_press(Message::EditArmorModifier(am.id)),
                    button("Remove").on_press(Message::RemoveArmorModifierLink(am.id)),
                ]
                .spacing(12)
                .align_items(Alignment::Center),
            );
        }
    }

    if !any {
        armor_mods_col = armor_mods_col.push(text("(none)").size(12));
    }

    // List stat modifiers associated with this level.
    let mut stat_mods_col = column![text("Stat Modifiers").size(16)].spacing(8);
    let mut any_stat_mods = false;

    for sm in &app.stat_modifiers {
        if sm.level_name.as_deref() == Some(original_name) {
            any_stat_mods = true;
            stat_mods_col = stat_mods_col.push(
                row![
                    text(format!(
                        "id={} {} {} {}",
                        sm.id, sm.operator, sm.value, sm.stat
                    ))
                    .size(12),
                    iced::widget::Space::with_width(Length::Fill),
                    button("Delete").on_press(Message::DeleteStatModifier(sm.id)),
                ]
                .spacing(12)
                .align_items(Alignment::Center),
            );
        }
    }

    if !any_stat_mods {
        stat_mods_col = stat_mods_col.push(text("(none)").size(12));
    }

    // Add stat modifier form (uses shared GUI buffers on `ToolsGui`).
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct StatChoice(String);

    impl std::fmt::Display for StatChoice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }

    fn stat_choices() -> Vec<StatChoice> {
        vec![
            StatChoice("Strength".to_string()),
            StatChoice("Focus".to_string()),
            StatChoice("Intelligence".to_string()),
            StatChoice("Knowledge".to_string()),
            StatChoice("Agility".to_string()),
        ]
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct OperatorChoice(String);

    impl std::fmt::Display for OperatorChoice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }

    fn operator_choices() -> Vec<OperatorChoice> {
        vec![
            OperatorChoice("Add".to_string()),
            OperatorChoice("Subtract".to_string()),
        ]
    }

    let selected_stat = {
        let trimmed = app.stat_modifier_stat.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(StatChoice(trimmed.to_string()))
        }
    };

    let selected_operator = {
        let trimmed = app.stat_modifier_operator.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(OperatorChoice(trimmed.to_string()))
        }
    };

    let add_stat_mod_form = column![
        text("Add Stat Modifier").size(16),
        row![
            column![
                text("Stat").size(12),
                pick_list(stat_choices(), selected_stat, |c: StatChoice| {
                    Message::StatModifierStatChanged(c.0)
                })
                .placeholder("Select...")
                .width(Length::Fixed(220.0)),
            ]
            .spacing(4),
            column![
                text("Operator").size(12),
                pick_list(
                    operator_choices(),
                    selected_operator,
                    |c: OperatorChoice| Message::StatModifierOperatorChanged(c.0)
                )
                .placeholder("Select...")
                .width(Length::Fixed(220.0)),
            ]
            .spacing(4),
            column![
                text("Value").size(12),
                text_input("Value", &app.stat_modifier_value)
                    .on_input(Message::StatModifierValueChanged)
                    .padding(8)
                    .width(Length::Fixed(220.0)),
            ]
            .spacing(4),
            button("Add").on_press(Message::CreateLevelStatModifier),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::End),
    ]
    .spacing(8);

    let form = column![
        add_assoc_row,
        add_assoc_filter_row,
        horizontal_rule(1),
        row![
            labeled_input("Name", &app.level_name, Message::LevelNameChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        row![
            labeled_multiline_input("Text", &app.level_text, Message::LevelTextChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        horizontal_rule(1),
        armor_mods_col,
        horizontal_rule(1),
        stat_mods_col,
        horizontal_rule(1),
        add_stat_mod_form,
        horizontal_rule(1),
        text("Add Armor Modifier").size(16),
        row![
            labeled_input(
                "Value",
                &app.armor_modifier_value,
                Message::ArmorModifierValueChanged
            ),
            labeled_pick_list(
                "Suit",
                suit_choices(),
                &app.armor_modifier_suit,
                Message::ArmorModifierSuitChanged
            ),
            labeled_pick_list_damage_type(
                "Damage Type",
                damage_type_choices(),
                &app.armor_modifier_damage_type,
                Message::ArmorModifierDamageTypeChanged
            ),
            button("Add").on_press(Message::CreateArmorModifier),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::End),
        horizontal_rule(1),
        associated_list,
    ]
    .spacing(10);

    // Let the parent/root scroll view measure this content naturally.
    column![header, horizontal_rule(1), form].spacing(12).into()
}

pub fn list_levels(conn: &rusqlite::Connection) -> Result<Vec<LevelRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT name, text
        FROM levels
        ORDER BY name ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(LevelRow {
            name: row.get(0)?,
            text: row.get(1)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn labeled_input<'a>(
    label: &'static str,
    value: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
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

fn labeled_pick_list<'a>(
    label: &'static str,
    choices: Vec<SuitChoice>,
    selected_value: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    let selected = {
        let trimmed = selected_value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(SuitChoice(trimmed.to_string()))
        }
    };

    column![
        text(label).size(12),
        pick_list(choices, selected, move |c: SuitChoice| on_change(c.0))
            .placeholder("Select...")
            .width(Length::Fixed(220.0)),
    ]
    .spacing(4)
    .into()
}

fn labeled_pick_list_damage_type<'a>(
    label: &'static str,
    choices: Vec<DamageTypeChoice>,
    selected_value: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    let selected = {
        let trimmed = selected_value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(DamageTypeChoice(trimmed.to_string()))
        }
    };

    column![
        text(label).size(12),
        pick_list(choices, selected, move |c: DamageTypeChoice| on_change(c.0))
            .placeholder("Select...")
            .width(Length::Fixed(220.0)),
    ]
    .spacing(4)
    .into()
}

fn labeled_multiline_input<'a>(
    label: &'static str,
    value: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    // Iced `text_input` is single-line in most configurations.
    // This still lets you edit longer text; it will scroll horizontally.
    // If you want true multi-line editing, we'd swap to a dedicated multi-line widget.
    column![
        text(label).size(12),
        text_input(label, value)
            .on_input(on_change)
            .padding(8)
            .width(Length::Fixed(700.0)),
    ]
    .spacing(4)
    .into()
}

fn preview_text(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return "(empty)".to_string();
    }

    let mut one_line = trimmed.replace('\n', " ").replace('\r', " ");
    // Collapse some repeated whitespace without pulling in regex.
    while one_line.contains("  ") {
        one_line = one_line.replace("  ", " ");
    }

    const MAX: usize = 120;
    if one_line.len() > MAX {
        format!("{}…", &one_line[..MAX])
    } else {
        one_line
    }
}
