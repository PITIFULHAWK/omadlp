use crate::style;
use crate::theme;
use iced::{
    widget::{row, text_input, Row},
    Length, Padding,
};

#[derive(Debug, Clone)]
pub enum UrlInputMessage {
    InputChanged(String),
    Submit,
}

pub fn view(value: &str) -> Row<'static, UrlInputMessage> {
    let input = text_input(
        "Paste a video URL from YouTube, Twitter, or anywhere...",
        value,
    )
    .style(style::text_input_default)
    .on_input(UrlInputMessage::InputChanged)
    .on_submit(UrlInputMessage::Submit)
    .width(Length::Fill)
    .padding(Padding::new(12.0));

    row![input].spacing(theme::SPACING).width(Length::Fill)
}
