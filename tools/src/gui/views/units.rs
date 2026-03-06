use anyhow::Result;
use iced::widget::{button, column, container, horizontal_rule, row, text, text_input};
use iced::{Element, Length};

use crate::gui::{Message, ToolsGui, UnitRow};

pub fn view(app: &ToolsGui) -> Element<'_, Message> {
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
        row![button("Create").on_press(Message::CreateUnit)].spacing(12),
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

    let form = column![
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
