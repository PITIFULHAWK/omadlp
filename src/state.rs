use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Pending,
    FetchingFormats,
    Ready,
    Downloading,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub percentage: f64,
    pub downloaded: String,
    pub total: String,
    pub speed: String,
    pub eta: String,
}

#[derive(Debug, Clone)]
pub enum PlaylistItemStatus {
    Pending,
    Downloading,
    Completed,
    Unavailable(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct PlaylistItem {
    pub index: usize,
    pub title: String,
    pub url: Option<String>,
    pub status: PlaylistItemStatus,
    pub progress: Option<ProgressUpdate>,
}

#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub selected_quality: Option<String>,
    pub status: DownloadStatus,
    pub format: String,
    pub progress: Option<ProgressUpdate>,
    pub file_path: Option<PathBuf>,
    pub is_playlist: bool,
    pub playlist_items: Vec<PlaylistItem>,
    pub playlist_expanded: bool,
}

impl QueueEntry {
    pub fn new(url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            title: None,
            selected_quality: None,
            status: DownloadStatus::Pending,
            format: "bestvideo+bestaudio/best".to_string(),
            progress: None,
            file_path: None,
            is_playlist: false,
            playlist_items: Vec::new(),
            playlist_expanded: true,
        }
    }
}

// ── History ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub format: String,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub downloaded_at: DateTime<Utc>,
    pub success: bool,
}

// ── Settings ──
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub download_directory: PathBuf,
    pub default_format: String,
    pub max_concurrent_downloads: u32,

    // yt-dlp options
    pub audio_only: bool,
    pub extract_audio_format: String,
    pub embed_thumbnail: bool,
    pub write_metadata: bool,
    pub download_subtitles: bool,
    pub sponsorblock_remove: bool,
    pub cookies_from_browser: String,

    // Advanced yt-dlp options
    pub video_codec: String,
    pub prefer_hdr: bool,
    pub download_playlist: bool,
    pub rate_limit: String,
    pub output_template: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_directory: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            default_format: "bestvideo+bestaudio/best".to_string(),
            max_concurrent_downloads: 3,

            audio_only: false,
            extract_audio_format: "mp3".to_string(),
            embed_thumbnail: true,
            write_metadata: true,
            download_subtitles: true,
            sponsorblock_remove: false,
            cookies_from_browser: String::new(),

            video_codec: "default".to_string(),
            prefer_hdr: false,
            download_playlist: true,
            rate_limit: String::new(),
            output_template: "%(title)s.%(ext)s".to_string(),
        }
    }
}

// ── Screen navigation ──
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    History,
    Settings,
}
