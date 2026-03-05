# VOCALOID Search Desktop

A desktop application for searching VOCALOID videos on Niconico.

## Features

- Search VOCALOID videos on Niconico
- Picture-in-Picture mode for background playback
- Multi-language support (Traditional Chinese, English, Japanese)
- Dark/Light theme

## Requirements

- Windows 10 1803+ or Windows 11 (WebView2 runtime included)
- Node.js 18+
- Rust (for building from source)

## Build from Source

```bash
cd vocaloid-search-desktop
npm install
npm run tauri build
```

The executable will be at `src-tauri/target/release/vocaloid-search-desktop.exe`.

## Development

```bash
cd vocaloid-search-desktop
npm install
npm run tauri dev
```

## Tech Stack

- Frontend: Vue 3, TypeScript, Tailwind CSS
- Backend: Tauri 2 (Rust)
- Database: SQLite (bundled)

## License

MIT
