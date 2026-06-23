use crate::state::Screen;
use crate::style;
use crate::theme;
use iced::{
    widget::{button, column, container, row, text, Column},
    Color, Length, Padding,
};

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    Navigate(Screen),
}

pub fn view(current_screen: Screen) -> Column<'static, SidebarMessage> {
    let logo = column![
        text("Omadlp").size(theme::TEXT_XL).color(theme::ACCENT),
        text("yt-dlp GUI")
            .size(theme::TEXT_XS)
            .color(theme::DIM_TEXT),
    ]
    .spacing(2);

    let logo_area = container(logo)
        .padding(Padding::new(16.0))
        .width(Length::Fill);

    let divider = container(
        // Thin horizontal divider line
        container(text(""))
            .height(1)
            .width(Length::Fill)
            .style(|_t: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(theme::BORDER_SUBTLE)),
                text_color: None,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: false,
            }),
    )
    .padding(Padding::new(0.0).left(12.0).right(12.0))
    .width(Length::Fill);

    let nav_items: [(Screen, &str); 3] = [
        (Screen::Home, "Home"),
        (Screen::History, "History"),
        (Screen::Settings, "Settings"),
    ];

    let nav_icons: [&str; 3] = ["🏠", "📋", "⚙"];

    let buttons: Column<'static, SidebarMessage> = nav_items.into_iter().enumerate().fold(
        column![]
            .spacing(2)
            .padding(Padding::new(8.0).left(4.0).right(4.0)),
        |col, (i, (screen, label))| {
            let is_active = screen == current_screen;

            let btn_content = row![
                text(nav_icons[i]).size(14),
                text(label).size(theme::TEXT_MD).color(if is_active {
                    theme::ACCENT
                } else {
                    theme::DIM_TEXT
                }),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill);

            let mut btn = button(
                container(btn_content)
                    .width(Length::Fill)
                    .padding(Padding::new(8.0)),
            )
            .style(style::button_sidebar)
            .width(Length::Fill);

            if !is_active {
                btn = btn.on_press(SidebarMessage::Navigate(screen));
            }

            let wrapped = container(btn)
                .style(if is_active {
                    style::container_sidebar_active
                } else {
                    |_theme: &iced::Theme| container::Style {
                        background: None,
                        text_color: None,
                        border: iced::Border {
                            color: Color::TRANSPARENT,
                            width: 3.0,
                            radius: iced::border::Radius::new(0.0),
                        },
                        shadow: iced::Shadow::default(),
                        snap: false,
                    }
                })
                .padding(Padding::new(0.0))
                .width(Length::Fill);

            col.push(wrapped)
        },
    );

    column![logo_area, divider, buttons]
        .width(200)
        .height(Length::Fill)
}
