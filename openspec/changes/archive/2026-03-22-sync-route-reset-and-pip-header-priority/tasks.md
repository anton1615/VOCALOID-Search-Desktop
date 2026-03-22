## 1. Route-entry playback reset

- [x] 1.1 [Both] Define the sync-route entry reset flow so entering `/scraper` clears authoritative active playback and invalidates any active Search playback snapshot without clearing browsing contexts.
- [x] 1.2 [Rust/state.rs] Add or adjust shared-state helpers needed to clear playback-session state while preserving `list_contexts`, `search_state`, `history_state`, and `watch_later_state`.
- [x] 1.3 [Rust/commands.rs] Expose a route-entry reset command or equivalent backend entry point that emits `active-playback-cleared` only when a playback session is actually terminated.
- [x] 1.4 [Frontend] Wire sync-route entry in `vocaloid-search-desktop/src/App.vue` so both main window and PiP refresh from the backend authoritative snapshot after the reset.

## 2. View restoration after sync reset

- [x] 2.1 [Frontend] Update Search restoration logic so returning from `/scraper` restores browsing state but does not revive the cleared playback selection.
- [x] 2.2 [Frontend] Update History and Watch Later restore/selection flows so they keep browsing state while dropping any stale selected-item revival after the sync-route reset.
- [x] 2.3 [Both] Verify PiP remains open but renders the shared no-selection state after route-entry reset, and that ordinary list-view navigation semantics remain unchanged.

## 3. PiP compact header priority contract

- [x] 3.1 [Frontend] Update the compact metadata layout contract to encode stats-first inline priority while keeping the existing fixed compact shell and title line-state variants.
- [x] 3.2 [Frontend] Adjust `VideoMetaPanel` compact uploader/stats row layout so the four stats stay inline and the uploader identity truncates first under width pressure.
- [x] 3.3 [Frontend] Preserve full-mode metadata behavior and ensure the compact-only header priority change does not leak into main-window presentation.

## 4. Regression coverage and verification

- [x] 4.1 [Frontend tests] Add or update source-contract/component tests for sync-route playback reset, non-revival of stale selections, and PiP empty-state behavior.
- [x] 4.2 [Frontend tests] Add or update compact-header contract tests so the stats-first no-wrap rule and uploader truncation priority are observable and guarded.
- [x] 4.3 [Both] Run the relevant Vitest suites, `npx vue-tsc --noEmit`, and the shared-player verification/build commands required by the affected playback and PiP surfaces.
