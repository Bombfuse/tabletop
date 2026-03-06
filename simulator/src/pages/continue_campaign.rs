use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::db::CampaignSummary;
use crate::types::Message;

/// Renders the "Continue Campaign" page:
/// - shows existing campaigns (loaded by the app from the simulator DB)
/// - allows selecting one to load
pub fn view(campaigns: &[CampaignSummary], load_error: Option<&str>) -> Element<'static, Message> {
    let heading = text("Continue Campaign")
        .size(40)
        .horizontal_alignment(Horizontal::Center);

    let sub = text("Select an existing campaign:")
        .size(18)
        .horizontal_alignment(Horizontal::Center);

    let error = match load_error {
        Some(e) => text(format!("DB error: {e}"))
            .size(16)
            .horizontal_alignment(Horizontal::Center),
        None => text("").size(1),
    };

    let mut list = column![].spacing(10).width(Length::Fill);

    if campaigns.is_empty() && load_error.is_none() {
        list = list.push(
            text("No campaigns found in the simulator database.")
                .size(16)
                .horizontal_alignment(Horizontal::Center),
        );
    } else {
        for c in campaigns {
            let title = text(format!("Campaign #{} — {}", c.id, c.hero_unit_name)).size(18);
            let meta = text(format!("Created: {}", c.created_at)).size(14);

            let select = button(text("Load").size(14))
                .padding(8)
                .on_press(Message::SelectCampaign(c.id));

            let row_item = row![column![title, meta].spacing(4), select]
                .spacing(16)
                .align_items(iced::Alignment::Center);

            list = list.push(container(row_item).padding(10));
        }
    }

    let list = scrollable(container(list).width(Length::Fill)).height(Length::FillPortion(1));

    let back = button(text("Back to Menu").size(18))
        .padding(12)
        .width(Length::Fixed(200.0))
        .on_press(Message::BackToMenu);

    let content = column![heading, sub, error, list, back]
        .spacing(14)
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
