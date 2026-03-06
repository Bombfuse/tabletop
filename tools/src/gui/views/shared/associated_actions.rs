use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::gui::{ActionRow, Message};

/// Renders a "Associated Actions" section for a card (Unit/Item/Level).
///
/// - `associated` should already be filtered to only the actions associated with the current card.
/// - `on_remove` should create the appropriate remove message for the specific card type.
///   For example:
///   - Unit: `|name| Message::RemoveUnitAssociation(name)`
///   - Item: `|name| Message::RemoveItemAssociation(name)`
///   - Level:`|name| Message::RemoveLevelAssociation(name)`
pub fn view<'a, F>(associated: &[&'a ActionRow], on_remove: F) -> Element<'a, Message>
where
    F: Fn(String) -> Message + Copy + 'a,
{
    let mut associated_sorted: Vec<&'a ActionRow> = associated.to_vec();
    associated_sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let mut out = column![text("Associated Actions").size(16)].spacing(8);

    if associated_sorted.is_empty() {
        out = out.push(text("(none)").size(12));
        return out.into();
    }

    for a in associated_sorted {
        let row_el = row![
            text(a.name.clone()).size(14),
            iced::widget::Space::with_width(Length::Fill),
            button("View").on_press(Message::EditAction(a.name.clone())),
            button("Remove").on_press(on_remove(a.name.clone())),
        ]
        .spacing(12)
        .align_items(Alignment::Center);

        out = out.push(container(row_el).padding(6));
    }

    out.into()
}

/// Helper to collect associated actions for a Unit by matching `ActionRow.unit_name`.
pub fn collect_for_unit<'a>(all_actions: &'a [ActionRow], unit_name: &str) -> Vec<&'a ActionRow> {
    all_actions
        .iter()
        .filter(|a| a.unit_name.as_deref() == Some(unit_name))
        .collect()
}

/// Helper to collect associated actions for an Item by matching `ActionRow.item_name`.
pub fn collect_for_item<'a>(all_actions: &'a [ActionRow], item_name: &str) -> Vec<&'a ActionRow> {
    all_actions
        .iter()
        .filter(|a| a.item_name.as_deref() == Some(item_name))
        .collect()
}

/// Helper to collect associated actions for a Level by matching `ActionRow.level_name`.
pub fn collect_for_level<'a>(all_actions: &'a [ActionRow], level_name: &str) -> Vec<&'a ActionRow> {
    all_actions
        .iter()
        .filter(|a| a.level_name.as_deref() == Some(level_name))
        .collect()
}
