## Context

The current playback model already stores active playback in Rust as list-bound state: `active_playback` keeps a `list_id`, `list_version`, and `current_index`, and commands such as `get_playlist_state`, `play_next`, `play_previous`, and active playback reentry all read from that state. However, the frontend player lifecycle is still partially keyed by `video.id`. When a user selects the same `video.id` from a different list, Rust can rebind the playback identity to the new list while the iframe and local player shell still behave like a reused media session.

This mismatch creates an ambiguous user experience. The player may show a new selection/loading transition while the embedded media continues uninterrupted, making it difficult to trust that playback is truly locked to the newly clicked list. The ambiguity is especially problematic for cross-window synchronization because both the main window and PiP depend on a shared playback identity but do not currently enforce a full player-session boundary for same-video cross-list selection.

## Goals / Non-Goals

**Goals:**
- Make explicit selection from a different list create a new playback session even when `video.id` is unchanged.
- Ensure Rust authoritative playback state, frontend playback identity, and iframe/media lifecycle all transition together.
- Preserve list-bound behavior for next/previous navigation, metadata refresh, and PiP/main-window reentry after cross-list same-video selection.
- Apply the same semantics consistently across `Search`, `History`, and `WatchLater` explicit selections.
- Make the expected behavior testable with deterministic acceptance scenarios.

**Non-Goals:**
- Changing search snapshot semantics, watched-boundary freezing, or non-selection-driven list refresh behavior.
- Redefining single-video metadata sourcing rules for Search vs History vs Watch Later.
- Introducing a generic cross-list deduplication model for displayed videos.

## Playback Identity Audit

The explicit selection path already uses a list-bound playback identity in Rust and forwards the same identity into both window shells.

- `set_playlist_index` reads the current browsing list, resolves its current `list_version`, stores `active_playback = { list_id, list_version, current_index }`, and emits `video-selected` with `playlist_type`, `playlist_version`, `index`, and `video.id`.
- `get_playlist_state` reconstructs the authoritative playback state from `active_playback.list_id` and `active_playback.current_index`, then returns `playlist_type`, `playlist_version`, `index`, `current_video_id`, `results`, and `has_next`.
- `reenter_active_playback_metadata` rebuilds the selected payload from `active_playback.list_id`, `active_playback.list_version`, `active_playback.current_index`, and the current list item at that index.
- `play_next` and `play_previous` currently navigate from `active_playback.list_id` plus `active_playback.current_index`, so list ownership is determined by the latest active playback binding rather than by `video.id`.
- `matches_active_playback_metadata_update` and `apply_playback_metadata_update_if_matches` accept metadata refresh only when `list_id`, `playlist_version`, `index`, and `video.id` all still match the active playback identity.
- `useAuthoritativePlaybackSync` is the frontend entry point that converts backend `getPlaylistState()` responses into the shared window state `{ playlistType, playlistVersion, currentVideoIndex, currentVideo, hasNext, resultsCount }`.
- `App.vue` and `PipApp.vue` both persist that shared playback identity into local refs and pass it into `UnifiedPlayer.vue`, which derives the local display identity as `{ playlistType, playlistVersion, currentIndex, videoId }`.
- The remaining gap is that the embedded player lifecycle still keys its remount behavior too heavily on `video.id`, so the frontend can visually reuse a prior media session even when the authoritative playback identity has already switched lists.

## Decisions

### Decision: Playback session identity remains list-context based, not content-id based
Explicit selection SHALL continue to bind playback to the selected list context using list type, list version, index, and video id. The system will not collapse playback sessions across lists just because the `video.id` matches.

Rationale:
- This matches the intended product behavior: clicking a list item should always lock playback to that list.
- It keeps next/previous navigation deterministic because navigation follows the active list context instead of guessing from shared content ids.
- It aligns with existing Rust state authority rather than replacing it with frontend-only heuristics.

Alternatives considered:
- Treat same `video.id` across lists as the same session. Rejected because it obscures which list owns playback and preserves the current ambiguity.
- Update only browsing/highlight state while keeping the old playback session. Rejected because explicit user selection would no longer reliably rebind playback.

### Decision: Same-video cross-list selection must trigger a full player-session transition
When explicit selection changes playback identity across lists, the player lifecycle SHALL transition as if a different video was selected. The implementation may use a dedicated playback-session key or equivalent mechanism, but it must force the embedded player instance and local shell state to reset together.

Rationale:
- The current issue exists because state identity can change without a corresponding iframe/media boundary.
- A full player-session transition gives users a consistent click-to-play experience regardless of content-id reuse.
- This makes dual-window synchronization easier to reason about because every playback identity change has a single observable transition.

Alternatives considered:
- Keep the iframe alive and only refresh metadata. Rejected because media continuity would still hide whether the new list owns playback.
- Force only a pause/play command sequence without rebuilding the player instance. Rejected because embedded player state could still leak across sessions and remain timing-sensitive.

Observable acceptance signals for this decision:
- A playback identity change across lists must produce a new frontend session boundary, such as a dedicated session key or equivalent remount trigger.
- Local player shell state must return to its pre-ready state before the new session becomes ready.
- Metadata updates must only resolve against the new playback identity after rebinding.
- PiP and main-window reconstruction paths must rebuild from the latest Rust active playback identity rather than continuing from stale local iframe state.

### Decision: Reentry and metadata refresh must follow the latest explicit selection identity
Any command or event path that reconstructs current playback, including PiP ownership return and active playback metadata reentry, SHALL use the latest active playback identity from Rust and SHALL not restore the previous list context merely because the `video.id` matches.

Rationale:
- Reentry flows are user-visible confirmation of which list currently owns playback.
- This ensures the same list-bound semantics across the main window, PiP, and backend event emission.

Alternatives considered:
- Reconstruct from only the current video id. Rejected because it loses list ownership.

## Risks / Trade-offs

- [More frequent player reloads] -> Cross-list same-video clicks will intentionally restart the embedded player session; this is acceptable because the product now defines them as distinct playback sessions.
- [Frontend/backend identity drift during transition] -> Keep Rust as the single source of truth and make frontend session reset derive from the authoritative playback identity rather than local inference.
- [PiP/main-window divergence] -> Verify that both windows consume the same rebinding semantics and that reentry paths rebuild from active playback instead of stale local props.
- [Test coverage gaps under non-Tauri dev mode] -> Cover state/event contracts in unit tests and reserve embedded-player verification for Tauri-capable test flows.

## Migration Plan

1. Add the new playback-session capability spec and implementation tasks.
2. Update frontend player lifecycle so playback identity changes across lists force a new player session even when `video.id` matches.
3. Add regression coverage for cross-list same-video selection, next/previous ownership, and PiP reentry behavior.
4. Validate behavior in a Tauri playback environment before shipping.

Rollback strategy:
- Revert the player-session boundary change and fallback to current same-video reuse behavior if regressions appear, while preserving the spec artifact history for follow-up redesign.

## Open Questions

- Whether the player remount should be driven by an explicit session key prop, a wrapper-level keyed subtree, or another lifecycle boundary mechanism.
- Whether any user-facing loading affordance should be adjusted to better communicate intentional session restart during same-video cross-list selection.
