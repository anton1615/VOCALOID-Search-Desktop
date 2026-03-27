## 1. Playback identity and session boundary

- [x] 1.1 [Both] Audit the explicit selection path and document the active playback identity fields consumed by Rust commands, main window sync, and PiP sync.
- [x] 1.2 [Frontend] Introduce a player-session boundary that forces a full player transition when playback identity changes across lists, even if `video.id` stays the same.
- [x] 1.3 [Frontend] Update shared player state handling so local shell reset, readiness state, and embedded player lifecycle remain aligned with the authoritative playback identity.

## 2. Rust authoritative playback behavior

- [x] 2.1 [Rust - state.rs] Verify and, if needed, tighten `active_playback` rebinding rules so explicit same-id cross-list selection always binds ownership to the newly selected list context.
- [x] 2.2 [Rust] Verify command paths that reconstruct or advance playback (`get_playlist_state`, active playback reentry, next/previous) always read and preserve the latest list-bound playback identity.
- [x] 2.3 [Rust] Ensure metadata update filtering continues to reject payloads from superseded list contexts after same-id cross-list rebinding.

## 3. View integration and regression coverage

- [x] 3.1 [Frontend] Update main-window and PiP integration points so cross-list same-video selection, playback-state refresh, and ownership return all reflect the new playback session semantics.
- [x] 3.2 [Both] Add regression tests covering same-id cross-list selection, active playback ownership, next/previous navigation, and metadata update filtering.
- [x] 3.3 [Both] Validate the final behavior in a Tauri-capable playback flow, confirming that cross-list same-video selection no longer appears to reuse the old media session.
