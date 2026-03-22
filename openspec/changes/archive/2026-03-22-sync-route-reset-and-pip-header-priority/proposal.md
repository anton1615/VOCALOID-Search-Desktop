## Why

Entering the sync route currently hides the main-window player UI while preserving the shared playback session, which leaves PiP playing, keeps the Search playback snapshot alive, and lets the main window revive the old playback binding when the user returns. The PiP compact header also allows the stats row to break icon and number onto separate lines under width pressure, which weakens readability and makes the uploader/stats priority feel inconsistent.

## What Changes

- Treat entering the sync route as an explicit playback-session reset boundary while keeping normal list-view navigation semantics unchanged.
- Clear active playback and any active Search playback snapshot on sync-route entry, but preserve stored browsing state such as query, sorting, filters, pagination, and loaded results.
- Keep the PiP window open during sync-route entry while transitioning both windows to the same empty playback state from the backend authoritative snapshot.
- Prevent Search, History, and Watch Later from reviving the old selected item when the user returns after the sync-route reset.
- Tighten the PiP compact header contract so the four stats remain on one inline row, the uploader name truncates first, and the fixed compact header shell remains stable.
- Add spec and regression-test coverage for the sync-route reset exception and the compact-header priority contract.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `playback-browsing-boundary`: visible browseable-list navigation still preserves playback, but entering the sync route explicitly resets the shared playback session without clearing browsing contexts.
- `playback-reset-notification`: route-entry playback resets on the sync page emit the same authoritative playback-cleared notification flow consumed by both windows.
- `searchview-state-restoration`: returning from the sync route restores Search browsing state without reviving the old playback selection.
- `pip-window`: PiP compact header layout must preserve a single inline stats row and truncate uploader identity first under horizontal pressure.
- `unified-player-component`: compact metadata presentation contract must encode stats-first width priority and remain testable through stable shared-player markers.

## Impact

- Affected frontend files include `vocaloid-search-desktop/src/App.vue`, `vocaloid-search-desktop/src/PipApp.vue`, `vocaloid-search-desktop/src/components/UnifiedPlayer.vue`, `vocaloid-search-desktop/src/components/VideoMetaPanel.vue`, and playlist restoration/layout helpers and tests.
- Affected backend files include `vocaloid-search-desktop/src-tauri/src/commands.rs`, `vocaloid-search-desktop/src-tauri/src/state.rs`, and any command wiring needed for route-entry playback reset.
- Shared-state behavior changes must preserve Rust as the single source of truth and keep main-window/PiP synchronization aligned.
