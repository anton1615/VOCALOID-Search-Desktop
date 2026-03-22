## Context

The current architecture deliberately separates browsing context from active playback so that moving among Search, History, and Watch Later does not rewrite the playback-bound list. That model works for browseable list views, but the sync route is not a browseable list: it hides the main player, leaves PiP running, keeps the Search playback snapshot alive, and lets the user return into a resurrected playback session even though they intentionally crossed into a data-refresh workflow. The desired behavior is narrower than a full session wipe: entering `/scraper` should clear active playback and Search playback snapshot state, but it must preserve Search/History/Watch Later browsing context, including query, sort, filters, pagination, and loaded results.

Separately, the PiP compact header already has a fixed outer shell and title line-state balancing contract, but the uploader/stats row still behaves like a generic flex row. Under width pressure, each stat token can wrap internally so the emoji/icon sits above the count. The desired presentation is a clearer priority contract: the four stats stay on one inline row, the uploader name yields first through truncation, and the shell height remains fixed.

## Goals / Non-Goals

**Goals:**
- Treat sync-route entry as an explicit exception to normal list-view navigation semantics by resetting the shared playback session as soon as the route becomes active.
- Clear `active_playback` and any active Search playback snapshot on sync-route entry without discarding Search, History, or Watch Later browsing state.
- Keep the PiP window open, but make both main window and PiP converge on the same empty playback state via the backend authoritative refresh path.
- Ensure returning from `/scraper` restores list browsing state without reviving the previously selected playback item.
- Encode PiP compact header width priority so the stats row stays inline and uploader text truncates first, while preserving the fixed compact shell and existing title line-state rebalance behavior.
- Add spec-driven regression coverage for both the sync-route reset behavior and the compact-header presentation contract.

**Non-Goals:**
- Do not change the existing rule that navigating among browseable list views preserves playback unless the active list itself becomes invalid.
- Do not clear Search, History, or Watch Later query/sort/filter/page/result state on sync-route entry.
- Do not close the PiP window as part of the sync-route reset flow.
- Do not redesign the PiP shell height, player sizing, or compact title line-measurement model.
- Do not implement a full browsing-session reset or new route categories beyond the sync-page exception.

## Decisions

### 1. Model `/scraper` as a route-entry playback reset boundary, not as a generic visible-page switch
Sync-route entry will be treated as an explicit playback-session boundary. Browseable list views continue to preserve active playback when the user switches among them, but entering `/scraper` immediately clears active playback instead of relying on later list invalidation.

**Why:** This preserves the core browsing/playback separation for list views while recognizing that the sync route is a workflow boundary rather than another playback-capable list surface.

**Alternatives considered:**
- **Reset only when sync actually starts:** rejected because the user explicitly wants route entry itself to end the playback session.
- **Apply the same reset rule to every route change:** rejected because it would undo the existing multi-view playback model.

### 2. Keep the route-entry reset scoped to Level 1 playback state only
The reset will clear `active_playback`, invalidate the Search playback snapshot, and sever the current playback/list binding, but it will leave `list_contexts`, `search_state`, `history_state`, and `watch_later_state` intact.

**Why:** The desired return behavior is “no selected/playing item” rather than “rerun the view from scratch.” Preserving browsing state keeps query, sorting, filters, pagination, and loaded results available when the user goes back.

**Alternatives considered:**
- **Full browsing-state reset:** rejected because it would discard useful context the user wants to keep.
- **Playback-only reset without snapshot invalidation:** rejected because the frozen Search watched boundary is part of the active playback session being terminated.

### 3. Use the existing authoritative refresh path for both windows and keep PiP open but empty
The sync-route reset should not create a new window-local fallback path. Backend state clears first, then both windows refresh from `get_playlist_state()`. The PiP window remains open, but because no active playback remains, it renders the existing no-selection empty state.

**Why:** This keeps Rust as the single source of truth and avoids a divergence where the main window appears cleared while PiP still holds a stale local playback ref.

**Alternatives considered:**
- **Close PiP on route entry:** rejected because the user wants the PiP window to remain open.
- **Manually null frontend refs without backend reset:** rejected because it would violate the authoritative-state contract.

### 4. Preserve list restoration state while explicitly dropping selected-item revival after sync-route reset
The views should continue restoring their saved list/query/pagination state, but returning from `/scraper` must no longer recreate the previously playing selection unless a new playback session has since been established.

**Why:** The requested behavior is a list-focused return, not a playback-focused one. The restored view should feel familiar without implicitly reclaiming playback ownership.

**Alternatives considered:**
- **Re-use the old selected index if results are still present:** rejected because it would contradict the explicit route-entry reset.

### 5. Represent PiP header priority as a compact presentation-contract rule, not a one-off CSS patch
The compact metadata contract will explicitly prioritize the stats row over uploader-width expansion. In practice, the stats cluster should be treated as a no-wrap inline group, while the uploader segment becomes the first truncation target through flex shrink and ellipsis.

**Why:** This matches the existing contract-oriented metadata architecture and makes future tests assert intent (“stats remain inline; uploader truncates first”) instead of brittle style literals.

**Alternatives considered:**
- **Only add `white-space: nowrap` to `.stat`:** helps, but leaves the priority rule implicit and easier to regress.
- **Add JS measurement for uploader/stats width negotiation:** rejected because the desired behavior fits CSS layout mechanics and does not require another measurement loop.

## Risks / Trade-offs

- **[Sync-route exception leaks into generic route handling]** → Mitigation: scope the new reset rule explicitly to `/scraper` entry and preserve existing browseable-list semantics everywhere else.
- **[Browsing state accidentally cleared alongside playback]** → Mitigation: codify Level 1-only behavior in specs and tests, especially around Search restoration after returning from the sync route.
- **[PiP/main window divergence during reset]** → Mitigation: route both surfaces through the existing backend authoritative refresh path and avoid frontend-only clearing.
- **[Compact stats no-wrap overflows at extreme widths]** → Mitigation: treat the PiP minimum supported width as the contract boundary and verify compact layout behavior within that intended size range.
- **[Presentation-contract drift]** → Mitigation: add source-contract tests for compact stats priority instead of relying only on visual inspection.

## Migration Plan

1. Update specs to define the sync-route reset exception and compact-header priority contract.
2. Add backend/frontend reset orchestration so entering `/scraper` clears authoritative playback and refreshes both windows.
3. Update view restoration behavior so browsing state survives while selection revival does not.
4. Tighten compact header layout contract and corresponding tests.
5. Verify with targeted frontend tests and the shared player verification commands before implementation completion.

## Open Questions

- Whether the sync-route reset should be triggered by a route watcher in `App.vue`, a router guard, or a dedicated sync-entry abstraction can be decided during implementation as long as the spec-level behavior stays the same.
- Whether `active-playback-cleared` should carry an explicit route-reset reason in addition to the cleared list identity is optional; the current design can work with the existing event semantics if both windows only use it as a refresh trigger.
