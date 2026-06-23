use crate::style;
use crate::theme;
use iced::{
    widget::{column, pick_list, row, text, Column},
    Length,
};

#[derive(Debug, Clone)]
pub enum FormatMessage {
    FormatSelected(&'static str),
}

pub struct FormatOption {
    pub label: &'static str,
    pub value: &'static str,
    pub description: &'static str,
}

pub static FORMAT_OPTIONS: [FormatOption; 6] = [
    FormatOption {
        label: "Best quality (4K if available)",
        value: "bestvideo+bestaudio/best",
        description: "Highest video and audio quality available, up to 4K/8K",
    },
    FormatOption {
        label: "4K UHD with fallback",
        value: "bestvideo[height<=2160]+bestaudio/bestvideo[height<=1440]+bestaudio/bestvideo[height<=1080]+bestaudio/bestvideo[height<=720]+bestaudio/best",
        description: "Prefer 4K, fall back to 2K → 1080p → 720p automatically",
    },
    FormatOption {
        label: "1080p HD with fallback",
        value: "bestvideo[height<=1080]+bestaudio/bestvideo[height<=720]+bestaudio/best",
        description: "Prefer 1080p, fall back to 720p if unavailable",
    },
    FormatOption {
        label: "720p HD",
        value: "bestvideo[height<=720]+bestaudio/best",
        description: "HD video with smaller file size",
    },
    FormatOption {
        label: "Audio only",
        value: "bestaudio/best",
        description: "Download just the audio track",
    },
    FormatOption {
        label: "Smallest file",
        value: "worst",
        description: "Lowest quality, fastest download",
    },
];

pub fn format_value(label: &str) -> Option<&'static str> {
    FORMAT_OPTIONS
        .iter()
        .find(|o| o.label == label)
        .map(|o| o.value)
}

pub fn format_label(value: &str) -> &'static str {
    FORMAT_OPTIONS
        .iter()
        .find(|o| o.value == value)
        .map(|o| o.label)
        .unwrap_or(FORMAT_OPTIONS[0].label)
}

pub fn format_description(value: &str) -> &'static str {
    FORMAT_OPTIONS
        .iter()
        .find(|o| o.value == value)
        .map(|o| o.description)
        .unwrap_or(FORMAT_OPTIONS[0].description)
}

pub fn view<'a>(selected_value: &str) -> Column<'a, FormatMessage> {
    let selected_label = format_label(selected_value);
    let description = format_description(selected_value);
    let labels: Vec<&'static str> = FORMAT_OPTIONS.iter().map(|o| o.label).collect();

    let pick = pick_list(labels, Some(selected_label), |label| {
        let value = format_value(label).unwrap_or(FORMAT_OPTIONS[0].value);
        FormatMessage::FormatSelected(value)
    })
    .width(Length::Fill)
    .style(style::pick_list_default);

    let label = text("Quality").size(theme::TEXT_SM).color(theme::DIM_TEXT);

    let desc_text = text(description)
        .size(theme::TEXT_XS)
        .color(theme::DIM_TEXT);

    column![
        row![label, pick]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
        desc_text,
    ]
    .spacing(theme::SPACING_SM)
    .width(Length::Fill)
}
