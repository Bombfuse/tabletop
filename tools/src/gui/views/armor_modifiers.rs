use anyhow::Result;
use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::gui::{ArmorModifierRow, Message, ToolsGui};

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

pub fn view(app: &ToolsGui) -> Element<'_, Message> {
    let form = column![
        text("Create Armor Modifier").size(18),
        row![
            labeled_input(
                "Value",
                "number",
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
        row![button("Create").on_press(Message::CreateArmorModifier)].spacing(12),
    ]
    .spacing(10);

    let list_header = row![
        text("Armor Modifiers").size(18),
        iced::widget::Space::with_width(Length::Fill),
        text(format!("{} total", app.armor_modifiers.len())).size(14),
    ];

    let mut list_col = column![list_header].spacing(8);

    for am in &app.armor_modifiers {
        let assoc = association_label(am);

        let row_el = row![
            column![
                text(format!(
                    "id={}  card_id={}  value={}  suit={}  damage_type={}",
                    am.id, am.card_id, am.value, am.suit, am.damage_type
                ))
                .size(16),
                text(assoc).size(12),
            ]
            .spacing(2),
            iced::widget::Space::with_width(Length::Fill),
            button("Edit").on_press(Message::EditArmorModifier(am.id)),
            button("Delete").on_press(Message::DeleteArmorModifier(am.id)),
        ]
        .spacing(12)
        .align_items(Alignment::Center);

        list_col = list_col.push(container(row_el).padding(8));
    }

    column![form, horizontal_rule(1), list_col]
        .spacing(12)
        .into()
}

pub fn edit_view<'a>(app: &'a ToolsGui, id: i64) -> Element<'a, Message> {
    let header = row![
        text(format!("Edit Armor Modifier: id={id}")).size(18),
        iced::widget::Space::with_width(Length::Fill),
        button("Cancel").on_press(Message::CancelEdit),
        button("Save").on_press(Message::SaveArmorModifierEdits),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let form = column![
        row![
            labeled_input(
                "Value",
                "number",
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
        horizontal_rule(1),
        row![button("Delete").on_press(Message::DeleteArmorModifier(id))].spacing(12),
        text("Note: association editing (Item/Level link) is handled from Item/Level edit views.")
            .size(12),
    ]
    .spacing(10);

    column![header, horizontal_rule(1), form].spacing(12).into()
}

pub fn list_armor_modifiers(conn: &rusqlite::Connection) -> Result<Vec<ArmorModifierRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            am.id,
            am.card_id,
            am.value,
            am.suit,
            am.damage_type,
            i.name AS item_name,
            l.name AS level_name
        FROM armor_modifiers am
        LEFT JOIN item_armor_modifiers iam
            ON iam.armor_modifier_id = am.id
        LEFT JOIN items i
            ON i.id = iam.item_id
        LEFT JOIN level_armor_modifiers lam
            ON lam.armor_modifier_id = am.id
        LEFT JOIN levels l
            ON l.id = lam.level_id
        ORDER BY am.id ASC
        "#,
    )?;

    let rows = stmt.query_map([], |r| {
        Ok(ArmorModifierRow {
            id: r.get(0)?,
            card_id: r.get(1)?,
            value: r.get(2)?,
            suit: r.get(3)?,
            damage_type: r.get(4)?,
            item_name: r.get(5)?,
            level_name: r.get(6)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn association_label(am: &ArmorModifierRow) -> String {
    match (&am.item_name, &am.level_name) {
        (Some(item), None) => format!("Associated Item: {item}"),
        (None, Some(level)) => format!("Associated Level: {level}"),
        (None, None) => "Associated: (none)".to_string(),
        (Some(item), Some(level)) => format!("Associated: INVALID (item={item}, level={level})"),
    }
}

fn labeled_input<'a>(
    label: &'static str,
    placeholder: &'static str,
    value: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    column![
        text(label).size(12),
        text_input(placeholder, value)
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
