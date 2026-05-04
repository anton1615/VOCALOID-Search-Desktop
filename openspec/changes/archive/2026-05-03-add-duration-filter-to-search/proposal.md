## Why

Search advanced filters already support activity metrics and upload date ranges, but users cannot narrow results by video length even though desktop search results and `videos.db` already carry `duration`. The worker implementation has validated the product shape for duration ranges, so desktop should add the same end-to-end filter without changing playback or browsing-state contracts.

## What Changes

- Add a video-duration min/max section to the SearchView advanced filter modal.
- Persist duration filter controls with the existing Search filter state and include them in active-filter detection and reset behavior.
- Extend the frontend `SearchRequest.filters` contract and Rust `Filters` model with a duration numeric range.
- Apply duration constraints in Rust search SQL for both first-page search and paginated Search load-more using the stored Search parameters.
- Keep Search list-context versioning unchanged: changing duration filters creates a new Search result identity, invalidates stale load-more requests, and resets active Search playback only when Search is the active playback context.
- Reuse the existing double-window synchronization path: main window and PiP observe the Rust Search list context rather than maintaining separate duration-filter state.
- No breaking changes; older persisted Search UI state simply has duration filters unset.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `video-search`: Search filters accept and apply duration min/max ranges in addition to existing numeric and date filters.
- `search-filters-composable`: Search filter state includes duration min/max controls, active-filter detection, reset behavior, and request serialization.
- `searchview-state-restoration`: Search browsing restoration preserves duration filters as part of the saved Search query/filter state and treats duration changes as result-set changes.

## Impact

- Frontend: `SearchView.vue`, `searchViewState.ts`, `useSearchFilters.ts`, related Search/filter tests, and localized filter labels.
- API contract: TypeScript `SearchRequest.filters` gains a `duration` numeric filter matching the existing `{ gte, lte }` shape.
- Rust: `models::Filters` gains duration; search query construction adds `v.duration >= ?` / `v.duration <= ?` predicates and targeted query tests.
- Data/schema: no migration required because `videos.duration` already exists and search results already expose `duration`.
- State/sync: Rust remains the source of truth; `reserve_list_context_version` stores the new filter payload with the existing Search context, so load-more and PiP synchronization continue through the authoritative state path.
