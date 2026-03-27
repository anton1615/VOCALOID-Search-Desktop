## Why

When the same video appears in multiple lists, selecting it from a different list currently produces a mixed experience: the playback context may be rebound to the new list in Rust state, but the player can appear to reuse the prior media session because the iframe lifecycle still keys heavily on `video.id`. This makes it unclear whether playback is truly locked to the newly clicked list, especially for next/previous navigation, PiP ownership reentry, and cross-window playback synchronization.

## What Changes

- Define playback selection identity so that the same `video.id` selected from a different list context is always treated as a new playback session.
- Require the player lifecycle to fully transition when the active playback identity changes across lists, even if the selected `video.id` stays the same.
- Ensure active playback, next/previous navigation, metadata refresh, and PiP/main-window reentry all remain bound to the list context of the most recent explicit selection.
- Apply the new session-boundary behavior consistently for explicit cross-list selection among `Search`, `History`, and `WatchLater` contexts.
- Add acceptance coverage for cross-list same-video selection, including dual-window synchronization expectations.

## Capabilities

### New Capabilities
- `playback-session-boundary`: Defines playback identity, session rebinding rules, and player transition requirements when the same video is selected from different lists.

### Modified Capabilities

None.

## Impact

- Affected Rust state and command flow in `vocaloid-search-desktop/src-tauri/src/state.rs` and `vocaloid-search-desktop/src-tauri/src/commands.rs`
- Affected frontend player/session handling in `vocaloid-search-desktop/src/composables/usePlayerCore.ts`, `vocaloid-search-desktop/src/components/UnifiedPlayer.vue`, `vocaloid-search-desktop/src/App.vue`, and `vocaloid-search-desktop/src/PipApp.vue`
- Affected playlist views in `vocaloid-search-desktop/src/views/SearchView.vue`, `vocaloid-search-desktop/src/views/HistoryView.vue`, and `vocaloid-search-desktop/src/views/WatchLaterView.vue`
- Requires verification of main window and PiP synchronization behavior under Tauri playback conditions
