# OmaDLP

A native GUI for **yt-dlp** — download videos, audio, and playlists from YouTube and hundreds of other sites.

![screenshot](https://via.placeholder.com/800x500?text=OmaDLP)

## Features

- **Download queue** — add URLs, pick quality, start downloads with live progress (speed, ETA, %)
- **Playlist support** — expand/collapse individual videos with per-item progress
- **Format presets** — Best quality, 1080p, 720p, 480p, 360p, or smallest file
- **Concurrent downloads** — download multiple playlist videos in parallel (configurable)
- **Audio extraction** — rip audio to mp3, m4a, wav, ogg, or flac
- **Subtitles** — download and embed subtitles automatically
- **SponsorBlock** — skip sponsor segments during download
- **Metadata & thumbnails** — embed metadata and thumbnails into output files
- **Codec preference** — prefer H.264, VP9, or AV1
- **HDR support** — prefer HDR streams when available
- **Rate limiting** — throttle download speed
- **Browser cookies** — import cookies from Chrome, Firefox, or Edge
- **History** — persistent log of all completed downloads
- **Custom output templates** — control filename structure

## Dependencies

### Required

| Dependency | Install command |
|---|---|---|
| **yt-dlp** | `sudo pacman -S yt-dlp` (Arch) / `winget install yt-dlp.yt-dlp` (Windows — includes ffmpeg) / `brew install yt-dlp` (macOS) / [GitHub releases](https://github.com/yt-dlp/yt-dlp/releases) |
| **ffmpeg** | included with `yt-dlp.yt-dlp` on Windows / `sudo pacman -S ffmpeg` (Arch) / `brew install ffmpeg` (macOS) |

ffmpeg is needed for audio extraction, thumbnail embedding, and subtitle muxing. Without it, downloads are limited to video-only formats.

### Build-time

- **Rust** — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **System packages (Linux)** — `sudo pacman -S gcc pkgconf`

## Installation

### Pre-built binaries

Download the latest release from [GitHub Releases](https://github.com/PITIFULHAWK/omadlp/releases).

- **Linux**: `chmod +x omadlp-linux && ./omadlp-linux`
- **Windows**: run `omadlp-windows.exe`

### Build from source

```sh
git clone https://github.com/PITIFULHAWK/omadlp.git
cd omadlp
cargo build --release
./target/release/omadlp
```

## Usage

1. Launch OmaDLP
2. Paste a URL into the input field
3. Select a quality preset from the dropdown
4. Click **Add** to queue the download
5. Click **Download** to start

For playlists, the app fetches the full list of videos — expand the entry to see individual items. Each video can be downloaded separately or the entire playlist at once.

### Configuration

Open the **Settings** tab from the sidebar to customize:

- Download directory
- Default video/audio format
- Concurrent download count
- Audio-only mode with format selection
- Subtitle, metadata, and thumbnail options
- SponsorBlock filtering
- Codec preference (H.264 / VP9 / AV1)
- Rate limiting
- Browser cookie import
- Output filename template

Settings are saved automatically and persist between launches.

## Configuration file

Settings are stored in the platform config directory:

| Platform | Path |
|---|---|
| Linux | `~/.config/omadlp/settings.json` |
| macOS | `~/Library/Application Support/omadlp/settings.json` |
| Windows | `%APPDATA%\omadlp\settings.json` |

## License

This project is licensed under the MIT License.
