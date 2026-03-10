# Load-more independence from playback (Search)

## Context
Search load-more currently rejects when `active_playlist_type` is not Search. This breaks infinite scroll when playback is bound to History and the user switches to Search.

## Decision
Make `load_more` validate against the requested list context (Search) and its version, not the active playback list.

## Architecture
- **List context (Search):** owns browsing state (query/sort/filters), pagination, version.
- **Active playback:** only indicates which list owns playback + index; does not gate browsing.
- **load_more:** accepts requests when list context id + version match. Playback is ignored.

## Data Flow
1. SearchView scroll triggers `api.loadMore('Search', searchState.version)`.
2. Backend reads Search list context + version.
3. If id/version match, fetch next page and append to Search context.
4. Return results to frontend and update has_next.

## Error Handling
- Version mismatch => stale error (existing behavior).
- Missing Search context => no-context error; frontend should re-run search.
- has_next false => no more results.

## Testing
- Playback bound to History, then Search scroll → load-more still succeeds.
- Version mismatch still rejected.
- Manual verification: History playback → Search scroll to end loads next page.
