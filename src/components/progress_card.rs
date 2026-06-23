use crate::components::format_selector;
use crate::state::{DownloadStatus, PlaylistItemStatus, QueueEntry};
use crate::style;
use crate::theme;
use iced::{
    widget::{button, column, container, progress_bar, row, text},
    Color, Element, Length, Padding,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ProgressMessage {
    Cancel(Uuid),
    Start(Uuid),
    Retry(Uuid),
    Remove(Uuid),
    TogglePlaylist(Uuid),
}

fn status_info(status: &DownloadStatus) -> (Color, String) {
    match status {
        DownloadStatus::Pending => (theme::DIM_TEXT, "Pending".to_string()),
        DownloadStatus::FetchingFormats => (theme::DIM_TEXT, "Fetching details...".to_string()),
        DownloadStatus::Ready => (theme::ACCENT, "Ready".to_string()),
        DownloadStatus::Downloading => (theme::SUCCESS, "Downloading".to_string()),
        DownloadStatus::Completed => (theme::SUCCESS, "Completed".to_string()),
        DownloadStatus::Failed(msg) => (theme::ERROR, format!("Failed: {}", msg)),
        DownloadStatus::Cancelled => (theme::DIM_TEXT, "Cancelled".to_string()),
    }
}

fn status_icon(status: &DownloadStatus) -> &'static str {
    match status {
        DownloadStatus::Pending | DownloadStatus::Ready => "⏳",
        DownloadStatus::FetchingFormats => "🔍",
        DownloadStatus::Downloading => "⬇",
        DownloadStatus::Completed => "✅",
        DownloadStatus::Failed(_) => "❌",
        DownloadStatus::Cancelled => "🚫",
    }
}

pub fn view(entry: &QueueEntry) -> Element<'static, ProgressMessage> {
    let (badge_color, badge_text) = status_info(&entry.status);
    let icon = status_icon(&entry.status);

    let title_text = text(entry.title.clone().unwrap_or_else(|| entry.url.clone()))
        .size(theme::TEXT_SM)
        .color(theme::FG)
        .width(Length::Fill);

    let badge = container(text(badge_text).size(theme::TEXT_XS).color(badge_color))
        .padding(Padding::new(4.0).left(8.0).right(8.0))
        .style(style::container_pill(badge_color));

    let format_text = text(
        entry
            .selected_quality
            .clone()
            .unwrap_or_else(|| format_selector::format_label(&entry.format).to_string()),
    )
    .size(theme::TEXT_XS)
    .color(theme::DIM_TEXT);

    let info_row = row![badge, format_text]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    let action_btn: Option<Element<'static, ProgressMessage>> = match &entry.status {
        DownloadStatus::Pending | DownloadStatus::Ready => {
            let id = entry.id;
            Some(
                button(text("▶ Download").size(theme::TEXT_SM))
                    .style(style::button_primary)
                    .on_press(ProgressMessage::Start(id))
                    .padding(Padding::new(8.0).left(14.0).right(14.0))
                    .into(),
            )
        }
        DownloadStatus::Downloading => {
            let id = entry.id;
            Some(
                button(text("■ Cancel").size(theme::TEXT_SM))
                    .style(style::button_danger)
                    .on_press(ProgressMessage::Cancel(id))
                    .padding(Padding::new(8.0).left(14.0).right(14.0))
                    .into(),
            )
        }
        DownloadStatus::Failed(_) => {
            let id = entry.id;
            Some(
                row![
                    button(text("↻ Retry").size(theme::TEXT_SM))
                        .style(style::button_primary)
                        .on_press(ProgressMessage::Retry(id))
                        .padding(Padding::new(8.0).left(14.0).right(14.0)),
                    button(text("✕").size(theme::TEXT_MD))
                        .style(style::button_ghost)
                        .on_press(ProgressMessage::Remove(id))
                        .padding(Padding::new(6.0)),
                ]
                .spacing(8)
                .into(),
            )
        }
        _ => {
            let id = entry.id;
            Some(
                button(text("✕").size(theme::TEXT_MD))
                    .style(style::button_ghost)
                    .on_press(ProgressMessage::Remove(id))
                    .padding(Padding::new(6.0))
                    .into(),
            )
        }
    };

    let mut content = column![
        row![text(icon).size(14), title_text,]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
        info_row,
    ]
    .spacing(6);

    if entry.is_playlist {
        let total = entry.playlist_items.len();
        let available = entry
            .playlist_items
            .iter()
            .filter(|item| !matches!(item.status, PlaylistItemStatus::Unavailable(_)))
            .count();
        let completed = entry
            .playlist_items
            .iter()
            .filter(|item| matches!(item.status, PlaylistItemStatus::Completed))
            .count();
        let unavailable = total.saturating_sub(available);
        let label = if entry.playlist_expanded {
            "▾ Hide videos"
        } else {
            "▸ Show videos"
        };
        let summary = if unavailable > 0 {
            format!("{total} videos • {available} available • {unavailable} unavailable")
        } else {
            format!("{total} videos")
        };
        content = content.push(
            row![
                text(summary).size(theme::TEXT_XS).color(theme::DIM_TEXT),
                button(text(label).size(theme::TEXT_XS))
                    .style(style::button_ghost)
                    .on_press(ProgressMessage::TogglePlaylist(entry.id))
                    .padding(Padding::new(3.0).left(7.0).right(7.0)),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );
        if matches!(
            entry.status,
            DownloadStatus::Downloading | DownloadStatus::Completed
        ) && available > 0
        {
            let fractional = entry
                .playlist_items
                .iter()
                .filter(|i| !matches!(i.status, PlaylistItemStatus::Unavailable(_)))
                .map(|i| match i.status {
                    PlaylistItemStatus::Completed => 1.0,
                    PlaylistItemStatus::Downloading => i
                        .progress
                        .as_ref()
                        .map(|p| p.percentage / 100.0)
                        .unwrap_or(0.0),
                    _ => 0.0,
                })
                .sum::<f64>()
                / available as f64;
            content = content.push(
                column![
                    text(format!(
                        "Overall: {completed}/{available} videos ({:.1}%)",
                        fractional * 100.0
                    ))
                    .size(theme::TEXT_XS)
                    .color(theme::DIM_TEXT),
                    container(
                        progress_bar(0.0..=1.0, fractional as f32)
                            .style(style::progress_bar_download)
                    )
                    .height(6),
                ]
                .spacing(4),
            );
        }
        if entry.playlist_expanded {
            let mut videos = column![].spacing(5);
            for item in &entry.playlist_items {
                let (icon, detail, color) = match &item.status {
                    PlaylistItemStatus::Pending => ("○", "Waiting".to_string(), theme::DIM_TEXT),
                    PlaylistItemStatus::Downloading => (
                        "↓",
                        item.progress
                            .as_ref()
                            .map(|p| {
                                format!(
                                    "{:.1}% • {} / {} • {} • ETA {}",
                                    p.percentage, p.downloaded, p.total, p.speed, p.eta
                                )
                            })
                            .unwrap_or_else(|| "Starting...".into()),
                        theme::ACCENT,
                    ),
                    PlaylistItemStatus::Completed => {
                        ("✓", "Downloaded".to_string(), theme::SUCCESS)
                    }
                    PlaylistItemStatus::Unavailable(reason) => ("!", reason.clone(), theme::ERROR),
                    PlaylistItemStatus::Failed(reason) => ("×", reason.clone(), theme::ERROR),
                };
                videos = videos.push(
                    column![
                        row![
                            text(format!("{}.", item.index))
                                .size(theme::TEXT_XS)
                                .color(theme::DIM_TEXT),
                            text(icon).size(theme::TEXT_XS).color(color),
                            text(item.title.clone())
                                .size(theme::TEXT_XS)
                                .color(theme::FG)
                                .width(Length::Fill),
                            text(detail).size(theme::TEXT_XS).color(color),
                        ]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                        if matches!(item.status, PlaylistItemStatus::Downloading) {
                            container(
                                progress_bar(
                                    0.0..=1.0,
                                    item.progress
                                        .as_ref()
                                        .map(|p| p.percentage as f32 / 100.0)
                                        .unwrap_or(0.0),
                                )
                                .style(style::progress_bar_download),
                            )
                            .height(4)
                        } else {
                            container(text("")).height(0)
                        },
                    ]
                    .spacing(2),
                );
            }
            content = content.push(container(videos).padding(Padding::new(8.0).left(12.0)));
        }
    }

    if matches!(entry.status, DownloadStatus::Downloading) && !entry.is_playlist {
        let pct = entry
            .progress
            .as_ref()
            .map(|p| p.percentage as f32 / 100.0)
            .unwrap_or(0.0);

        let progress_text = if let Some(p) = &entry.progress {
            format!(
                "{:.1}%  •  {} / {}  •  {}  •  ETA {}",
                p.percentage, p.downloaded, p.total, p.speed, p.eta
            )
        } else {
            String::from("Starting...")
        };

        content = content.push(
            column![
                container(progress_bar(0.0..=1.0, pct).style(style::progress_bar_download),)
                    .height(6),
                text(progress_text)
                    .size(theme::TEXT_XS)
                    .color(theme::DIM_TEXT),
            ]
            .spacing(4),
        );
    }

    let action_container: Element<'static, ProgressMessage> = if let Some(btn) = action_btn {
        container(btn).into()
    } else {
        container(text("")).into()
    };

    let card_row = row![content.width(Length::Fill), action_container,]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill);

    container(card_row)
        .style(style::container_card)
        .padding(Padding::new(theme::PADDING_LG))
        .width(Length::Fill)
        .into()
}
