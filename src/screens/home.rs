use crate::components::{
    format_selector::{self, FormatMessage},
    progress_card::{self, ProgressMessage},
    url_input::{self, UrlInputMessage},
};
use crate::state::DownloadStatus;
use crate::style;
use crate::theme;
use iced::{
    widget::{button, column, container, row, scrollable, text, Column},
    Element, Length, Padding,
};

#[derive(Debug, Clone)]
pub enum HomeMessage {
    UrlInput(UrlInputMessage),
    FormatSelector(FormatMessage),
    ProgressAction(ProgressMessage),
    ClearCompleted,
}

pub struct HomeState {
    pub url_input: String,
    pub queue: Vec<crate::state::QueueEntry>,
    pub selected_format: String,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            url_input: String::new(),
            queue: Vec::new(),
            selected_format: "bestvideo+bestaudio/best".to_string(),
        }
    }
}

impl HomeState {
    pub fn add_to_queue(&mut self) {
        let url = self.url_input.trim().to_string();
        if url.is_empty() {
            return;
        }
        let mut entry = crate::state::QueueEntry::new(url.clone());
        entry.format = self.selected_format.clone();
        entry.status = DownloadStatus::FetchingFormats;
        self.queue.push(entry);
        self.url_input.clear();
    }
}

fn add_btn() -> Element<'static, HomeMessage> {
    button(
        container(text("+ Add to Queue").size(theme::TEXT_MD))
            .padding(Padding::new(8.0).left(16.0).right(16.0)),
    )
    .style(style::button_primary)
    .on_press(HomeMessage::UrlInput(UrlInputMessage::Submit))
    .into()
}

pub fn view(state: &HomeState) -> Column<'static, HomeMessage> {
    let url_section = container(
        column![
            text("🔗 Enter URL")
                .size(theme::TEXT_LG)
                .color(theme::HEADER_FG),
            {
                let v: Element<'_, UrlInputMessage> = url_input::view(&state.url_input).into();
                v.map(HomeMessage::UrlInput)
            },
            row![
                {
                    let v: Element<'_, FormatMessage> =
                        format_selector::view(&state.selected_format).into();
                    v.map(HomeMessage::FormatSelector)
                },
                add_btn(),
            ]
            .spacing(theme::SPACING)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
        ]
        .spacing(theme::SPACING)
        .width(Length::Fill),
    )
    .style(style::container_card)
    .padding(Padding::new(theme::PADDING_LG))
    .width(Length::Fill);

    let mut content = column![url_section]
        .spacing(theme::SPACING_LG)
        .width(Length::Fill);

    if state.queue.is_empty() {
        let empty = container(
            text("No downloads yet. Paste a URL above to start.")
                .size(theme::TEXT_MD)
                .color(theme::DIM_TEXT),
        )
        .padding(Padding::new(32.0))
        .width(Length::Fill)
        .center_x(Length::Fill);

        content = content.push(empty);
    } else {
        let has_completed = state.queue.iter().any(|e| {
            matches!(
                e.status,
                DownloadStatus::Completed | DownloadStatus::Failed(_) | DownloadStatus::Cancelled
            )
        });

        let mut header = row![text("📥 Download Queue")
            .size(theme::TEXT_LG)
            .color(theme::HEADER_FG),]
        .spacing(theme::SPACING)
        .align_y(iced::Alignment::Center);

        if has_completed {
            header = header.push(
                button(text("Clear completed").size(theme::TEXT_XS))
                    .style(style::button_ghost)
                    .on_press(HomeMessage::ClearCompleted)
                    .padding(Padding::new(4.0).left(8.0).right(8.0)),
            );
        }

        content = content.push(header);

        for entry in &state.queue {
            let card = progress_card::view(entry).map(HomeMessage::ProgressAction);
            content = content.push(card);
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
