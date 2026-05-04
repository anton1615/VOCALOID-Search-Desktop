## MODIFIED Requirements

### Requirement: User can search videos using Tauri IPC

The system SHALL allow users to search for videos using the Tauri invoke API instead of HTTP requests.

#### Scenario: Execute search
- **WHEN** user submits a search query
- **THEN** frontend calls Tauri command `search` with search parameters
- **AND** Rust backend queries SQLite database
- **AND** results are returned to frontend

#### Scenario: Search with filters
- **WHEN** user applies filters for views, mylists, comments, likes, duration, or upload date range
- **THEN** frontend passes filters to Tauri command
- **AND** Rust backend applies filters to SQL query using parameterized predicates
- **AND** duration filters compare against the `videos.duration` value in seconds

#### Scenario: Search with formula filter
- **WHEN** user enables formula filter
- **THEN** frontend passes formula weights and minimum score
- **AND** Rust backend calculates formula scores and filters results

#### Scenario: Paginated results
- **WHEN** search returns more results than page size
- **THEN** frontend receives total count and current page
- **AND** `AppState.search_state.has_next` is set accordingly
- **AND** user can load more results via `load_more()` command
- **AND** pagination ordering is stable for a fixed query and watched-exclusion identity

#### Scenario: Stable pagination ordering for tied sort values
- **WHEN** the user sorts by a field that can contain ties (e.g., Like)
- **AND** multiple results share the same primary sort value
- **THEN** the backend SHALL apply a deterministic tie-breaker so that the global ordering is total and repeatable
- **AND** page boundaries produced by `LIMIT/OFFSET` do not skip or duplicate items when pages are concatenated

#### Scenario: Search playback and manual scrolling share pagination rules
- **WHEN** Search load-more is triggered by manual scrolling or continuous playback
- **THEN** the backend SHALL use the same Search pagination flow for both
- **AND** the backend SHALL resolve watched exclusion from either the live watched history or the active Search playback snapshot boundary, depending on which Search session state is active

#### Scenario: Search state persisted across windows
- **WHEN** search is performed in main window
- **AND** PiP window queries search state
- **THEN** PiP receives same search parameters and results from `AppState`
