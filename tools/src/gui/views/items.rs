use anyhow::Result;
use iced::widget::{button, column, container, horizontal_rule, row, text, text_input};
use iced::{Element, Length};

use crate::gui::{ItemRow, Message, ToolsGui};

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

    let form = column![
        row![
            labeled_input("Name", &app.item_name, Message::ItemNameChanged),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
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
