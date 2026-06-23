use crate::components::format_selector::{self, FormatMessage};
use crate::style;
use crate::theme;
use iced::{
    widget::{
        button, checkbox, column, container, pick_list, row, scrollable, text, text_input, Column,
    },
    Alignment, Element, Length, Padding,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    SelectDirectory,
    OpenDownloadFolder,
    DirectorySelected(Option<PathBuf>),
    FormatSelected(&'static str),
    ConcurrentIncrement,
    ConcurrentDecrement,
    ToggleAudioOnly(bool),
    AudioFormatChanged(&'static str),
    ToggleEmbedThumbnail(bool),
    ToggleWriteMetadata(bool),
    ToggleDownloadSubtitles(bool),
    ToggleSponsorblock(bool),
    CookiesBrowserChanged(String),
    VideoCodecChanged(&'static str),
    TogglePreferHdr(bool),
    ToggleDownloadPlaylist(bool),
    RateLimitChanged(String),
    OutputTemplateChanged(&'static str),
    Save,
}

pub struct SettingsState {
    pub download_dir_input: String,
    pub selected_format: String,
    pub concurrent: u32,

    pub audio_only: bool,
    pub extract_audio_format: String,
    pub embed_thumbnail: bool,
    pub write_metadata: bool,
    pub download_subtitles: bool,
    pub sponsorblock_remove: bool,
    pub cookies_from_browser: String,

    pub video_codec: String,
    pub prefer_hdr: bool,
    pub download_playlist: bool,
    pub rate_limit: String,
    pub output_template: String,

    pub save_status: Option<String>,
}

impl SettingsState {
    pub fn new(config: &crate::state::Config) -> Self {
        Self {
            concurrent: config.max_concurrent_downloads,
            download_dir_input: config.download_directory.display().to_string(),
            selected_format: config.default_format.clone(),

            audio_only: config.audio_only,
            extract_audio_format: config.extract_audio_format.clone(),
            embed_thumbnail: config.embed_thumbnail,
            write_metadata: config.write_metadata,
            download_subtitles: config.download_subtitles,
            sponsorblock_remove: config.sponsorblock_remove,
            cookies_from_browser: config.cookies_from_browser.clone(),

            video_codec: config.video_codec.clone(),
            prefer_hdr: config.prefer_hdr,
            download_playlist: config.download_playlist,
            rate_limit: config.rate_limit.clone(),
            output_template: config.output_template.clone(),

            save_status: None,
        }
    }
}

fn section_title(label: &'static str) -> text::Text<'static, iced::Theme> {
    text(label).size(theme::TEXT_SM).color(theme::DIM_TEXT)
}

fn value_text(value: String) -> text::Text<'static, iced::Theme> {
    text(value).size(theme::TEXT_MD).color(theme::FG)
}

pub fn view(state: &SettingsState) -> Column<'static, SettingsMessage> {
    // ── Download directory ──
    let dir_display = value_text(state.download_dir_input.clone()).width(Length::Fill);

    let browse_btn = button(
        container(text("Browse...").size(theme::TEXT_SM))
            .padding(Padding::new(6.0).left(12.0).right(12.0)),
    )
    .style(style::button_primary)
    .on_press(SettingsMessage::SelectDirectory);

    // ── Default quality format ──
    let format_row: Element<'_, SettingsMessage> = {
        let v: Element<'_, FormatMessage> = format_selector::view(&state.selected_format).into();
        v.map(|msg| match msg {
            FormatMessage::FormatSelected(value) => SettingsMessage::FormatSelected(value),
        })
    };

    // ── Concurrent downloads spinner ──
    let concurrent_row = row![
        button(text("-").size(theme::TEXT_MD))
            .style(style::button_ghost)
            .on_press(SettingsMessage::ConcurrentDecrement)
            .padding(Padding::new(6.0).left(12.0).right(12.0)),
        container(text(state.concurrent.to_string()).size(theme::TEXT_MD))
            .padding(Padding::new(6.0).left(12.0).right(12.0))
            .style(|_t: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(theme::SURFACE)),
                text_color: None,
                border: iced::Border {
                    color: theme::BORDER_SUBTLE,
                    width: 1.0,
                    radius: iced::border::Radius::new(theme::CORNER_RADIUS),
                },
                shadow: iced::Shadow::default(),
                snap: false,
            }),
        button(text("+").size(theme::TEXT_MD))
            .style(style::button_ghost)
            .on_press(SettingsMessage::ConcurrentIncrement)
            .padding(Padding::new(6.0).left(12.0).right(12.0)),
    ]
    .spacing(theme::SPACING)
    .align_y(Alignment::Center);

    // ── Feature toggles ──
    let audio_checkbox = checkbox(state.audio_only)
        .label("Download audio only")
        .on_toggle(SettingsMessage::ToggleAudioOnly)
        .text_size(theme::TEXT_MD);

    let audio_format_row = if state.audio_only {
        let formats = ["mp3", "m4a", "wav", "ogg", "flac"];
        let selected = match state.extract_audio_format.as_str() {
            "m4a" => "m4a",
            "wav" => "wav",
            "ogg" => "ogg",
            "flac" => "flac",
            _ => "mp3",
        };
        let pick = pick_list(formats, Some(selected), SettingsMessage::AudioFormatChanged)
            .width(Length::Fixed(120.0))
            .style(style::pick_list_default);
        Some(
            row![
                text("Audio format")
                    .size(theme::TEXT_SM)
                    .color(theme::DIM_TEXT),
                pick,
            ]
            .spacing(theme::SPACING)
            .align_y(Alignment::Center),
        )
    } else {
        None
    };

    let thumb_checkbox = checkbox(state.embed_thumbnail)
        .label("Embed thumbnail")
        .on_toggle(SettingsMessage::ToggleEmbedThumbnail)
        .text_size(theme::TEXT_MD);

    let meta_checkbox = checkbox(state.write_metadata)
        .label("Write metadata")
        .on_toggle(SettingsMessage::ToggleWriteMetadata)
        .text_size(theme::TEXT_MD);

    let subs_checkbox = checkbox(state.download_subtitles)
        .label("Embed subtitles in video (no separate subtitle file)")
        .on_toggle(SettingsMessage::ToggleDownloadSubtitles)
        .text_size(theme::TEXT_MD);

    let sponsor_checkbox = checkbox(state.sponsorblock_remove)
        .label("Skip sponsor segments")
        .on_toggle(SettingsMessage::ToggleSponsorblock)
        .text_size(theme::TEXT_MD);

    let cookies_input = text_input(
        "Browser for cookies (chrome, firefox, edge, or leave blank)",
        &state.cookies_from_browser,
    )
    .style(style::text_input_default)
    .on_input(SettingsMessage::CookiesBrowserChanged)
    .padding(Padding::new(10.0))
    .width(Length::Fill);

    // ── Advanced yt-dlp options ──
    let open_folder_btn = button(
        container(text("Open folder").size(theme::TEXT_SM))
            .padding(Padding::new(6.0).left(12.0).right(12.0)),
    )
    .style(style::button_ghost)
    .on_press(SettingsMessage::OpenDownloadFolder);

    let dir_row = row![dir_display, browse_btn, open_folder_btn]
        .spacing(theme::SPACING)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let codec_options = [
        "Default",
        "H.264 (best compatibility)",
        "VP9 (best quality)",
        "AV1 (newest / smallest)",
    ];
    let selected_codec = match state.video_codec.as_str() {
        "h264" => "H.264 (best compatibility)",
        "vp9" => "VP9 (best quality)",
        "av1" => "AV1 (newest / smallest)",
        _ => "Default",
    };
    let codec_pick = pick_list(codec_options, Some(selected_codec), |label| {
        let value = match label {
            "H.264 (best compatibility)" => "h264",
            "VP9 (best quality)" => "vp9",
            "AV1 (newest / smallest)" => "av1",
            _ => "default",
        };
        SettingsMessage::VideoCodecChanged(value)
    })
    .width(Length::Fill)
    .style(style::pick_list_default);

    let playlist_checkbox = checkbox(state.download_playlist)
        .label("Allow playlist downloads (auto-detect single video vs playlist)")
        .on_toggle(SettingsMessage::ToggleDownloadPlaylist)
        .text_size(theme::TEXT_MD);

    let rate_input = text_input(
        "e.g. 1M, 500K, or leave blank for unlimited",
        &state.rate_limit,
    )
    .style(style::text_input_default)
    .on_input(SettingsMessage::RateLimitChanged)
    .padding(Padding::new(10.0))
    .width(Length::Fill);

    let template_options = [
        ("Title only", "%(title)s.%(ext)s"),
        ("Title + uploader", "%(uploader)s - %(title)s.%(ext)s"),
        (
            "Playlist index + title",
            "%(playlist_index)s - %(title)s.%(ext)s",
        ),
    ];
    let selected_template = template_options
        .iter()
        .find(|(_, value)| *value == state.output_template)
        .map(|(label, _)| *label)
        .unwrap_or("Title only");
    let template_labels: Vec<&'static str> =
        template_options.iter().map(|(label, _)| *label).collect();
    let template_pick = pick_list(template_labels, Some(selected_template), move |label| {
        let value = template_options
            .iter()
            .find(|(l, _)| *l == label)
            .map(|(_, v)| *v)
            .unwrap_or("%(title)s.%(ext)s");
        SettingsMessage::OutputTemplateChanged(value)
    })
    .width(Length::Fill)
    .style(style::pick_list_default);

    // ── Assemble form ──
    let mut form = column![
        text("⚙ Settings")
            .size(theme::TEXT_LG)
            .color(theme::HEADER_FG),
        section_title("Download directory"),
        dir_row,
        section_title("Default quality"),
        format_row,
        section_title("Max concurrent downloads"),
        concurrent_row,
        container(
            text("Features")
                .size(theme::TEXT_LG)
                .color(theme::HEADER_FG)
        )
        .padding(Padding::new(0.0).top(theme::SPACING_LG))
        .width(Length::Fill),
        audio_checkbox,
    ]
    .spacing(theme::SPACING)
    .width(Length::Fill);

    if let Some(row) = audio_format_row {
        form = form.push(row);
    }

    form = form.push(thumb_checkbox);
    form = form.push(meta_checkbox);
    form = form.push(subs_checkbox);
    form = form.push(sponsor_checkbox);

    form = form.push(section_title("Cookies from browser"));
    form = form.push(cookies_input);

    form = form.push(
        container(
            text("Advanced")
                .size(theme::TEXT_LG)
                .color(theme::HEADER_FG),
        )
        .padding(Padding::new(0.0).top(theme::SPACING_LG))
        .width(Length::Fill),
    );

    form = form.push(section_title("Video codec preference"));
    form = form.push(codec_pick);

    let hdr_checkbox = checkbox(state.prefer_hdr)
        .label("Prefer HDR (download HDR streams when available)")
        .on_toggle(SettingsMessage::TogglePreferHdr)
        .text_size(theme::TEXT_MD);

    form = form.push(hdr_checkbox);

    form = form.push(playlist_checkbox);

    form = form.push(section_title("Rate limit"));
    form = form.push(rate_input);

    form = form.push(section_title("Filename template"));
    form = form.push(template_pick);

    form = form.push(
        button(
            container(text("💾 Save Settings").size(theme::TEXT_MD))
                .padding(Padding::new(8.0).left(16.0).right(16.0)),
        )
        .style(style::button_primary)
        .on_press(SettingsMessage::Save)
        .padding(Padding::new(8.0).top(4.0)),
    );

    if let Some(status) = &state.save_status {
        let color = if status.starts_with("✓") {
            theme::SUCCESS
        } else {
            theme::ERROR
        };
        form = form.push(text(status.clone()).size(theme::TEXT_XS).color(color));
    }

    let settings_card = container(form)
        .style(style::container_card)
        .padding(Padding::new(theme::PADDING_LG))
        .width(Length::Fill);

    let footer = text("Omadlp v0.1.0  •  yt-dlp GUI")
        .size(theme::TEXT_XS)
        .color(theme::DIM_TEXT);

    let content = column![
        settings_card,
        container(footer)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(Padding::new(theme::PADDING)),
    ]
    .spacing(theme::SPACING)
    .width(Length::Fill);

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
