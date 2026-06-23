use crate::state::FormatInfo;
use std::process::Command;

fn clean_error(stderr: &str) -> String {
    let lines: Vec<&str> = stderr
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            !lower.starts_with("warning:")
                && !lower.contains("your yt-dlp version")
                && !lower.contains("strongly recommended")
                && !lower.contains("installed yt-dlp")
                && !lower.contains("suppress this warning")
                && !lower.contains("to suppress this warning")
                && !line.trim().is_empty()
        })
        .collect();

    if lines.is_empty() {
        "yt-dlp failed".to_string()
    } else {
        lines.join("\n")
    }
}

/// Fetch available formats for a URL by running `yt-dlp -F`.
pub fn fetch_formats(url: &str) -> Result<Vec<FormatInfo>, String> {
    let output = Command::new("yt-dlp")
        .args(["--no-update", "--no-warnings", "--no-playlist", "-F", url])
        .output()
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let clean = clean_error(&stderr);
        return Err(clean);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut formats = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() || line.starts_with('[') || line.starts_with("---") {
            continue;
        }

        // Skip header lines
        if line.contains("ID") && line.contains("EXT") && line.contains("RESOLUTION") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let ext = parts[1].to_string();

        let resolution = if parts.len() > 2
            && !parts[2].contains("MiB")
            && !parts[2].contains("KiB")
            && !parts[2].contains("GiB")
        {
            Some(parts[2].to_string())
        } else {
            None
        };

        // Determine codecs from the remaining parts
        let note = parts.get(2).map(|s| s.to_string());

        formats.push(FormatInfo {
            format_id: parts[0].to_string(),
            ext,
            resolution,
            filesize: None,
            note,
            vcodec: None,
            acodec: None,
        });
    }

    // Add common format presets
    formats.insert(
        0,
        FormatInfo {
            format_id: "bestvideo+bestaudio/best".to_string(),
            ext: "mp4".to_string(),
            resolution: Some("best".to_string()),
            filesize: None,
            note: Some("Best video+audio (auto)".to_string()),
            vcodec: None,
            acodec: None,
        },
    );

    Ok(formats)
}
