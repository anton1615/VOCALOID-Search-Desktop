# VOCALOID Search Desktop

A desktop application for searching VOCALOID videos from Niconico.

## Release Notes

### v1.5.0 (2025-03-12)

#### Playlist State Synchronization Fix

- **Fixed "gap" issue**: Resolved incorrect ordering at the 51st video when loading more results after switching between Search, History, and Watch Later views
- **Independent playback state**: Playing video continues across page switches until you explicitly select a new video from another list
- **Version control**: Added list context versioning to prevent data corruption from concurrent load requests
- **Backward compatible**: Legacy fields preserved for smooth upgrades

#### Technical Improvements

- Refactored Rust state management with `ListContext` and `ActivePlayback` models
- Frontend restore logic now reads from `list_context` for data consistency
- Added version reservation mechanism to invalidate concurrent `load_more` requests

---

## Features

- Search videos with advanced filters (views, mylists, comments, likes, date range)
- Custom formula-based sorting and filtering
- Picture-in-Picture (PiP) mode for continuous playback
- Watch history tracking
- Dark/Light theme support
- Multi-language support (Traditional Chinese, Japanese, English)

## Installation

### Portable Mode (USB Drive)

1. Download the `vocaloid-search-desktop.exe` file
2. Create a folder named `data` in the same directory as the executable
3. Run the executable - all data will be stored in the `data` folder

```
/your-usb-drive/
├── vocaloid-search-desktop.exe
└── data/
    ├── data.db
    └── config.json
```

### Standard Mode

1. Download and run the executable
2. Data will be stored in `%APPDATA%/vocaloid-search-desktop/`

## Requirements

- Windows 10 or later
- WebView2 Runtime (usually pre-installed on Windows 10/11)

If WebView2 is not installed, the application will prompt you to install it automatically.

## Usage

### First Launch

1. The application will detect an empty database and navigate to the Scraper page
2. Configure your search query (default: "VOCALOID")
3. Click "Start Sync" to download video data from Niconico
4. Wait for the sync to complete (may take several hours for large datasets)

### Search

- Enter keywords in the search bar and press Enter
- Use the filter bar to sort by views, mylists, comments, likes, or upload date
- Click the gear icon for advanced filters
- Check "Hide Watched" to exclude videos you've already seen

### Playback

- Click a video thumbnail to play it in the embedded player
- Use the PiP button to open a separate always-on-top player window
- Enable auto-play to automatically play the next video

### Custom Formula

Create a weighted score based on multiple metrics:
- Score = (View Weight × Views) + (Mylist Weight × Mylists) + (Comment Weight × Comments) + (Like Weight × Likes)

## Database Freshness

The application checks database freshness on startup:
- Niconico Snapshot API updates daily around 05:00 JST
- If your database was updated before 06:00 today, you'll be prompted to sync

## Keyboard Shortcuts

- `Enter` - Search
- `Space` - Play/Pause (when player is focused)
- `←` / `→` - Previous/Next video

## Technical Details

- Built with Tauri 2.x (Rust backend + Vue 3 frontend)
- SQLite database with FTS5 full-text search
- Niconico Snapshot API for video data

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build release
npm run tauri build -- --no-bundle
```

## License

MIT
