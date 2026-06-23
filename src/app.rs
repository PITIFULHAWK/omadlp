use crate::{
    components::{
        format_selector::FormatMessage,
        progress_card::ProgressMessage,
        sidebar::{self, SidebarMessage},
        url_input::UrlInputMessage,
    },
    engine::download,
    persistence,
    screens::settings::SettingsMessage,
    screens::{
        history::{self, HistoryMessage, HistoryState},
        home::{self, HomeMessage, HomeState},
        settings::{self, SettingsState},
    },
    state::{Config, DownloadStatus, HistoryEntry, QueueEntry, Screen},
    style,
};
use iced::{
    widget::{container, row},
    Element, Length, Subscription, Task,
};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Message {
    Sidebar(SidebarMessage),
    Home(HomeMessage),
    History(HistoryMessage),
    Settings(SettingsMessage),
    MetadataFetched(Uuid, Result<crate::engine::metadata::MediaMetadata, String>),
    DownloadEvent(Uuid, download::DownloadEvent),
    SettingsSaved,
}

pub struct State {
    pub current_screen: Screen,
    pub home: HomeState,
    pub history: HistoryState,
    pub settings: SettingsState,
    pub config: Config,
}

pub fn title(_state: &State) -> String {
    String::from("Omadlp — yt-dlp GUI")
}

pub fn subscription(_state: &State) -> Subscription<Message> {
    Subscription::none()
}

pub fn new() -> (State, Task<Message>) {
    let config = persistence::settings::load();
    let history_entries = persistence::history::load();
    let settings_state = SettingsState::new(&config);

    (
        State {
            current_screen: Screen::Home,
            home: HomeState::default(),
            history: HistoryState {
                entries: history_entries,
            },
            settings: settings_state,
            config,
        },
        Task::none(),
    )
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        // ── Navigation ──
        Message::Sidebar(SidebarMessage::Navigate(screen)) => {
            state.current_screen = screen;
            Task::none()
        }

        // ── Home Screen ──
        Message::Home(msg) => match msg {
            HomeMessage::UrlInput(url_msg) => match url_msg {
                UrlInputMessage::InputChanged(val) => {
                    state.home.url_input = val;
                    Task::none()
                }
                UrlInputMessage::Submit => {
                    state.home.add_to_queue();
                    let last = state.home.queue.last().cloned();
                    if let Some(entry) = last {
                        let url = entry.url.clone();
                        let allow_playlist = state.config.download_playlist;
                        let selected_format = if state.config.audio_only {
                            "bestaudio/best".to_string()
                        } else {
                            download::effective_format(
                                &entry.format,
                                &state.config.video_codec,
                                state.config.prefer_hdr,
                            )
                        };
                        let cookies_browser = state.config.cookies_from_browser.clone();
                        return Task::perform(
                            async move {
                                crate::engine::metadata::fetch_metadata(
                                    &url,
                                    allow_playlist,
                                    &selected_format,
                                    &cookies_browser,
                                )
                            },
                            move |result| Message::MetadataFetched(entry.id, result),
                        );
                    }
                    Task::none()
                }
            },
            HomeMessage::FormatSelector(fmt_msg) => match fmt_msg {
                FormatMessage::FormatSelected(f) => {
                    state.home.selected_format = f.to_string();
                    Task::none()
                }
            },
            HomeMessage::ProgressAction(prog_msg) => match prog_msg {
                ProgressMessage::Start(id) => {
                    if let Some(entry) = state.home.queue.iter_mut().find(|e| e.id == id) {
                        entry.status = DownloadStatus::Downloading;
                        let url = entry.url.clone();
                        let format = entry.format.clone();
                        let dir = state.config.download_directory.display().to_string();
                        let download_id = entry.id;

                        let config = state.config.clone();
                        let is_playlist = entry.is_playlist;
                        let playlist_title = entry.title.clone();
                        let playlist_items = entry
                            .playlist_items
                            .iter()
                            .filter(|item| {
                                !matches!(
                                    item.status,
                                    crate::state::PlaylistItemStatus::Unavailable(_)
                                )
                            })
                            .filter_map(|item| item.url.clone().map(|url| (item.index, url)))
                            .collect();
                        let stream = download::download_stream(
                            url,
                            format,
                            dir,
                            config,
                            is_playlist,
                            playlist_title,
                            playlist_items,
                        );
                        return Task::run(stream, move |event| {
                            Message::DownloadEvent(download_id, event)
                        });
                    }
                    Task::none()
                }
                ProgressMessage::Cancel(id) => {
                    if let Some(entry) = state.home.queue.iter_mut().find(|e| e.id == id) {
                        entry.status = DownloadStatus::Cancelled;
                    }
                    Task::none()
                }
                ProgressMessage::Retry(id) => {
                    if let Some(entry) = state.home.queue.iter_mut().find(|e| e.id == id) {
                        entry.status = DownloadStatus::Ready;
                        entry.progress = None;
                        entry.file_path = None;
                    }
                    Task::done(Message::Home(HomeMessage::ProgressAction(
                        ProgressMessage::Start(id),
                    )))
                }
                ProgressMessage::Remove(id) => {
                    state.home.queue.retain(|e| e.id != id);
                    Task::none()
                }
                ProgressMessage::TogglePlaylist(id) => {
                    if let Some(entry) = state.home.queue.iter_mut().find(|e| e.id == id) {
                        entry.playlist_expanded = !entry.playlist_expanded;
                    }
                    Task::none()
                }
            },
            HomeMessage::ClearCompleted => {
                state.home.queue.retain(|e| {
                    !matches!(
                        e.status,
                        DownloadStatus::Completed
                            | DownloadStatus::Failed(_)
                            | DownloadStatus::Cancelled
                    )
                });
                Task::none()
            }
        },

        // ── History Screen ──
        Message::History(msg) => match msg {
            HistoryMessage::ClearHistory => {
                state.history.entries.clear();
                let history = state.history.entries.clone();
                Task::perform(
                    async move { persistence::history::save(&history).is_ok() },
                    |_| Message::SettingsSaved,
                )
            }
            HistoryMessage::Redownload(id) => {
                if let Some(entry) = state.history.entries.iter().find(|e| e.id == id) {
                    let url = entry.url.clone();
                    let mut queue_entry = QueueEntry::new(url);
                    queue_entry.format = entry.format.clone();
                    queue_entry.status = DownloadStatus::Ready;
                    state.home.queue.push(queue_entry);
                    state.current_screen = Screen::Home;
                }
                Task::none()
            }
            HistoryMessage::CopyUrl(id) => {
                if let Some(entry) = state.history.entries.iter().find(|e| e.id == id) {
                    return iced::clipboard::write(entry.url.clone());
                }
                Task::none()
            }
        },

        // ── Settings Screen ──
        Message::Settings(msg) => match msg {
            SettingsMessage::SelectDirectory => Task::perform(
                async move { rfd::FileDialog::new().pick_folder() },
                |path| Message::Settings(SettingsMessage::DirectorySelected(path)),
            ),
            SettingsMessage::OpenDownloadFolder => {
                let path = PathBuf::from(&state.settings.download_dir_input);
                Task::perform(async move { open::that(path) }, |_| Message::SettingsSaved)
            }
            SettingsMessage::DirectorySelected(Some(path)) => {
                state.settings.download_dir_input = path.display().to_string();
                Task::none()
            }
            SettingsMessage::DirectorySelected(None) => Task::none(),
            SettingsMessage::FormatSelected(value) => {
                state.settings.selected_format = value.to_string();
                Task::none()
            }
            SettingsMessage::ConcurrentIncrement => {
                state.settings.concurrent = state.settings.concurrent.saturating_add(1);
                Task::none()
            }
            SettingsMessage::ConcurrentDecrement => {
                state.settings.concurrent = state.settings.concurrent.saturating_sub(1).max(1);
                Task::none()
            }
            SettingsMessage::ToggleAudioOnly(val) => {
                state.settings.audio_only = val;
                Task::none()
            }
            SettingsMessage::AudioFormatChanged(fmt) => {
                state.settings.extract_audio_format = fmt.to_string();
                Task::none()
            }
            SettingsMessage::ToggleEmbedThumbnail(val) => {
                state.settings.embed_thumbnail = val;
                Task::none()
            }
            SettingsMessage::ToggleWriteMetadata(val) => {
                state.settings.write_metadata = val;
                Task::none()
            }
            SettingsMessage::ToggleDownloadSubtitles(val) => {
                state.settings.download_subtitles = val;
                Task::none()
            }
            SettingsMessage::ToggleSponsorblock(val) => {
                state.settings.sponsorblock_remove = val;
                Task::none()
            }
            SettingsMessage::CookiesBrowserChanged(val) => {
                state.settings.cookies_from_browser = val;
                Task::none()
            }
            SettingsMessage::VideoCodecChanged(val) => {
                state.settings.video_codec = val.to_string();
                Task::none()
            }
            SettingsMessage::TogglePreferHdr(val) => {
                state.settings.prefer_hdr = val;
                Task::none()
            }
            SettingsMessage::ToggleDownloadPlaylist(val) => {
                state.settings.download_playlist = val;
                Task::none()
            }
            SettingsMessage::RateLimitChanged(val) => {
                state.settings.rate_limit = val;
                Task::none()
            }
            SettingsMessage::OutputTemplateChanged(val) => {
                state.settings.output_template = val.to_string();
                Task::none()
            }
            SettingsMessage::Save => {
                state.config.download_directory = PathBuf::from(&state.settings.download_dir_input);
                state.config.default_format = state.settings.selected_format.clone();
                state.config.max_concurrent_downloads = state.settings.concurrent;

                state.config.audio_only = state.settings.audio_only;
                state.config.extract_audio_format = state.settings.extract_audio_format.clone();
                state.config.embed_thumbnail = state.settings.embed_thumbnail;
                state.config.write_metadata = state.settings.write_metadata;
                state.config.download_subtitles = state.settings.download_subtitles;
                state.config.sponsorblock_remove = state.settings.sponsorblock_remove;
                state.config.cookies_from_browser = state.settings.cookies_from_browser.clone();

                state.config.video_codec = state.settings.video_codec.clone();
                state.config.prefer_hdr = state.settings.prefer_hdr;
                state.config.download_playlist = state.settings.download_playlist;
                state.config.rate_limit = state.settings.rate_limit.clone();
                state.config.output_template = state.settings.output_template.clone();

                let config = state.config.clone();
                Task::perform(
                    async move { persistence::settings::save(&config).is_ok() },
                    |_| Message::SettingsSaved,
                )
            }
        },

        // ── Internal async results ──
        Message::MetadataFetched(id, result) => {
            if let Some(entry) = state.home.queue.iter_mut().find(|e| e.id == id) {
                match result {
                    Ok(metadata) => {
                        entry.title = Some(metadata.title);
                        entry.selected_quality = metadata.selected_quality;
                        entry.is_playlist = metadata.is_playlist;
                        entry.playlist_items = metadata.entries;
                        entry.status = DownloadStatus::Ready;
                    }
                    Err(e) => {
                        entry.status = DownloadStatus::Failed(e);
                    }
                }
            }
            Task::none()
        }
        Message::DownloadEvent(id, event) => {
            if let Some(entry) = state.home.queue.iter_mut().find(|e| e.id == id) {
                match event {
                    download::DownloadEvent::Progress { index, update } => {
                        entry.progress = Some(update.clone());
                        if let Some(item) =
                            index.and_then(|i| entry.playlist_items.get_mut(i.saturating_sub(1)))
                        {
                            item.status = crate::state::PlaylistItemStatus::Downloading;
                            item.progress = Some(update);
                        }
                    }
                    download::DownloadEvent::ItemStarted(index) => {
                        if let Some(item) = entry.playlist_items.get_mut(index.saturating_sub(1)) {
                            item.status = crate::state::PlaylistItemStatus::Downloading;
                        }
                    }
                    download::DownloadEvent::ItemCompleted { index, path } => {
                        entry.file_path = Some(path);
                        if let Some(item) =
                            index.and_then(|i| entry.playlist_items.get_mut(i.saturating_sub(1)))
                        {
                            item.status = crate::state::PlaylistItemStatus::Completed;
                            item.progress = None;
                        }
                    }
                    download::DownloadEvent::ItemFailed { index, error } => {
                        if let Some(item) = entry.playlist_items.get_mut(index.saturating_sub(1)) {
                            item.status = crate::state::PlaylistItemStatus::Failed(error);
                            item.progress = None;
                        }
                    }
                    download::DownloadEvent::Finished { successful, error } => {
                        if successful == 0 {
                            entry.status = DownloadStatus::Failed(
                                error.unwrap_or_else(|| "No videos downloaded".into()),
                            );
                            return Task::none();
                        }
                        entry.status = DownloadStatus::Completed;
                        for item in &mut entry.playlist_items {
                            if matches!(
                                item.status,
                                crate::state::PlaylistItemStatus::Pending
                                    | crate::state::PlaylistItemStatus::Downloading
                            ) {
                                item.status = crate::state::PlaylistItemStatus::Failed(
                                    "Download failed".into(),
                                );
                            }
                        }
                        let path = entry
                            .file_path
                            .clone()
                            .unwrap_or_else(|| state.config.download_directory.clone());
                        let title = entry.title.clone().unwrap_or_else(|| "Unknown".into());
                        let history_entry = HistoryEntry {
                            id: entry.id,
                            url: entry.url.clone(),
                            title,
                            format: entry.format.clone(),
                            file_path: path,
                            file_size: 0,
                            downloaded_at: chrono::Utc::now(),
                            success: true,
                        };
                        state.history.entries.push(history_entry);
                        let history = state.history.entries.clone();
                        return Task::perform(
                            async move { persistence::history::save(&history).is_ok() },
                            |_| Message::SettingsSaved,
                        );
                    }
                }
            }
            Task::none()
        }
        Message::SettingsSaved => {
            state.settings.save_status = Some("✓ Settings saved".to_string());
            Task::none()
        }
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let sidebar: Element<'_, Message> = {
        let nav: Element<'_, SidebarMessage> = sidebar::view(state.current_screen).into();
        nav.map(Message::Sidebar)
    };

    let content: Element<'_, Message> = match state.current_screen {
        Screen::Home => {
            let v: Element<'_, HomeMessage> = home::view(&state.home).into();
            v.map(Message::Home)
        }
        Screen::History => {
            let v: Element<'_, HistoryMessage> = history::view(&state.history).into();
            v.map(Message::History)
        }
        Screen::Settings => {
            let v: Element<'_, SettingsMessage> = settings::view(&state.settings).into();
            v.map(Message::Settings)
        }
    };

    let main_content = container(
        row![
            sidebar,
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(style::container_content),
        ]
        .spacing(0)
        .height(Length::Fill)
        .width(Length::Fill),
    )
    .style(style::container_root)
    .width(Length::Fill)
    .height(Length::Fill);

    main_content.into()
}
