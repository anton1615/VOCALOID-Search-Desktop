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
- **Exclude Watched**: Filter out videos you've already seen from search results
- **Window State Persistence**: Remember window position, size, and maximize state across sessions
- **Local Database**: Scrape and store video data locally for fast offline search
- **Custom Formula Sorting & Filtering**: Weight videos by views, likes, mylists, and comments using your own formula
- **Auto-Skip**: Automatically skip video endings (useful for bypassing credits or sponsor segments)
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

### Portable Mode

To use portable mode (store data in the application folder), create a `data/` folder in the same directory as the executable:
```
<vocaloid-search-desktop.exe location>/data/
```

When portable mode is active, all data (database, config, thumbnails) is stored in this folder. You can copy the entire folder to another computer and the application will use the same data.

**Note**: Switching between portable and standard mode does NOT automatically migrate data.

---

## User Guide

### Usage Flow

1. **Startup Check**: When the app starts, it automatically checks if the database is empty or outdated. The Niconico Snapshot API updates daily around 5-6 AM JST. If your database is stale, you'll be prompted to sync.

2. **Sync Database**: Go to **Data Sync** page to configure and run the scraper:
   - **Query**: Search keyword (default: `VOCALOID`)
   - **Max Age**: Only fetch videos from the last N days (default: 365, set to empty for unlimited)
   - **Targets**: Search in `tags`, `tagsExact`, `title`, `description`, or combinations like `tags,title`
   - **Category**: Filter by category (default: `MUSIC`)
   
   Click "Start Sync" to download video data. Syncing once per day is recommended.

3. **Search & Browse**: Use the search bar to find videos by keywords. Add tag filters to narrow results. Adjust sorting formula to prioritize views, likes, mylists, or comments. Enable "Exclude Watched" to hide videos you've already seen.

4. **Watch Videos**: Click a video to play it in the embedded player. Videos auto-play continuously from the list. Enable Auto-Skip in playback settings to automatically skip video endings.

5. **PiP Mode**: Click the pop-out button to open a Picture-in-Picture window. The PiP window is a simplified player view without navigation, and stays always on top.

### Scraper Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `query` | `VOCALOID` | Search keyword for Niconico Snapshot API |
| `max_age_days` | `365` | Only fetch videos newer than N days. Leave empty for unlimited |
| `targets` | `tags` | Search targets: `tags`, `tagsExact`, `title`, `description`, or combinations |
| `category_filter` | `MUSIC` | Niconico category filter (MUSIC, GAME, ANIME, ENTERTAINMENT, DANCE, OTHER) |

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

# OR build with debug flag (faster build, larger binary)
npm run tauri build -- --debug
```

> **Important**: Tauri uses a custom protocol (`tauri://localhost`) instead of HTTP localhost for the embed player. This is required because Niconico's embed player rejects requests from `localhost` domains.
>
> **Development Note**: Since Niconico rejects localhost connections, `tauri dev` cannot be used for testing the embedded player. You must rebuild the entire application to test frontend changes. Using `--debug` significantly speeds up the build by skipping Rust optimizations, making the development iteration faster. Use release build (without `--debug`) for final distribution.

The built executable will be at:
```
vocaloid-search-desktop/src-tauri/target/release/vocaloid-search-desktop.exe
```

---

## Known Issues & Future Plans

> ⚠️ **Prototype Status**: This is an early prototype. Features may be unstable and untested.

### Known Issues

1. **History Page**: Displays watch records but clicking videos does not work
2. **PiP ↔ Main Window Sync**: State synchronization is incomplete ✅ Fixed in v1.0.1
   - ~~Playback in PiP is not recorded to watch history~~ PiP playback is now correctly recorded to watch history
   - Playback state is reset when opening/closing PiP
3. **PiP Window**: Occasionally cannot be closed (unknown cause)
4. **PiP Playlist Loading**: When PiP reaches the end of loaded results, it waits for the main window to load more (PiP cannot trigger load more itself)
5. **Region-Locked Videos**: Interrupt auto-play; cannot be marked as watched since they fail to play
6. **Tab Switching During Active Events**: Switching tabs while an event is in progress (e.g., scraper sync, video playback in main or PiP window) may cause unexpected issues such as UI state inconsistency and PiP window sync failures
7. **Other Issues**: Many edge cases remain untested

### Future Plans

1. **Built-in Watch Later**: Implement "Watch Later" and custom playlists similar to Niconico's あとで見る feature
2. **Custom Playlists**: User-defined video collections
3. **Improved State Sync**: Better synchronization between main window and PiP
4. **Keyboard Shortcuts**: Add global keyboard shortcuts for playback controls (play/pause, next/previous, volume)
5. **Open in Browser**: Interact with embedded player to open links in default browser, or add a button in PiP window to quickly open the current video's Niconico page
6. **Global Volume Control**: Add application-wide volume control independent of the embedded player
7. **Database Size Display**: Show current database size on the sync page
8. **Clickable Tags**: Allow clicking tags to add them to the search box as search conditions
9. **Title & Author Links**: Make video title and author name above the embedded player clickable to open their respective Niconico pages in default browser
10. **Linux Support**: Native Linux builds using Tauri's cross-platform capabilities

---

## License

[MIT License](./LICENSE)
