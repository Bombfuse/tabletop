use anyhow::Result;
use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::gui::views::shared::associated_actions;
use crate::gui::{ItemRow, Message, ToolsGui};

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
struct ActionNameChoice(String);

impl std::fmt::Display for ActionNameChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingArmorModifierRow {
    pub value: i64,
    pub suit: String,
    pub damage_type: String,
}

fn normalize_for_match(s: &str) -> String {
    s.trim().to_lowercase()
}

pub fn view(app: &ToolsGui) -> Element<'_, Message> {
    // Create form: include optional "Add Action" association input (same UX as edit view).
    // The association is applied when creating the item (if an action is selected).
    let filter = normalize_for_match(&app.item_assoc_action_name);

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
        let trimmed = app.item_assoc_action_name.trim();
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
            |choice: ActionNameChoice| Message::ItemAssocActionNameChanged(choice.0)
        )
        .placeholder("Select...")
        .width(Length::Fixed(320.0)),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let add_assoc_filter_row = row![
        text("Filter").width(Length::Fixed(140.0)),
        text_input("type to filter", &app.item_assoc_action_name)
            .on_input(Message::ItemAssocActionNameChanged)
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

    let form = column![
        text("Create Item").size(18),
        row![
            labeled_input("Name", &app.item_name, Message::ItemNameChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        horizontal_rule(1),
        add_assoc_row,
        add_assoc_filter_row,
        row![button("Create").on_press(Message::CreateItemAndMaybeAssociate)].spacing(12),
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
        .spacing(12),
        row![
            button("Queue Armor Modifier").on_press(Message::AddPendingArmorModifier),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        pending_col,
    ]
    .spacing(10);

    let list_header = row![
        text("Items").size(18),
        iced::widget::Space::with_width(Length::Fill),
        text(format!("{} total", app.items.len())).size(14),
    ];

    let mut list_col = column![list_header].spacing(8);

    for it in &app.items {
        let row_el = row![
            column![text(it.name.clone()).size(16)].spacing(2),
            iced::widget::Space::with_width(Length::Fill),
            button("Edit").on_press(Message::EditItem(it.name.clone())),
            button("Delete").on_press(Message::DeleteItem(it.name.clone())),
        ]
        .spacing(12);

        list_col = list_col.push(container(row_el).padding(8));
    }

    // Let the parent/root scroll view measure this content naturally.
    // If we force `Length::Fill` here, it can behave like "infinite height"
    // and fight the root scroll container.
    column![form, horizontal_rule(1), list_col]
        .spacing(12)
        .into()
}

pub fn edit_view<'a>(app: &'a ToolsGui, original_name: &'a str) -> Element<'a, Message> {
    let header = row![
        text(format!("Edit Item: {original_name}")).size(18),
        iced::widget::Space::with_width(Length::Fill),
        button("Cancel").on_press(Message::CancelEdit),
        button("Save").on_press(Message::SaveItemEdits),
    ]
    .spacing(12);

    // Add association: pick an Action name (dropdown) and click "Add".
    //
    // We filter choices using whatever the user has typed into the association box.
    // This approximates an "autocomplete dropdown" without a dedicated widget.
    let filter = normalize_for_match(&app.item_assoc_action_name);

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
        let trimmed = app.item_assoc_action_name.trim();
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
            |choice: ActionNameChoice| Message::ItemAssocActionNameChanged(choice.0)
        )
        .placeholder("Select...")
        .width(Length::Fixed(320.0)),
        button("Add").on_press(Message::AddItemAssociation),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let add_assoc_filter_row = row![
        text("Filter").width(Length::Fixed(140.0)),
        text_input("type to filter", &app.item_assoc_action_name)
            .on_input(Message::ItemAssocActionNameChanged)
            .padding(8)
            .width(Length::Fixed(320.0)),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    // Fully list all associated actions at the bottom (shared component).
    let associated = associated_actions::collect_for_item(&app.actions, original_name);
    let associated_list =
        associated_actions::view(&associated, |name| Message::RemoveItemAssociation(name));

    // List armor modifiers associated with this item.
    let mut armor_mods_col = column![text("Armor Modifiers").size(16)].spacing(8);
    let mut any = false;

    for am in &app.armor_modifiers {
        if am.item_name.as_deref() == Some(original_name) {
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

    let form = column![
        add_assoc_row,
        add_assoc_filter_row,
        horizontal_rule(1),
        row![
            labeled_input("Name", &app.item_name, Message::ItemNameChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        horizontal_rule(1),
        armor_mods_col,
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

pub fn list_items(conn: &rusqlite::Connection) -> Result<Vec<ItemRow>> {
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
