use anyhow::Result;
use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::gui::{ItemRow, Message, ToolsGui};

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
        text("Create Item").size(18),
        row![
            labeled_input("Name", &app.item_name, Message::ItemNameChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
        row![button("Create").on_press(Message::CreateItem)].spacing(12),
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
            text(it.name.clone()).size(16),
            iced::widget::Space::with_width(Length::Fill),
            button("Edit").on_press(Message::EditItem(it.name.clone())),
            button("Delete").on_press(Message::DeleteItem(it.name.clone())),
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

    // Fully list all associated actions at the bottom, each with a Remove button.
    //
    // NOTE: This relies on the loaded actions list including association info.
    // We treat every action whose `item_name == original_name` as associated.
    let mut associated: Vec<&crate::gui::ActionRow> = app
        .actions
        .iter()
        .filter(|a| a.item_name.as_deref() == Some(original_name))
        .collect();
    associated.sort_by(|a, b| a.name.cmp(&b.name));

    let mut associated_list = column![text("Associated Actions").size(16)].spacing(8);

    if associated.is_empty() {
        associated_list = associated_list.push(text("(none)").size(12));
    } else {
        for a in associated {
            let row_el = row![
                text(a.name.clone()).size(14),
                iced::widget::Space::with_width(Length::Fill),
                button("Remove").on_press(Message::RemoveItemAssociation(a.name.clone())),
            ]
            .spacing(12)
            .align_items(Alignment::Center);

            associated_list = associated_list.push(container(row_el).padding(6));
        }
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
