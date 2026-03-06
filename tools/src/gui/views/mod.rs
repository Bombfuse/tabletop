pub mod actions;
pub mod armor_modifiers;
pub mod hex_grids;
pub mod items;
pub mod levels;
pub mod units;

pub mod shared;

use iced::widget::{button, row, text};
use iced::{Element, Length};

use crate::gui::{Message, Tab};

pub fn tab_button(current: Tab, tab: Tab) -> iced::widget::Button<'static, Message> {
    let label = tab.label();
    let mut b = button(label);

    if current != tab {
        b = b.on_press(Message::SwitchTab(tab));
    }

    b
}

pub fn status_bar<'a>(status: Option<&'a str>) -> Element<'a, Message> {
    match status {
        Some(s) if !s.trim().is_empty() => row![
            text(s.to_string()).size(14),
            iced::widget::Space::with_width(Length::Fill),
            button("Dismiss").on_press(Message::ClearStatus)
        ]
        .spacing(12)
        .into(),
        _ => row![text("")].into(),
    }
}
