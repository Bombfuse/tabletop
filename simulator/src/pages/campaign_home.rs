use iced::alignment::Horizontal;
use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::Message;

/// Renders a minimal "Campaign Home" page for a selected campaign.
///
/// For now, this is a placeholder that confirms which campaign was loaded.
/// You can expand this later to show hero details, current location, quest log, etc.
pub fn view(campaign_id: i64) -> Element<'static, Message> {
    let heading = text("Campaign")
        .size(40)
        .horizontal_alignment(Horizontal::Center);

    let note = text(format!("Loaded campaign #{campaign_id}"))
        .size(18)
        .horizontal_alignment(Horizontal::Center);

    let back = button(text("Back to Menu").size(18))
        .padding(12)
        .width(Length::Fixed(200.0))
        .on_press(Message::BackToMenu);

    let content = column![heading, note, back]
        .spacing(18)
        .align_items(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(24)
        .into()
}
