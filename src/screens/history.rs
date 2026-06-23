use crate::components::format_selector;
use crate::state::HistoryEntry;
use crate::style;
use crate::theme;
use iced::{
    widget::{button, column, container, row, scrollable, text, Column},
    Length, Padding,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum HistoryMessage {
    ClearHistory,
    Redownload(Uuid),
    CopyUrl(Uuid),
}

#[derive(Default)]
pub struct HistoryState {
    pub entries: Vec<HistoryEntry>,
}

fn history_card(entry: &HistoryEntry) -> container::Container<'static, HistoryMessage> {
    let date_str = entry.downloaded_at.format("%Y-%m-%d %H:%M").to_string();
    let icon = if entry.success { "✅" } else { "❌" };

    container(
        row![
            text(icon).size(16),
            column![
                text(entry.title.clone())
                    .size(theme::TEXT_MD)
                    .color(theme::FG)
                    .width(Length::Fill),
                row![
                    text(date_str).size(theme::TEXT_XS).color(theme::DIM_TEXT),
                    text(format_selector::format_label(&entry.format))
                        .size(theme::TEXT_XS)
                        .color(theme::DIM_TEXT),
                    text(entry.file_path.display().to_string())
                        .size(theme::TEXT_XS)
                        .color(theme::DIM_TEXT)
                        .width(Length::Fill),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(2)
            .width(Length::Fill),
            button(text("▶ Re-DL").size(theme::TEXT_XS))
                .style(style::button_primary)
                .on_press(HistoryMessage::Redownload(entry.id))
                .padding(Padding::new(8.0).left(14.0).right(14.0)),
            button(text("📋 Copy URL").size(theme::TEXT_XS))
                .style(style::button_ghost)
                .on_press(HistoryMessage::CopyUrl(entry.id))
                .padding(Padding::new(8.0).left(14.0).right(14.0)),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    )
    .style(style::container_card)
    .padding(Padding::new(theme::PADDING_LG))
    .width(Length::Fill)
}

pub fn view(state: &HistoryState) -> Column<'static, HistoryMessage> {
    let mut header = row![text("📋 Download History")
        .size(theme::TEXT_LG)
        .color(theme::HEADER_FG),]
    .spacing(theme::SPACING)
    .align_y(iced::Alignment::Center);

    if !state.entries.is_empty() {
        header = header.push(
            button(text("🗑 Clear All").size(theme::TEXT_XS))
                .style(style::button_ghost)
                .on_press(HistoryMessage::ClearHistory)
                .padding(Padding::new(4.0).left(8.0).right(8.0)),
        );
    }

    let mut content = column![header]
        .spacing(theme::SPACING_LG)
        .width(Length::Fill);

    if state.entries.is_empty() {
        let empty = container(
            text("No download history yet.")
                .size(theme::TEXT_MD)
                .color(theme::DIM_TEXT),
        )
        .padding(Padding::new(32.0))
        .width(Length::Fill)
        .center_x(Length::Fill);

        content = content.push(empty);
    } else {
        for entry in &state.entries {
            content = content.push(history_card(entry));
        }
    }

    column![scrollable(
        container(content)
            .width(Length::Fill)
            .padding(Padding::new(theme::PADDING)),
    )
    .style(style::scrollable_default)
    .width(Length::Fill)
    .height(Length::Fill)]
    .width(Length::Fill)
    .height(Length::Fill)
}
