## 1. Define the baseline CSP

- [x] 1.1 Audit the current Tauri + Vue desktop runtime to identify the minimum CSP directives and source origins required for dev mode, packaged assets, and NicoNico embedded playback.
- [x] 1.2 Replace `app.security.csp = null` in `src-tauri/tauri.conf.json` with the agreed minimal baseline CSP for the current architecture.

## 2. Verify compatibility

- [x] 2.1 Verify that the desktop app still starts correctly in the local Tauri development workflow with the new CSP enabled.
- [x] 2.2 Verify that the packaged/frontend-build path still loads correctly with the new CSP enabled.
- [x] 2.3 Manually verify that embedded playback remains functional in both the main window and PiP window under the new CSP baseline.

## 3. Update specifications and deferred notes

- [x] 3.1 Sync the implemented CSP baseline behavior with the `desktop-webview-security` and `video-playback` specs if implementation details require refinement.
- [x] 3.2 Document in the relevant spec or change artifacts that `shell:allow-open` remains unchanged and is deferred as future hardening work.
