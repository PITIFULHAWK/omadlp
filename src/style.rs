use crate::theme;
use iced::{
    border::Radius,
    widget::{button, container, pick_list, progress_bar, scrollable, text_input},
    Background, Border, Color, Shadow, Vector,
};

// ── Container Styles ──

pub fn container_root(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::BG)),
        text_color: None,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn container_card(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::SURFACE)),
        text_color: None,
        border: Border {
            color: theme::BORDER_SUBTLE,
            width: 1.0,
            radius: Radius::new(theme::CORNER_RADIUS),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        snap: false,
    }
}

pub fn container_content(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::BG)),
        text_color: None,
        border: Border {
            color: theme::BORDER_SUBTLE,
            width: 1.0,
            radius: Radius::new(0.0),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn container_sidebar_active(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::SURFACE)),
        text_color: None,
        border: Border {
            color: theme::ACCENT,
            width: 3.0,
            radius: Radius::new(0.0),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn container_pill<'a>(color: Color) -> impl Fn(&iced::Theme) -> container::Style + 'a {
    move |_theme: &iced::Theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(
            color.r, color.g, color.b, 0.15,
        ))),
        text_color: None,
        border: Border {
            color: Color::from_rgba(color.r, color.g, color.b, 0.3),
            width: 1.0,
            radius: Radius::new(10.0),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

// ── Button Styles ──

fn accent_bright() -> Color {
    Color::from_rgb(
        0xc5 as f32 / 255.0,
        0xa7 as f32 / 255.0,
        0xa0 as f32 / 255.0,
    )
}

fn error_bright() -> Color {
    Color::from_rgb(
        0xff as f32 / 255.0,
        0x55 as f32 / 255.0,
        0x55 as f32 / 255.0,
    )
}

pub fn button_primary(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Active | button::Status::Pressed => theme::ACCENT,
        button::Status::Hovered => accent_bright(),
        button::Status::Disabled => theme::BORDER,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: theme::FG,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::new(theme::CORNER_RADIUS),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn button_danger(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Active | button::Status::Pressed => theme::ERROR,
        button::Status::Hovered => error_bright(),
        button::Status::Disabled => theme::BORDER,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: theme::FG,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::new(theme::CORNER_RADIUS),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn button_ghost(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered => (
            Some(Background::Color(Color::from_rgba(
                theme::ACCENT.r,
                theme::ACCENT.g,
                theme::ACCENT.b,
                0.15,
            ))),
            theme::ACCENT,
        ),
        button::Status::Pressed => (
            Some(Background::Color(Color::from_rgba(
                theme::ACCENT.r,
                theme::ACCENT.g,
                theme::ACCENT.b,
                0.25,
            ))),
            accent_bright(),
        ),
        _ => (None, theme::DIM_TEXT),
    };
    button::Style {
        background: bg,
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::new(theme::CORNER_RADIUS),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn button_sidebar(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(Color::from_rgba(
            theme::ACCENT.r,
            theme::ACCENT.g,
            theme::ACCENT.b,
            0.10,
        ))),
        _ => None,
    };
    let text_color = match status {
        button::Status::Hovered => theme::FG,
        _ => theme::DIM_TEXT,
    };
    button::Style {
        background: bg,
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::new(theme::CORNER_RADIUS),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

// ── TextInput Style ──

pub fn text_input_default(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let focused = matches!(status, text_input::Status::Focused { .. });
    text_input::Style {
        background: Background::Color(theme::BG),
        border: Border {
            color: if focused {
                theme::ACCENT
            } else {
                theme::BORDER_SUBTLE
            },
            width: theme::BORDER_WIDTH,
            radius: Radius::new(theme::CORNER_RADIUS),
        },
        icon: theme::DIM_TEXT,
        placeholder: theme::DIM_TEXT,
        value: theme::FG,
        selection: theme::FG,
    }
}

// ── Scrollable Style ──

pub fn scrollable_default(_theme: &iced::Theme, _status: scrollable::Status) -> scrollable::Style {
    scrollable::Style {
        container: container::Style {
            background: None,
            text_color: None,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: Radius::new(2.0),
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)),
                border: Border::default(),
            },
        },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(Color::TRANSPARENT),
            border: Border::default(),
            shadow: Shadow::default(),
            icon: Color::TRANSPARENT,
        },
    }
}

// ── ProgressBar Style ──

pub fn progress_bar_download(_theme: &iced::Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(Color::from_rgb(
            0x0c as f32 / 255.0,
            0x0b as f32 / 255.0,
            0x0c as f32 / 255.0,
        )),
        bar: Background::Color(theme::ACCENT),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::new(theme::CORNER_RADIUS),
        },
    }
}

// ── PickList Style ──

pub fn pick_list_default(_theme: &iced::Theme, _status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        placeholder_color: theme::DIM_TEXT,
        text_color: theme::FG,
        background: Background::Color(theme::SURFACE),
        border: Border {
            color: theme::BORDER_SUBTLE,
            width: 1.0,
            radius: Radius::new(theme::CORNER_RADIUS),
        },
        handle_color: theme::DIM_TEXT,
    }
}
