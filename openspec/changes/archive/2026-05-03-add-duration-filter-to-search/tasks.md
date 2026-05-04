## 1. Frontend Contract and State

- [x] 1.1 [Frontend] Extend `SearchRequest.filters` and `SearchPersistenceState` with optional `duration` / `durationGte` / `durationLte` fields in seconds.
- [x] 1.2 [Frontend] Update `restoreSearchPersistenceState`, `createSearchPersistenceState`, and `buildSearchRequest` so duration defaults to unset and serializes as `filters.duration.{gte,lte}` only when finite.
- [x] 1.3 [Frontend] Update `useSearchFilters` return type, refs, active-filter detection, reset behavior, and `getFilterState` to include duration min/max.

## 2. SearchView UI

- [x] 2.1 [Frontend] Add duration min/max controls to the SearchView advanced filter modal near the other range filters, accepting empty, seconds, and `m:ss` input as parsed seconds.
- [x] 2.2 [Frontend] Include duration values in SearchView save/load state, search request creation, active-filter badge logic, reset behavior, and the state persistence watcher.
- [x] 2.3 [Frontend] Add localized duration label and helper text for `en`, `ja`, and `zh-TW` filter copy.

## 3. Rust Search Filtering

- [x] 3.1 [Rust] Add optional `duration: NumericFilter` to `models::Filters` without changing the videos schema.
- [x] 3.2 [Rust] Apply `v.duration >= ?` and `v.duration <= ?` parameterized SQL predicates in all Search query construction paths used by page-one search and tests.
- [x] 3.3 [Rust] Verify `load_more` reuses the stored Search `context.filters.clone()` with duration and does not require `state.rs` changes; if `state.rs` is touched, document why in the implementation notes.

## 4. Tests and Verification

- [x] 4.1 [Frontend] Update `searchViewState.test.ts` and `useSearchFilters.test.ts` to cover duration serialization, restore defaults, active-filter detection, and reset behavior.
- [x] 4.2 [Rust] Add command/query tests proving duration filters generate the expected SQL predicates and parameters.
- [x] 4.3 [Both] Run targeted verification: `npx vitest run src/features/playlistViews/searchViewState.test.ts src/composables/useSearchFilters.test.ts` and `cd src-tauri && cargo test build_query_with_duration_filter` or the relevant Search query test name.
