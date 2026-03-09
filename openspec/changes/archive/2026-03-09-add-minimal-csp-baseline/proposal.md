## Why

The desktop app currently sets Tauri WebView CSP to `null`, which removes a global browser-level safeguard against future script or resource injection mistakes. The current codebase already sanitizes the known `v-html` path, but relying on local sanitization alone increases long-term maintenance risk as the UI evolves.

## What Changes

- Add a minimal Content Security Policy baseline for the Tauri app instead of leaving CSP disabled.
- Define the minimum allowed sources needed for the current app architecture, including the local app bundle, dev server, and the NicoNico embedded player flow.
- Verify that the baseline works in both development and packaged builds without breaking the current playback UI.
- Document that `shell:allow-open` remains intentionally unchanged in this change and is only tracked as a future hardening consideration.

## Capabilities

### New Capabilities
- `desktop-webview-security`: Defines the minimum CSP baseline required for the desktop WebView and the compatibility expectations for development and production builds.

### Modified Capabilities
- `video-playback`: Clarify that the embedded NicoNico playback flow must remain compatible with the desktop WebView security baseline.

## Impact

- Affected config: `vocaloid-search-desktop/src-tauri/tauri.conf.json`
- Affected frontend surface: embedded NicoNico player loading and any remote resource loading used by the desktop UI
- Affected verification: development startup, production build, and manual playback validation
- No intended changes to shell capabilities or external-link opening behavior in this change
