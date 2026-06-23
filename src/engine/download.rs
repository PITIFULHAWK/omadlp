use crate::state::{Config, ProgressUpdate};
use iced::futures::channel::mpsc;
use iced::futures::StreamExt;
use regex::Regex;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};

static PROGRESS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^OMADLP_PROGRESS\|([^|]*)\|([^|]*)\|\s*([\d.]+)%\|\s*([^|]*)\|\s*([^|]*)\|\s*([^|]*)\|\s*(.*)$",
    )
    .unwrap()
});
static ITEM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[download\] Downloading item (\d+) of \d+").unwrap());

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Progress {
        index: Option<usize>,
        update: ProgressUpdate,
    },
    ItemStarted(usize),
    ItemCompleted {
        index: Option<usize>,
        path: PathBuf,
    },
    ItemFailed {
        index: usize,
        error: String,
    },
    Finished {
        successful: usize,
        error: Option<String>,
    },
}

/// Start yt-dlp and return a stream which remains connected to its stdout.
pub fn download_stream(
    url: String,
    format: String,
    output_dir: String,
    config: Config,
    is_playlist: bool,
    playlist_title: Option<String>,
    playlist_items: Vec<(usize, String)>,
) -> mpsc::UnboundedReceiver<DownloadEvent> {
    if is_playlist && config.max_concurrent_downloads > 1 && !playlist_items.is_empty() {
        return concurrent_playlist_stream(
            format,
            output_dir,
            config,
            playlist_title.unwrap_or_else(|| "Playlist".to_string()),
            playlist_items,
        );
    }
    let (tx, rx) = mpsc::unbounded();
    std::thread::spawn(move || {
        let audio_only = config.audio_only || is_audio_format(&format);
        let mut cmd = Command::new("yt-dlp");
        let template = if is_playlist {
            format!("{output_dir}/%(playlist_title|Playlist).180B/%(title).180B.%(ext)s")
        } else {
            format!("{output_dir}/{}", config.output_template)
        };
        cmd.args([
            "--no-update", "--no-simulate", "--no-color", "--newline", "--progress", "--continue", "--no-warnings",
            "--progress-template", "download:OMADLP_PROGRESS|%(playlist_index)s|%(progress.filename)s|%(progress._percent_str)s|%(progress._downloaded_bytes_str)s|%(progress._total_bytes_str)s|%(progress._speed_str)s|%(progress._eta_str)s",
            "--print", "after_move:OMADLP_DONE|%(playlist_index)s|%(filepath)s",
            "-o", &template,
        ]);
        if is_playlist {
            cmd.args(["--yes-playlist", "--ignore-errors", "--no-abort-on-error"]);
        } else {
            cmd.arg("--no-playlist");
        }
        if audio_only {
            cmd.args(["-x", "--audio-format", &config.extract_audio_format]);
        } else {
            cmd.args([
                "-f",
                &effective_format(&format, &config.video_codec, config.prefer_hdr),
            ]);
        }
        if config.embed_thumbnail && !audio_only {
            cmd.arg("--embed-thumbnail");
        }
        if config.write_metadata {
            cmd.arg("--add-metadata");
        }
        if config.download_subtitles && !audio_only {
            cmd.args([
                "--write-subs",
                "--write-auto-subs",
                "--sub-langs",
                "en",
                "--embed-subs",
                "--compat-options",
                "no-keep-subs",
            ]);
        }
        if config.sponsorblock_remove {
            cmd.args(["--sponsorblock-remove", "all"]);
        }
        if !config.cookies_from_browser.trim().is_empty() {
            cmd.args(["--cookies-from-browser", config.cookies_from_browser.trim()]);
        }
        apply_browser_impersonation(&mut cmd);
        if !config.rate_limit.trim().is_empty() {
            cmd.args(["-r", config.rate_limit.trim()]);
        }
        cmd.args([
            "--concurrent-fragments",
            &config.max_concurrent_downloads.max(1).to_string(),
        ]);
        cmd.arg(&url).stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let _ = tx.unbounded_send(DownloadEvent::Finished {
                    successful: 0,
                    error: Some(e.to_string()),
                });
                return;
            }
        };
        let stderr = child.stderr.take();
        let stderr_tx = tx.clone();
        let error_handle = std::thread::spawn(move || {
            stderr
                .map(|s| {
                    std::io::BufReader::new(s)
                        .lines()
                        .map_while(Result::ok)
                        .filter(|line| !send_progress_event(line, &stderr_tx))
                        .filter(|line| line.starts_with("ERROR:"))
                        .map(|l| l.trim_start_matches("ERROR: ").to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        });
        let mut successful = 0;
        if let Some(stdout) = child.stdout.take() {
            for line in std::io::BufReader::new(stdout)
                .lines()
                .map_while(Result::ok)
            {
                if send_progress_event(&line, &tx) {
                    continue;
                }
                if let Some(rest) = line.strip_prefix("OMADLP_DONE|") {
                    let mut fields = rest.splitn(2, '|');
                    let index = fields.next().and_then(|v| v.parse().ok());
                    let path = PathBuf::from(fields.next().unwrap_or(&output_dir));
                    successful += 1;
                    let _ = tx.unbounded_send(DownloadEvent::ItemCompleted { index, path });
                }
            }
        }
        let status = child.wait().ok();
        let errors = error_handle.join().unwrap_or_default();
        let error = if successful == 0 && !status.is_some_and(|s| s.success()) {
            Some(
                errors
                    .last()
                    .cloned()
                    .unwrap_or_else(|| "Download failed".into()),
            )
        } else {
            None
        };
        let _ = tx.unbounded_send(DownloadEvent::Finished { successful, error });
    });
    rx
}

pub fn apply_browser_impersonation(cmd: &mut Command) {
    static CHROME_IMPERSONATION_AVAILABLE: LazyLock<bool> = LazyLock::new(|| {
        Command::new("yt-dlp")
            .arg("--list-impersonate-targets")
            .output()
            .ok()
            .is_some_and(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|line| line.starts_with("Chrome") && !line.contains("unavailable"))
            })
    });
    if *CHROME_IMPERSONATION_AVAILABLE {
        cmd.args(["--impersonate", "chrome"]);
    }
}

fn concurrent_playlist_stream(
    format: String,
    output_dir: String,
    config: Config,
    playlist_title: String,
    items: Vec<(usize, String)>,
) -> mpsc::UnboundedReceiver<DownloadEvent> {
    let (tx, rx) = mpsc::unbounded();
    let total = items.len();
    let queue = Arc::new(Mutex::new(std::collections::VecDeque::from(items)));
    let remaining = Arc::new(AtomicUsize::new(total));
    let successful = Arc::new(AtomicUsize::new(0));
    let workers = usize::min(config.max_concurrent_downloads.max(1) as usize, total);
    let folder = Path::new(&output_dir)
        .join(safe_folder_name(&playlist_title))
        .to_string_lossy()
        .into_owned();

    for _ in 0..workers {
        let queue = Arc::clone(&queue);
        let remaining = Arc::clone(&remaining);
        let successful = Arc::clone(&successful);
        let tx = tx.clone();
        let format = format.clone();
        let folder = folder.clone();
        let mut item_config = config.clone();
        item_config.max_concurrent_downloads = 1;

        std::thread::spawn(move || {
            while let Some((index, url)) = queue.lock().ok().and_then(|mut queue| queue.pop_front())
            {
                item_config.output_template = format!("{index:03} - %(title).180B.%(ext)s");
                let mut stream = download_stream(
                    url,
                    format.clone(),
                    folder.clone(),
                    item_config.clone(),
                    false,
                    None,
                    Vec::new(),
                );
                iced::futures::executor::block_on(async {
                    while let Some(event) = stream.next().await {
                        match event {
                            DownloadEvent::Progress { update, .. } => {
                                let _ = tx.unbounded_send(DownloadEvent::Progress {
                                    index: Some(index),
                                    update,
                                });
                            }
                            DownloadEvent::ItemCompleted { path, .. } => {
                                let _ = tx.unbounded_send(DownloadEvent::ItemCompleted {
                                    index: Some(index),
                                    path,
                                });
                            }
                            DownloadEvent::Finished {
                                successful: count,
                                error,
                            } => {
                                if count > 0 {
                                    successful.fetch_add(1, Ordering::Relaxed);
                                } else {
                                    let _ = tx.unbounded_send(DownloadEvent::ItemFailed {
                                        index,
                                        error: error
                                            .unwrap_or_else(|| "Download failed".to_string()),
                                    });
                                }
                            }
                            DownloadEvent::ItemStarted(_) | DownloadEvent::ItemFailed { .. } => {}
                        }
                    }
                });
                if remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
                    let count = successful.load(Ordering::Relaxed);
                    let _ = tx.unbounded_send(DownloadEvent::Finished {
                        successful: count,
                        error: (count == 0).then(|| "All playlist videos failed".to_string()),
                    });
                }
            }
        });
    }
    rx
}

fn safe_folder_name(title: &str) -> String {
    let name = title
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | '\0') {
                '_'
            } else {
                c
            }
        })
        .collect::<String>();
    let name = name.trim().trim_matches('.');
    if name.is_empty() {
        "Playlist".to_string()
    } else {
        name.chars().take(180).collect()
    }
}

fn send_progress_event(line: &str, tx: &mpsc::UnboundedSender<DownloadEvent>) -> bool {
    if let Some(c) = PROGRESS_RE.captures(line) {
        let filename = c[2].trim().to_ascii_lowercase();
        if [
            ".vtt", ".srt", ".ass", ".lrc", ".json", ".jpg", ".jpeg", ".png", ".webp",
        ]
        .iter()
        .any(|extension| filename.ends_with(extension))
        {
            return true;
        }
        let total = c[5].trim().to_string();
        let downloaded = match c[4].trim() {
            "NA" => total.clone(),
            value => value.to_string(),
        };
        let _ = tx.unbounded_send(DownloadEvent::Progress {
            index: c[1].trim().parse().ok(),
            update: ProgressUpdate {
                percentage: c[3].parse().unwrap_or(0.0),
                downloaded,
                total,
                speed: c[6].trim().to_string(),
                eta: match c[7].trim() {
                    "NA" => "00:00".to_string(),
                    value => value.to_string(),
                },
            },
        });
        true
    } else if let Some(c) = ITEM_RE.captures(line) {
        if let Ok(index) = c[1].parse() {
            let _ = tx.unbounded_send(DownloadEvent::ItemStarted(index));
        }
        true
    } else {
        false
    }
}

pub fn effective_format(format: &str, codec: &str, prefer_hdr: bool) -> String {
    let original_format = format;
    let codec_filter = match codec {
        "h264" => Some("[vcodec~='^(avc|h264)']"),
        "vp9" => Some("[vcodec~='^(vp0?9).*']"),
        "av1" => Some("[vcodec~='^(av0?1).*']"),
        _ => None,
    };
    let hdr_filter = prefer_hdr.then_some("[dynamic_range~='HDR']");
    let preferred_candidates = original_format
        .split('/')
        .filter_map(|segment| {
            if segment.starts_with("bestvideo") {
                let mut replacement = "bestvideo".to_string();
                if let Some(f) = codec_filter {
                    replacement.push_str(f);
                }
                if let Some(f) = hdr_filter {
                    replacement.push_str(f);
                }
                Some(segment.replacen("bestvideo", &replacement, 1))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    let has_preferences = codec_filter.is_some() || hdr_filter.is_some();
    let normal_format = if !has_preferences || preferred_candidates.is_empty() {
        original_format.to_string()
    } else {
        format!("{preferred_candidates}/{original_format}")
    };

    // Newer yt-dlp versions expose YouTube super-resolution streams both with
    // an `-sr` format ID and an `AI-upscaled` format note. Try both markers for
    // every video quality alternative, preserving height/codec/HDR constraints.
    let enhanced = original_format
        .split('/')
        .flat_map(|segment| {
            if segment.starts_with("bestvideo") {
                let (video, rest) = segment
                    .split_once('+')
                    .map_or((segment, ""), |(video, rest)| (video, rest));
                let suffix = if rest.is_empty() {
                    String::new()
                } else {
                    format!("+{rest}")
                };
                vec![
                    format!("{video}[format_note*=AI-upscaled]{suffix}"),
                    format!("{video}[format_id$=-sr]{suffix}"),
                ]
            } else {
                Vec::new()
            }
        })
        .collect::<Vec<_>>();

    if enhanced.is_empty() {
        normal_format
    } else {
        format!("{}/{normal_format}", enhanced.join("/"))
    }
}

fn is_audio_format(format: &str) -> bool {
    !format.contains("video") && (format.contains("audio") || format == "worstaudio")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_machine_progress() {
        let c = PROGRESS_RE
            .captures("OMADLP_PROGRESS|2|video.f399.mp4|73.2%|3.36MiB|4.59MiB|915KiB/s|00:05")
            .unwrap();
        assert_eq!(&c[1], "2");
        assert_eq!(&c[3], "73.2");
    }
    #[test]
    fn automatic_format_prefers_super_resolution() {
        let format = effective_format("bestvideo[height<=720]+bestaudio/best", "default", false);
        assert!(format.contains("bestvideo[height<=720][format_note*=AI-upscaled]+bestaudio"));
        assert!(format.contains("bestvideo[height<=720][format_id$=-sr]+bestaudio"));
    }

    #[test]
    fn super_resolution_takes_priority_over_codec_fallback() {
        let format = effective_format("bestvideo+bestaudio/best", "h264", true);
        let enhanced_position = format.find("format_note*=AI-upscaled").unwrap();
        let codec_position = format.find("vcodec~=").unwrap();
        assert!(enhanced_position < codec_position);
        assert!(format.ends_with("bestvideo+bestaudio/best"));
        assert!(!format.contains("dynamic_range~='HDR']+bestaudio/best/bestvideo"));
    }

    #[test]
    fn hdr_preference_cannot_short_circuit_to_combined_360p() {
        let format = effective_format("bestvideo+bestaudio/best", "default", true);
        assert!(
            format.contains("bestvideo[dynamic_range~='HDR']+bestaudio/bestvideo+bestaudio/best")
        );
        assert!(!format.contains("bestvideo[dynamic_range~='HDR']+bestaudio/best/bestvideo"));
    }

    #[test]
    fn recognizes_audio_quality_preset() {
        assert!(is_audio_format("bestaudio/best"));
        assert!(!is_audio_format("bestvideo+bestaudio/best"));
    }

    #[test]
    fn sanitizes_playlist_folder_names() {
        assert_eq!(safe_folder_name("A/B\\C"), "A_B_C");
        assert_eq!(safe_folder_name("..."), "Playlist");
    }
}
