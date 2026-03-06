use anyhow::Result;
use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::gui::{LevelRow, Message, ToolsGui};

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
        row![button("Create").on_press(Message::CreateLevel)].spacing(12),
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

    // Editable association: pick an Action name (dropdown), then explicitly Save/Clear.
    // This is the closest we can get to an "auto complete dropdown" with current widgets:
    // - a pick-list for selection
    // - plus a text box for quick filtering/typing
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

    let assoc_row = row![
        text("Associated Action").width(Length::Fixed(140.0)),
        pick_list(
            action_choices,
            selected_action,
            |choice: ActionNameChoice| Message::LevelAssocActionNameChanged(choice.0)
        )
        .placeholder("(none)")
        .width(Length::Fixed(320.0)),
        button("Save").on_press(Message::SaveLevelAssociation),
        button("Clear").on_press(Message::ClearLevelAssociation),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let assoc_filter_row = row![
        text("Filter").width(Length::Fixed(140.0)),
        text_input(
            "type to filter / exact action name",
            &app.level_assoc_action_name
        )
        .on_input(Message::LevelAssocActionNameChanged)
        .padding(8)
        .width(Length::Fixed(320.0)),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::Center);

    let form = column![
        assoc_row,
        assoc_filter_row,
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
