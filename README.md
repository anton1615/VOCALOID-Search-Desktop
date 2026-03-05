# VOCALOID Search Desktop

A local-first desktop application for searching Niconico VOCALOID videos with a modern Spotify-like interface.

> **Inspired by [ニコニコ超検索](https://gokulin.info/search/)** – a popular Niconico video search service.

This is a desktop port of the [web-based VOCALOID Search](https://github.com/anton1615/VOCALOID-Search), rebuilt as a local-first application using Tauri.

> **Note**: This project is entirely **vibe coded** – built through iterative AI-assisted development.

**[日本語](./README.ja.md) | [中文](./README.zh.md)**

---

## Screenshots

![Search Interface](./screenshot1.png)
![PiP Window](./screenshot2.png)

---

## Features

- **Modern Interface**: Spotify-inspired UI with playlist-like layout
- **Light/Dark Mode**: Toggle between themes
- **Multi-language Support**: English, Japanese, Chinese (Traditional)
- **PiP Window**: Pop out a picture-in-picture player that stays on top
- **Watch History**: Track videos you've watched
- **Local Database**: Scrape and store video data locally for fast offline search
- **Custom Formula Sorting & Filtering**: Weight videos by views, likes, mylists, and comments using your own formula
- **Embedded Player**: Continuous playback with the official Niconico embed player
- **Keyword + Tag Search**: Full-text search with tag filtering
- **Infinite Scroll**: Dynamic loading instead of fixed pagination

---

## Web vs Desktop Edition

| Feature | Web Edition | Desktop Edition |
|---------|-------------|-----------------|
| **Deployment** | Self-hosted server (NixOS/Linux) | Local application (Windows) |
| **Runtime** | 24/7 server operation | On-demand launch |
| **Scraper** | Automated via systemd timer | Manual execution |
| **Multi-user** | Yes, user registration supported | Single user, local data |
| **Mobile Support** | PWA/TWA for Android | Not applicable |
| **PiP Mode** | Compact window resize | Native always-on-top window |
| **Data Storage** | Server-side database | Local filesystem |
| **Offline Search** | Requires server connection | Fully local |
| **Platform** | NixOS/Linux only | Windows (Linux planned) |

### When to Use Which?

**Choose Web Edition if:**
- You want 24/7 automated scraping
- You need multi-user support
- You want mobile access via PWA
- You run NixOS/Linux servers

**Choose Desktop Edition if:**
- You prefer a native desktop experience
- You want PiP window tightly integrated with OS
- You don't want to manage a server
- You need fully offline search capability

---

## Technical Specifications

| Layer | Technology |
|-------|------------|
| **Frontend** | TypeScript, Vue 3, Vite, Tailwind CSS |
| **Backend** | Rust (Tauri 2.x) |
| **Database** | SQLite with FTS5 full-text search |
| **Data Sources** | [Niconico Snapshot API v2](https://site.nicovideo.jp/search-api-docs/snapshot.html), [GetThumbInfo API](https://site.nicovideo.jp/search-api-docs/thumb-info.html) |

### Implementation Architecture

```
┌─────────────────────────────────────────┐
│           Vue 3 Frontend                │
│  (Search UI, Player, Settings, PiP)     │
└─────────────────┬───────────────────────┘
                  │ Tauri IPC
┌─────────────────▼───────────────────────┐
│           Rust Backend                   │
│  (SQLite, HTTP Client, File System)     │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│    Local SQLite Database (FTS5)         │
└─────────────────────────────────────────┘
```

### Key Technical Details

- **Custom Protocol**: Uses Tauri's custom protocol (`tauri://localhost`) instead of HTTP localhost to avoid Niconico embed player domain restrictions
- **FTS5 Full-text Search**: SQLite FTS5 enables fast keyword and tag searches with AND/OR/NOT operators
- **Formula-based Scoring**: Custom weighted scoring system for flexible sorting and filtering

## System Requirements

- **OS**: Windows 10/11 (x64)
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: ~500MB for application + database size varies
- **Network**: Internet connection required for video playback and data sync

## Data Storage

Database and settings are stored at:
```
Windows: %APPDATA%\com.vocaloid-search.desktop
```

---

## User Guide

### Usage Flow

1. **Sync Database**: Open the app and go to **Data Sync** page. Click "Start Sync" to download video data from Niconico. The Niconico Snapshot API updates daily around 5-6 AM JST, so syncing once per day is recommended.

2. **Search & Browse**: Use the search bar to find videos by keywords. Add tag filters to narrow results. Adjust sorting formula to prioritize views, likes, mylists, or comments.

3. **Watch Videos**: Click a video to play it in the embedded player. Videos auto-play continuously from the list.

4. **PiP Mode**: Click the pop-out button to open a Picture-in-Picture window. The PiP window is a simplified player view without navigation, and stays always on top.

---

## Build from Source

### Prerequisites

- Node.js 18+
- Rust 1.70+ (with `cargo`)
- Windows 10/11 SDK

### Build Steps

```bash
# Clone the repository
git clone https://github.com/anton1615/VOCALOID-Search-Desktop.git
cd VOCALOID-Search-Desktop/vocaloid-search-desktop

# Install dependencies
npm install
```

#### Development Build

```bash
npm run tauri dev
```

This runs the frontend dev server and Tauri in development mode.

#### Production Build

```bash
# Build frontend and backend together
npm run tauri build

# OR build with debug flag (larger binary but better diagnostics)
npm run tauri build -- --debug
```

> **Important**: Tauri uses a custom protocol (`tauri://localhost`) instead of HTTP localhost for the embed player. This is required because Niconico's embed player rejects requests from `localhost` domains. The production build properly handles this; if you encounter issues, try the `--debug` flag.

The built executable will be at:
```
vocaloid-search-desktop/src-tauri/target/release/vocaloid-search-desktop.exe
```

---

## Known Issues & Future Plans

> ⚠️ **Prototype Status**: This is an early prototype. Features may be unstable and untested.

### Known Issues

1. **History Page**: Displays watch records but clicking videos does not work
2. **PiP ↔ Main Window Sync**: State synchronization is incomplete
   - Playback in PiP is not recorded to watch history
   - Playback state is reset when opening/closing PiP
3. **PiP Window**: Occasionally cannot be closed (unknown cause)
4. **Region-Locked Videos**: Interrupt auto-play; cannot be marked as watched since they fail to play
5. **Other Issues**: Many edge cases remain untested

### Future Plans

1. **Built-in Watch Later**: Implement "Watch Later" and custom playlists similar to Niconico's あとで見る feature
2. **Custom Playlists**: User-defined video collections
3. **Improved State Sync**: Better synchronization between main window and PiP
4. **Linux Support**: Native Linux builds using Tauri's cross-platform capabilities

---

## License

[MIT License](./LICENSE)
