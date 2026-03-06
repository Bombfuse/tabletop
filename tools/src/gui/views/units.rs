use anyhow::Result;
use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::gui::views::shared::associated_actions;
use crate::gui::{Message, ToolsGui, UnitRow};

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
    // Create form: include optional "Add Action" association input (same UX as edit view).
    // Note: association is performed when you click "Add" (after create).
    // The actual DB link happens via `Message::AddUnitAssociation`, which expects you to be
    // in an edit context, so this is an input-only affordance until the unit exists.
    let filter = normalize_for_match(&app.unit_assoc_action_name);
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
        let trimmed = app.unit_assoc_action_name.trim();
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
            |choice: ActionNameChoice| Message::UnitAssocActionNameChanged(choice.0)
        )
        .placeholder("Select...")
        .width(Length::Fixed(320.0)),
        button("Add").on_press(Message::AddUnitAssociation),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let add_assoc_filter_row = row![
        text("Filter").width(Length::Fixed(140.0)),
        text_input("type to filter", &app.unit_assoc_action_name)
            .on_input(Message::UnitAssocActionNameChanged)
            .padding(8)
            .width(Length::Fixed(320.0)),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let form = column![
        text("Create Unit").size(18),
        row![
            labeled_input("Name", &app.unit_name, Message::UnitNameChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        row![
            labeled_input("Strength", &app.unit_strength, Message::UnitStrengthChanged),
            labeled_input("Focus", &app.unit_focus, Message::UnitFocusChanged),
            labeled_input(
                "Intelligence",
                &app.unit_intelligence,
                Message::UnitIntelligenceChanged
            ),
        ]
        .spacing(12),
        row![
            labeled_input("Agility", &app.unit_agility, Message::UnitAgilityChanged),
            labeled_input(
                "Knowledge",
                &app.unit_knowledge,
                Message::UnitKnowledgeChanged
            ),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        horizontal_rule(1),
        add_assoc_row,
        add_assoc_filter_row,
        row![button("Create").on_press(Message::CreateUnitAndMaybeAssociate)].spacing(12),
    ]
    .spacing(10);

    let list_header = row![
        text("Units").size(18),
        iced::widget::Space::with_width(Length::Fill),
        text(format!("{} total", app.units.len())).size(14),
    ];

    let mut list_col = column![list_header].spacing(8);

    for u in &app.units {
        let stats = format!(
            "STR {}  FOC {}  INT {}  AGI {}  KNO {}",
            u.strength, u.focus, u.intelligence, u.agility, u.knowledge
        );

        let row_el = row![
            column![text(u.name.clone()).size(16), text(stats).size(12)].spacing(2),
            iced::widget::Space::with_width(Length::Fill),
            button("Edit").on_press(Message::EditUnit(u.name.clone())),
            button("Delete").on_press(Message::DeleteUnit(u.name.clone())),
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
        text(format!("Edit Unit: {original_name}")).size(18),
        iced::widget::Space::with_width(Length::Fill),
        button("Cancel").on_press(Message::CancelEdit),
        button("Save").on_press(Message::SaveUnitEdits),
    ]
    .spacing(12);

    // Add association: pick an Action name (dropdown) and click "Add".
    //
    // We filter choices using whatever the user has typed into the association box.
    // This approximates an "autocomplete dropdown" without a dedicated widget.
    let filter = normalize_for_match(&app.unit_assoc_action_name);

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

    // Current selection from the Unit edit buffer.
    let selected_action = {
        let trimmed = app.unit_assoc_action_name.trim();
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
            |choice: ActionNameChoice| Message::UnitAssocActionNameChanged(choice.0)
        )
        .placeholder("Select...")
        .width(Length::Fixed(320.0)),
        button("Add").on_press(Message::AddUnitAssociation),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let add_assoc_filter_row = row![
        text("Filter").width(Length::Fixed(140.0)),
        text_input("type to filter", &app.unit_assoc_action_name)
            .on_input(Message::UnitAssocActionNameChanged)
            .padding(8)
            .width(Length::Fixed(320.0)),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    // Fully list all associated actions at the bottom (shared component).
    let associated = associated_actions::collect_for_unit(&app.actions, original_name);
    let associated_list =
        associated_actions::view(&associated, |name| Message::RemoveUnitAssociation(name));

    let form = column![
        add_assoc_row,
        add_assoc_filter_row,
        horizontal_rule(1),
        row![
            labeled_input("Name", &app.unit_name, Message::UnitNameChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        row![
            labeled_input("Strength", &app.unit_strength, Message::UnitStrengthChanged),
            labeled_input("Focus", &app.unit_focus, Message::UnitFocusChanged),
            labeled_input(
                "Intelligence",
                &app.unit_intelligence,
                Message::UnitIntelligenceChanged
            ),
        ]
        .spacing(12),
        row![
            labeled_input("Agility", &app.unit_agility, Message::UnitAgilityChanged),
            labeled_input(
                "Knowledge",
                &app.unit_knowledge,
                Message::UnitKnowledgeChanged
            ),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        horizontal_rule(1),
        associated_list,
    ]
    .spacing(10);

    // Let the parent/root scroll view measure this content naturally.
    column![header, horizontal_rule(1), form].spacing(12).into()
}

pub fn list_units(conn: &rusqlite::Connection) -> Result<Vec<UnitRow>> {
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
