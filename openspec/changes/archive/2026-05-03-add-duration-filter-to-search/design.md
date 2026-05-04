## Context

Desktop Search already stores `duration` in `videos.db`, selects it in Search results, and exposes it to the frontend as `Video.duration`. The current advanced filter modal only sends numeric ranges for views, mylists, comments, likes, plus an upload-date range. Search page-one requests reserve a new Rust `ListContext.version` with the full query/sort/filter payload, while `load_more` rebuilds the next-page request from the stored Search list context. Any duration filter must therefore become part of the same request payload stored in Rust, not a frontend-only post-filter.

The worker reference implements duration as another numeric filter in the existing range-filter family and presents the modal inputs as duration-aware text fields. Desktop can reuse the contract idea (`duration: { gte, lte }` in seconds) while keeping desktop’s Rust/Tauri state model authoritative.

## Goals / Non-Goals

**Goals:**
- Add duration min/max controls to the Search advanced filter modal.
- Serialize duration filters into the existing Search request `filters` object as seconds.
- Persist and restore duration controls with the existing Search UI state defaults.
- Apply duration predicates in Rust SQL for initial Search and load-more pagination.
- Keep Search versioning, active-playback reset, watched-boundary snapshot behavior, and PiP synchronization on the existing Rust authoritative path.
- Cover the change with targeted TypeScript and Rust tests.

**Non-Goals:**
- Do not change the `videos` table schema; `duration` already exists.
- Do not add duration sorting or formula weighting.
- Do not post-filter Search results in the frontend.
- Do not redesign the advanced filter modal beyond adding the duration range row and helper copy.
- Do not change playback, PiP, CSP, or Niconico embed behavior.

## Decisions

### 1. Treat duration as an existing numeric filter family member

Add `duration?: NumericFilter` to TypeScript `Filters` and `duration: Option<NumericFilter>` to Rust `models::Filters`. The outgoing request shape will match existing metric ranges:

```json
{
  "filters": {
    "duration": { "gte": 60, "lte": 300 }
  }
}
```

Rationale: the database stores duration as integer seconds and the worker reference already uses the same `{ gte, lte }` numeric filter shape. This avoids a duration-specific API contract and lets `reserve_list_context_version`, `finalize_list_context_search`, and `update_list_context` store duration with the same filter payload clone they already handle.

Alternatives considered:
- Add `duration_gte` / `duration_lte` as top-level request fields: rejected because it would bypass the established filter family and require special versioning comparisons.
- Add a duration-specific object with unit metadata: rejected because seconds are already the canonical stored/result unit.

### 2. Parse user-facing duration controls to seconds at the UI boundary

The modal should expose min/max duration inputs next to the other numeric filters. Use the worker pattern conceptually: accept empty values as unset, accept plain digits as seconds, and accept `m:ss` for readability. Persist only parsed seconds (`durationGte`, `durationLte`) in `SearchPersistenceState`; do not persist raw edit strings. If the desktop implementation keeps the modal inline in `SearchView.vue`, local raw-input refs can preserve in-progress text while the modal is open.

Rationale: users think in song lengths, while the backend needs a stable numeric value. Keeping parsed seconds as the persisted contract makes restore, request building, and Rust state deterministic.

Alternatives considered:
- Use `type="number"` only and require seconds: simpler but worse UX and less aligned with the worker reference.
- Persist raw strings and parse in Rust: rejected because invalid UI text should not become part of authoritative Search context identity.

### 3. Apply duration in Rust search construction before pagination and counting

Add `v.duration >= ?` and `v.duration <= ?` clauses wherever existing numeric filters are applied. The predicates must be included in both result SQL and count SQL, and must run through parameter binding like the existing metric filters. Load-more needs no separate UI input handling because it uses `context.filters.clone()` from the stored Search list context.

Rationale: SQL-level filtering preserves correct `total`, `has_next`, stable pagination, and playback snapshot membership. Frontend post-filtering would corrupt pagination counts and continuous playback assumptions.

Alternatives considered:
- Filter after fetching a page: rejected because page boundaries and totals would become false.
- Add a new duration index immediately: not required for correctness. Consider only if profiling shows duration range queries are slow.

### 4. Duration changes are Search result-set changes

When the user applies a changed duration range, the Search request includes different filters. Existing page-one `search()` behavior will reserve a new Search list-context version and update browsing params atomically. If active playback is bound to Search, the existing active-playback clearing and Search playback snapshot invalidation paths must apply exactly as they do for other filter changes. If playback is bound to History or Watch Later, only Search browsing state changes.

Rationale: duration changes alter effective membership and ordering boundaries. They must participate in the same version and playback-session boundary contracts as all other result-shaping controls.

Alternatives considered:
- Treat duration as a UI-only refinement that does not advance version: rejected because stale load-more results could be merged into a different result set.

### 5. Keep double-window state authoritative in Rust

The main window can persist control values in local UI state for form restoration, but Search results, pagination, version, and active playback remain Rust-owned. PiP and mounted list views should observe the same Search list context returned by `get_search_state`; no separate PiP duration state is introduced.

Rationale: this matches the current Rust single-source-of-truth architecture and avoids cross-window divergence.

Alternatives considered:
- Broadcast duration filters through a frontend-only event: rejected because it duplicates Rust state and risks PiP/main-window mismatch.

## Risks / Trade-offs

- [Invalid or partially typed duration input could serialize as a real filter] → Only parsed finite seconds are written to `SearchPersistenceState`; invalid/empty input remains unset and is not sent.
- [Duration min greater than max can produce confusing empty results] → Keep behavior consistent with existing numeric filters for this change; optionally add validation in a later UI-polish change if current numeric filters gain the same validation.
- [Updating only first-page Search but not load-more would corrupt pagination] → Add Rust query tests and ensure load-more uses stored `context.filters.clone()` with duration included.
- [Adding duration to one query-builder copy but not another causes test/production drift] → Update both the test-only `build_search_query` helper and runtime `execute_search`/`search` query construction, or consolidate duplicated filter construction if the implementation scope permits.
- [Older localStorage state lacks duration fields] → `restoreSearchPersistenceState` defaults duration fields to `undefined`, preserving backward compatibility.
- [Performance may degrade on large databases without a duration index] → Do not add an index speculatively; profile after implementation if duration filtering is slow.

## Migration Plan

1. Extend frontend/Rust request models with optional duration filters defaulting to absent.
2. Add duration controls, labels, persistence, request serialization, active-filter detection, and reset behavior.
3. Add SQL predicates and tests for duration-only and combined filters.
4. Run targeted frontend tests for `searchViewState` / `useSearchFilters` and Rust tests for Search query construction or command behavior.
5. Rollback is safe by removing the optional duration fields and UI controls; no data migration is involved.

## Open Questions

- None for proposal scope.
