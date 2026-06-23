mod app;
mod components;
mod engine;
mod persistence;
mod screens;
mod state;
mod style;
mod theme;

use iced::{window, Size};

fn theme(_: &app::State) -> iced::Theme {
    iced::Theme::Dark
}

fn main() -> iced::Result {
    env_logger::init();

    iced::application(
        app::new,
        app::update as fn(&mut app::State, app::Message) -> iced::Task<app::Message>,
        app::view as for<'a> fn(&'a app::State) -> iced::Element<'a, app::Message>,
    )
    .title(app::title as fn(&app::State) -> String)
    .subscription(app::subscription)
    .theme(theme as fn(&app::State) -> iced::Theme)
    .window(window::Settings {
        size: Size::new(1000.0, 680.0),
        position: window::Position::Centered,
        decorations: false,
        transparent: true,
        resizable: true,
        ..window::Settings::default()
    })
    .run()
}
