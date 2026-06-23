use crate::state::{PlaylistItem, PlaylistItemStatus};
use serde_json::Value;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct MediaMetadata {
    pub title: String,
    pub selected_quality: Option<String>,
    pub is_playlist: bool,
    pub entries: Vec<PlaylistItem>,
}

fn unavailable_reason(entry: &Value) -> Option<String> {
    if entry.is_null() {
        return Some("Unavailable video".into());
    }
    let availability = entry.get("availability").and_then(Value::as_str);
    let title = entry
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let lower = title.to_lowercase();
    if lower.contains("deleted video") {
        return Some("Deleted video".into());
    }
    if lower.contains("private video") {
        return Some("Private video".into());
    }
    match availability {
        Some("private") => Some("Private video".into()),
        Some("subscriber_only") => Some("Members only".into()),
        Some("premium_only") => Some("Premium only".into()),
        Some("needs_auth") => Some("Sign-in required".into()),
        _ => None,
    }
}

/// Fetch title and playlist entries in one machine-readable yt-dlp call.
pub fn fetch_metadata(
    url: &str,
    allow_playlist: bool,
    format: &str,
    cookies_browser: &str,
) -> Result<MediaMetadata, String> {
    let mut cmd = Command::new("yt-dlp");
    cmd.args([
        "--no-update",
        "--no-warnings",
        "--flat-playlist",
        "--dump-single-json",
    ]);
    if !allow_playlist {
        cmd.arg("--no-playlist");
    }
    if !cookies_browser.trim().is_empty() {
        cmd.args(["--cookies-from-browser", cookies_browser.trim()]);
    }
    crate::engine::download::apply_browser_impersonation(&mut cmd);
    let output = cmd
        .arg(url)
        .output()
        .map_err(|e| format!("Failed to run yt-dlp: {e}"))?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(error
            .lines()
            .find(|l| l.starts_with("ERROR:"))
            .unwrap_or("Could not fetch video information")
            .trim_start_matches("ERROR: ")
            .to_string());
    }
    let json: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Invalid metadata from yt-dlp: {e}"))?;
    let title = json
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Untitled video")
        .to_string();
    let is_playlist = json.get("entries").and_then(Value::as_array).is_some();
    let selected_quality = (!is_playlist)
        .then(|| fetch_selected_quality(url, format, cookies_browser))
        .flatten();
    let entries = json
        .get("entries")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let reason = unavailable_reason(item);
                    PlaylistItem {
                        index: i + 1,
                        title: item
                            .get("title")
                            .and_then(Value::as_str)
                            .filter(|s| !s.is_empty())
                            .unwrap_or("Unavailable video")
                            .to_string(),
                        url: item
                            .get("url")
                            .or_else(|| item.get("webpage_url"))
                            .and_then(Value::as_str)
                            .map(ToString::to_string),
                        status: reason
                            .map(PlaylistItemStatus::Unavailable)
                            .unwrap_or(PlaylistItemStatus::Pending),
                        progress: None,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(MediaMetadata {
        title,
        selected_quality,
        is_playlist,
        entries,
    })
}

fn fetch_selected_quality(url: &str, format: &str, cookies_browser: &str) -> Option<String> {
    let mut cmd = Command::new("yt-dlp");
    cmd.args([
        "--no-update",
        "--no-warnings",
        "--no-playlist",
        "--simulate",
        "-f",
        format,
        "--print",
        "%(resolution)s|%(format_id)s|%(format_note)s",
    ]);
    if !cookies_browser.trim().is_empty() {
        cmd.args(["--cookies-from-browser", cookies_browser.trim()]);
    }
    crate::engine::download::apply_browser_impersonation(&mut cmd);
    let output = cmd.arg(url).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut fields = stdout.lines().next()?.split('|');
    let resolution = fields
        .next()
        .unwrap_or("Unknown quality")
        .split('+')
        .next()
        .unwrap_or("Unknown quality");
    let format_id = fields.next().unwrap_or("unknown");
    let note = fields.next().unwrap_or_default();
    let enhanced =
        note.contains("AI-upscaled") || format_id.split('+').any(|id| id.ends_with("-sr"));
    Some(if enhanced {
        format!("{resolution} • AI-upscaled • format {format_id}")
    } else {
        format!("{resolution} • format {format_id}")
    })
}
