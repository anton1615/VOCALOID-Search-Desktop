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
- **Storage**: ~10MB for application executable. Database size depends on sync scope (e.g., VOCALOID keyword, 20 days, music category ≈ 40MB). WebView2 Runtime usually pre-installed on Windows 10/11.
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
   - ~~Playback state is reset when opening/closing PiP~~ Fixed in v1.1.0
3. **PiP Window**: Occasionally cannot be closed (unknown cause)
4. **PiP Playlist Loading**: When PiP reaches the end of loaded results, it waits for the main window to load more (PiP cannot trigger load more itself) ✅ Fixed in v1.1.0
5. **Region-Locked Videos**: Interrupt auto-play; cannot be marked as watched since they fail to play
6. **Tab Switching During Active Events**: Switching tabs while an event is in progress (e.g., scraper sync, video playback in main or PiP window) may cause unexpected issues
   - ~~UI state inconsistency when switching back to search page~~ Fixed in v1.1.2 - SearchView now restores state from Rust
   - PiP window sync failures (rare, cause unknown)
### Future Plans

1. **Built-in Watch Later**: Implement "Watch Later" and custom playlists similar to Niconico's あとで見る feature
2. **Custom Playlists**: User-defined video collections
3. **Improved State Sync**: Better synchronization between main window and PiP
4. **Keyboard Shortcuts**: Add global keyboard shortcuts for playback controls (play/pause, next/previous, volume)
5. **Open in Browser**: Interact with embedded player to open links in default browser, or add a button in PiP window to quickly open the current video's Niconico page
6. **Global Volume Control**: Add application-wide volume control independent of the embedded player
7. **Database Size Display**: Show current database size on the sync page
8. **Database Size Estimation**: Estimate database size before sync based on search conditions to help users plan storage
9. **Storage Space Check**: Warn users if insufficient storage space before starting sync
10. **Clickable Tags**: Allow clicking tags to add them to the search box as search conditions
11. **Title & Author Links**: Make video title and author name above the embedded player clickable to open their respective Niconico pages in default browser
12. **Linux Support**: Native Linux builds using Tauri's cross-platform capabilities
13. **Offline Playback**: Download videos locally for offline viewing when internet is unavailable
---

## License

[MIT License](./LICENSE)

---

## Release Notes

### v1.5.3 - URL Copy Box in Player Metadata Panel

**Highlights:**
- Added a URL display box below the embedded player showing the full video URL
- Added a copy button (📋) that copies the URL to clipboard with one click
- Button shows "已複製 ✓" feedback for 1.5 seconds after successful copy
- URL box is shared between main window and PiP window via VideoMetaPanel component
- URL text truncates with ellipsis when too long, but full URL is still copied

**Technical Implementation:**
- Added `copied` ref state and `copyToClipboard()` async function in VideoMetaPanel.vue
- Used `navigator.clipboard.writeText()` for clipboard access
- Added `.url-section`, `.url-text`, `.copy-btn` CSS styles matching existing design
- Conditional rendering: only shows when `video.watch_url` exists

### v1.5.2 - Auto-Scroll to Playing Video on Page Switch

**Highlights:**
- Added automatic scroll behavior: When switching back to a playlist view (Search, History, or Watch Later) while a video is playing, the list now automatically scrolls to bring the playing video into view
- Extracted shared scroll function: Created `scrollVideoIntoView()` utility in `playlistViewState.ts` to reduce code duplication across the three views
- Consistent scroll behavior: The auto-scroll uses the same logic as the existing scroll-on-play behavior, ensuring the previous video is visible above and the next-next video is visible below when possible

**Technical Improvements:**
- Added `scrollVideoIntoView(index, listContainer)` function with null-safety checks
- Integrated scroll call in `onMounted` after state restoration using `nextTick()` for proper DOM timing
- Added unit tests for the new scroll function covering edge cases (null container, missing elements, etc.)


### v1.5.1 - Playback Reset on List Mutation

**Highlights:**
- Fixed player reset issue: When changing search filters, sorting, or switching list contexts, the player now properly resets to an empty state instead of attempting to reload a video at the same index from the new results
- Added event-based notification: Backend now emits `active-playback-cleared` event when active playback is invalidated, ensuring frontend stays in sync with Rust state
- Proper cleanup: `getPlaylistState()` now returns empty state when no active playback exists, preventing stale index fallback

**Technical Improvements:**
- Added `AppHandle` parameter to `search()`, `get_history()`, and `get_watch_later()` functions for event emission
- Added listener for `active-playback-cleared` event in `App.vue` that calls `refreshActivePlayback()`
- Fixed `getPlaylistState()` fallback behavior to return empty results instead of legacy index

### v1.5.0 - Playlist State Synchronization Fix

**Highlights:**
- Fixed "gap" issue where video ordering broke at the 51st position when loading more results after switching between Search, History, and Watch Later views
- Playback now persists across view switches until you explicitly select a video from another list
- Added list context versioning to prevent data corruption from concurrent load requests
- Backward compatible with legacy state fields for smooth upgrades

**Technical Improvements:**
- Refactored Rust state management with `ListContext` and `ActivePlayback` models
- Frontend restore logic now reads from `list_context` for data consistency
- Added version reservation mechanism to invalidate concurrent `load_more` requests
- Synced `search_state.results` and `search_results` properly on load_more

### v1.4.1 - Shared Avatar Fallback

**Highlights:**
- Added a shared uploader avatar component so player and list views now render avatar images through one consistent path
- Kept the existing per-user Niconico avatar URL derivation while adding a browser-side fallback to Niconico's default blank avatar
- Unified uploader avatar behavior across the player metadata panel, PiP metadata panel, and search results list

**Bug fixes:**
- Fixed broken uploader images for users who do not have a custom Niconico avatar
- Replaced the old search-result behavior that hid failed avatar images with a proper blank-avatar fallback
- Preserved the local placeholder avatar as a final fallback if both remote avatar images fail to load

### v1.4.0 - Sync Preflight Estimates and Storage Guardrails

**Highlights:**
- Added a preflight sync confirmation flow that always runs before scraper execution
- Shows estimated matched video count, estimated database size, and available disk space before clearing the local database
- Reworked the scraper page with a clearer database-status banner and a more meaningful storage information section

**Bug fixes:**
- Fixed preflight count estimation so `max days = 0` correctly behaves as unlimited instead of near-zero results
- Expanded the scraper category selector to match the backend-supported Snapshot API categories and added a no-filter option
- Blocked sync confirmation when the estimated database size exceeds available disk space to avoid destructive out-of-space runs

### v1.3.1 - WebView Security Baseline and PiP Playback Fixes

**Highlights:**
- Added a minimal Tauri WebView CSP baseline instead of leaving CSP disabled
- Preserved development startup, packaged builds, and embedded NicoNico playback under the new baseline
- Synced PiP playback behavior with the main window so embedded playback remains stable across both windows

**Bug fixes:**
- Fixed a PiP regression where the embedded player iframe could render without a bound source after re-render
- Fixed PiP autoplay failing after load-complete events when iframe window targeting was unavailable
- Documented deferred shell hardening so `shell:allow-open` remains tracked without expanding the scope of this release

### v1.3.0 - Playback Settings Refinement

**Highlights:**
- Replaced the always-visible auto-skip checkbox in the main window with a gear-triggered playback settings panel
- Exposed auto-play and ending auto-skip as two independent toggles
- Kept the settings panel collapsed by default without persisting its open/closed UI state
- Restored auto-play and auto-skip selections after app restart

**Bug fixes:**
- Fixed next-video transitions that failed to resume playback when auto-play was enabled
- Fixed unstable cross-origin iframe message targeting that could prevent play commands from reaching the embed player
- Fixed videos advancing to the next item on natural end even when auto-skip was disabled
- Removed the user-facing skip-threshold input while preserving the internal threshold behavior

### v1.1.2 - Bug Fix

**Bug Fixes:**
- Fixed UI state inconsistency when switching tabs during playback - SearchView now restores state from Rust AppState instead of resetting
- Fixed PiP state not being restored when switching tabs - `pipActive` is now synced with Rust AppState
- Fixed orphaned PiP window when closing main window - PiP window now closes automatically with main window

### v1.1.1 - Bug Fix

**Bug Fixes:**
- Fixed database freshness check logic - now correctly compares last update time against the most recent 6:00 JST threshold (previously used incorrect 24-hour window logic)

### v1.1.0 - Architecture Refactor

**Major Changes:**
- **Rust Unified State Management**: All application state is now managed by Rust AppState, with frontend components acting as pure display layers
- **SearchState in AppState**: Search parameters (query, filters, sort, page) are now stored in `AppState.search_state` for consistent access across windows
- **is_watched Sync**: Watch status is now synchronized between main window and PiP without explicit per-window sync logic
- **loadMore from Any Window**: PiP can now trigger `loadMore()` directly when reaching end of results, eliminating playback interruptions

**Bug Fixes:**
- Fixed database freshness check logic - now correctly compares against the most recent 6:00 JST threshold
- Fixed PiP playback discontinuity at result boundaries - preload kicks in before reaching the end
- Added auto-scroll during continuous playback - keeps current playing video visible with 2 videos below

### v1.0.2 - Bug Fix

**Bug Fixes:**
- Fixed "Exclude Watched" filter returning empty results - corrected SQL table reference from `watched` to `history`

### v1.0.0 - Initial Release

Initial release of VOCALOID Search Desktop with core features:
- Modern Spotify-inspired UI with light/dark mode
- Multi-language support (English, Japanese, Chinese Traditional)
- PiP window for picture-in-picture playback
- Watch history tracking
- Exclude watched filter
- Window state persistence
- Local SQLite database with FTS5 full-text search
- Custom formula sorting & filtering
- Auto-skip functionality
- Embedded Niconico player with continuous playback
- Infinite scroll pagination
