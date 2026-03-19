<p align="center">
  <img src="./icon.png" alt="VOCALOID Search Desktop icon" width="128" height="128">
</p>

<h1 align="center">VOCALOID Search Desktop</h1>

<p align="center">
  🎵 A local-first desktop app for searching and playing Niconico VOCALOID videos with a modern Spotify-like interface.
</p>

<p align="center">
  <strong>Windows native • Tauri + Rust backend • Vue 3 frontend • SQLite FTS5 search</strong>
</p>

<p align="center">
  <a href="https://www.microsoft.com/windows"><img alt="Platform" src="https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4"></a>
  <a href="https://tauri.app/"><img alt="Tauri" src="https://img.shields.io/badge/Tauri-2.x-24C8DB"></a>
  <a href="https://vuejs.org/"><img alt="Vue" src="https://img.shields.io/badge/Vue-3-42B883"></a>
  <a href="https://www.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/Rust-backend-000000"></a>
  <a href="https://www.sqlite.org/fts5.html"><img alt="Database" src="https://img.shields.io/badge/SQLite-FTS5-003B57"></a>
</p>

<p align="center">
  <a href="./README.ja.md">日本語</a> · <a href="./README.zh.md">中文</a>
</p>

> **Inspired by [ニコニコ超検索](https://gokulin.info/search/)** – a popular Niconico video search service.
>
> This project is a desktop port of the [web-based VOCALOID Search](https://github.com/anton1615/VOCALOID-Search), rebuilt as a local-first application using Tauri.

> [!TIP]
> **Best fit for:** people who want a native Windows app with local search, integrated PiP playback, Watch Later, and a Rust-authoritative playback model instead of a self-hosted web stack.

## 📑 Table of Contents

- [⚡ Quick Start](#-quick-start)
- [✨ Why this desktop edition exists](#-why-this-desktop-edition-exists)
- [📸 Screenshots](#-screenshots)
- [🚀 Core Features](#-core-features)
- [🧭 Architecture Highlights](#-architecture-highlights)
- [🧩 Web vs Desktop Edition](#-web-vs-desktop-edition)
- [🛠️ Technical Specifications](#️-technical-specifications)
- [💻 System Requirements](#-system-requirements)
- [🗂️ Data Storage](#️-data-storage)
- [📚 User Guide](#-user-guide)
- [❓ FAQ / Troubleshooting](#-faq--troubleshooting)
- [🏗️ Build from Source](#️-build-from-source)
- [⚠️ Known Limitations](#️-known-limitations)
- [🗺️ Future Plans](#️-future-plans)
- [📄 License](#-license)

---

## ⚡ Quick Start

If you just want to try the app locally:

1. Clone the repository
2. Go into `vocaloid-search-desktop/`
3. Install dependencies with `npm install`
4. Run `npm run tauri dev` for general UI work
5. Use `npm run tauri build -- --debug` when you need to validate embedded-player behavior

> [!IMPORTANT]
> Niconico rejects localhost-origin embedded playback, so `tauri dev` is **not enough** for final playback validation. Use a debug or release Tauri build when checking real embedded-player behavior.

> [!NOTE]
> The desktop app now includes Watch Later, authoritative Rust-managed playback state, PiP/main shared player behavior, and preflight sync storage checks out of the box.

---

## ✨ Why this desktop edition exists

VOCALOID Search Desktop is designed for people who want a **native local app** instead of a self-hosted web service:

- 🎧 **Spotify-like playback workflow** with an embedded Niconico player
- 🖥️ **Native PiP window** that stays on top and shares the same playback state as the main window
- 🔎 **Fast local search** powered by SQLite FTS5
- 📚 **Watch history + Watch Later** built into the desktop app
- 🧠 **Custom formula sorting and filtering** for ranking videos your way
- 💾 **Local-first storage** so browsing and search stay fast after sync

---

## 📸 Screenshots

![Search Interface](./screenshot1.png)
![PiP Window](./screenshot2.png)

---

## 🚀 Core Features

- 🎨 **Modern Interface**: Spotify-inspired UI with playlist-style layout
- 🌗 **Light / Dark Mode**: Theme switching for different viewing environments
- 🌍 **Multi-language Support**: English, Japanese, and Traditional Chinese
- 🪟 **PiP Window**: Native always-on-top picture-in-picture playback window
- 📜 **Watch History**: Track what you have watched and jump back into playback
- ⏰ **Watch Later**: Save videos for future playback in a dedicated list
- 🙈 **Exclude Watched**: Filter watched videos out of search results
- 💾 **Window State Persistence**: Restore window size, position, and maximize state between sessions
- 🗃️ **Local Database**: Scrape and store video metadata locally for fast search
- 🧮 **Custom Formula Sorting & Filtering**: Weight views, likes, mylists, and comments using your own formula
- ⏭️ **Auto-Skip**: Automatically skip video endings when desired
- ▶️ **Embedded Player**: Continuous playback with the official Niconico embed player
- 🏷️ **Keyword + Tag Search**: Full-text keyword search with tag filtering
- ♾️ **Infinite Scroll**: Dynamic loading instead of fixed pagination
- 🧪 **Shared Player Logic**: Main window and PiP consume the same playback event flow

---

## 🧭 Architecture Highlights

This project has evolved beyond a simple prototype. Several architectural decisions are now central to how it works:

### 1. Rust is the single source of truth
All authoritative playback, browsing, and list state live in the Rust backend. The Vue frontend acts as a display and interaction layer rather than maintaining a competing copy of state.

### 2. ListContext versioning protects against race conditions
Each list (Search, History, Watch Later) has its own context, identity, and version. This helps prevent stale pagination or overlapping search/load-more requests from corrupting visible results.

### 3. Main window and PiP share one player model
The app uses a unified player architecture so both windows respond to the same backend playback events and metadata updates. Fixes to playback behavior generally apply to both windows.

### 4. Search playback uses a frozen watched boundary
When Search playback is active with watched exclusion enabled, the app freezes the watched boundary for that session. This keeps pagination membership stable instead of letting videos disappear mid-session.

### 5. Playback metadata enrichment is centralized in Rust
The player renders first, then richer metadata lands through authoritative backend updates. This avoids view-specific fetch divergence and keeps Search, History, Watch Later, main window, and PiP aligned.

---

## 🧩 Web vs Desktop Edition

| Feature | Web Edition | Desktop Edition |
|---------|-------------|-----------------|
| **Deployment** | Self-hosted server (NixOS/Linux) | Local application (Windows) |
| **Runtime** | 24/7 server operation | On-demand launch |
| **Scraper** | Automated via systemd timer | Manual execution with preflight confirmation |
| **Multi-user** | Yes, user registration supported | Single user, local data |
| **Mobile Support** | PWA/TWA for Android | Not applicable |
| **PiP Mode** | Compact window resize | Native always-on-top window |
| **Data Storage** | Server-side database | Local filesystem |
| **Offline Search** | Requires server connection | Fully local after sync |
| **Platform** | NixOS/Linux only | Windows (Linux planned) |

### When to Use Which?

**Choose Web Edition if:**
- You want 24/7 automated scraping
- You need multi-user support
- You want mobile access via PWA
- You run NixOS/Linux servers

**Choose Desktop Edition if:**
- You prefer a native desktop experience
- You want PiP tightly integrated with the OS
- You do not want to manage a server
- You want search and playlist browsing to stay local and fast

---

## 🛠️ Technical Specifications

| Layer | Technology |
|-------|------------|
| **Frontend** | TypeScript, Vue 3, Vite, Tailwind CSS |
| **Backend** | Rust (Tauri 2.x) |
| **Database** | SQLite with FTS5 full-text search |
| **Data Sources** | [Niconico Snapshot API v2](https://site.nicovideo.jp/search-api-docs/snapshot.html), [GetThumbInfo API](https://site.nicovideo.jp/search-api-docs/thumb-info.html) |

### Implementation Architecture

```text
┌─────────────────────────────────────────┐
│           Vue 3 Frontend                │
│  (Search UI, Player, Settings, PiP)     │
└─────────────────┬───────────────────────┘
                  │ Tauri IPC
┌─────────────────▼───────────────────────┐
│            Rust Backend                  │
│ (State, SQLite, HTTP Client, File I/O)  │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│     Local SQLite Database (FTS5)        │
└─────────────────────────────────────────┘
```

### Key Technical Details

- 🔐 **Custom Protocol**: Uses `tauri://localhost` instead of HTTP localhost because Niconico's embed player rejects localhost origins
- 🔎 **FTS5 Full-text Search**: Fast keyword and tag search with SQLite FTS5
- 🧮 **Formula-based Scoring**: Flexible weighting for views, likes, mylists, and comments
- 🔄 **Versioned List Contexts**: Search / History / Watch Later each maintain stable context identity
- 🪄 **Unified Player Core**: Shared playback state and event handling across main window and PiP
- 🧷 **Staged Metadata Rendering**: Player-first UX with authoritative metadata refresh after backend enrichment

### Reliability Notes

- ✅ Frontend logic is covered by focused Vitest suites
- ✅ Rust state and playback flows include backend tests
- ✅ Recent releases prioritized race-condition fixes, state authority, and dual-window consistency rather than cosmetic-only changes

---

## 💻 System Requirements

- **OS**: Windows 10/11 (x64)
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: ~10MB for the application executable. Database size depends on sync scope (for example, `VOCALOID` keyword, 20 days, music category ≈ 40MB)
- **Network**: Internet connection required for video playback and data sync
- **Runtime**: WebView2 Runtime is usually pre-installed on Windows 10/11

---

## 🗂️ Data Storage

Database and settings are stored at:

```text
Windows: %APPDATA%\com.vocaloid-search.desktop
```

### Portable Mode

To use portable mode (store data in the application folder), create a `data/` folder in the same directory as the executable:

```text
<vocaloid-search-desktop.exe location>/data/
```

When portable mode is active, all data (database, config, thumbnails) is stored in this folder. You can copy the entire folder to another computer and the application will use the same data.

**Note**: Switching between portable and standard mode does **not** automatically migrate data.

---

## 📚 User Guide

### Usage Flow

1. **Startup Check**
   - On startup, the app checks whether the database is empty or stale.
   - Niconico Snapshot API data typically refreshes daily around 5–6 AM JST.

2. **Sync Database**
   - Open the **Data Sync** page to configure scraping.
   - Before a sync runs, the app performs a **preflight confirmation** with estimated matched videos, estimated database size, and available disk space.
   - Sync is blocked when the estimated database size exceeds free disk space.

3. **Search & Browse**
   - Search by keyword, combine tag filters, and adjust formula-based sorting.
   - Enable **Exclude Watched** if you want search results to ignore watched videos.

4. **Watch Videos**
   - Click a video in Search, History, or Watch Later to start playback.
   - The embedded player appears first, and metadata is refined through backend-authoritative updates.

5. **Use PiP Mode**
   - Pop playback into a native PiP window.
   - PiP shares the same authoritative playback flow as the main window.

### Scraper Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `query` | `VOCALOID` | Search keyword for Niconico Snapshot API |
| `max_age_days` | `365` | Only fetch videos newer than N days. Leave empty for unlimited |
| `targets` | `tags` | Search targets: `tags`, `tagsExact`, `title`, `description`, or combinations |
| `category_filter` | `MUSIC` | Niconico category filter (MUSIC, GAME, ANIME, ENTERTAINMENT, DANCE, OTHER) |

---

## ❓ FAQ / Troubleshooting

### Can I validate embedded playback with `npm run tauri dev`?

Not fully. `tauri dev` is fine for general UI work, but Niconico embedded playback rejects localhost-origin behavior. Use `npm run tauri build -- --debug` or a release build when validating real embedded-player behavior.

### Why does the player sometimes show the embed first and metadata later?

That is intentional. The app uses staged metadata rendering: the embedded player appears first, then richer metadata is applied after authoritative Rust-side enrichment completes.

### Why does Search playback stay stable even when I mark videos as watched?

Search playback uses a frozen watched boundary for the active playback session. This prevents result membership from shifting underneath the current session.

### Why does the desktop app need a sync preflight step?

The scraper now estimates matched videos, estimated database size, and available disk space before a sync runs. This helps avoid destructive or misleading sync attempts when storage is insufficient.

---

## 🏗️ Build from Source

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
> **Development Note**: Since Niconico rejects localhost connections, `tauri dev` cannot be used to validate embedded-player behavior. Rebuild the full application when testing playback-related frontend changes. Using `--debug` speeds up iteration by skipping Rust optimizations.

The built executable will be at:

```text
vocaloid-search-desktop/src-tauri/target/release/vocaloid-search-desktop.exe
```

---

## ⚠️ Known Limitations

These are the noteworthy issues that still appear relevant to the current app behavior:

1. **PiP Window**: It may occasionally fail to close cleanly (rare / cause still unclear)
2. **Region-Locked Videos**: They can interrupt auto-play and cannot be marked as watched if playback fails entirely
3. **Active Event Timing**: Switching tabs while a long-running event is in progress (for example sync or playback transitions) can still expose edge cases
4. **Rare PiP Sync Failure**: PiP and main window synchronization is much more stable now, but rare sync failures are still noted as edge cases

---

## 🗺️ Future Plans

The following ideas still appear aligned with the current product direction:

- 🎛️ **Keyboard Shortcuts** for playback controls
- 🌐 **Open in Browser** actions from player surfaces or PiP
- 🔊 **Global Volume Control** independent of the embedded player
- 🏷️ **Clickable Tags** to push tags directly into search criteria
- 🔗 **Title & Author Links** above the embedded player
- 🐧 **Linux Support** using Tauri's cross-platform model
- 📦 **Offline Playback** through local downloads
- 🗂️ **Custom Playlists** beyond Watch Later

---

## 📄 License

[MIT License](./LICENSE)
