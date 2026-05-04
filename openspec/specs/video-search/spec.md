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

### Requirement: Exclude watched filter works without user ID

The system SHALL support "exclude watched" filter for the single local user.

#### Scenario: Live Search excludes currently watched videos
- **WHEN** user enables "exclude watched" filter
- **AND** no active Search playback snapshot applies to the current Search session
- **THEN** search results exclude videos marked as watched in the latest local history state
- **AND** watched status is based on local history data, not a remote user identity

#### Scenario: Active Search playback keeps membership stable across newly watched videos
- **WHEN** user enables "exclude watched" filter
- **AND** the current Search session has an active Search playback snapshot
- **AND** videos become watched during that same playback session
- **THEN** subsequent Search pagination for that Search session excludes only videos watched before the frozen boundary
- **AND** videos watched during the same playback session remain eligible for that session's continued pagination
- **AND** the UI may still display those videos as watched


## ADDED Requirements (fix-frontend-alignment change)

### Requirement: Advanced filter modal closes only on deliberate backdrop click

The system SHALL only close the advanced filter modal when the user deliberately clicks on the backdrop, not when releasing a mouse drag that started inside the modal.

#### Scenario: Mouse down on backdrop, mouse up on backdrop
- **WHEN** user presses mouse button on backdrop
- **AND** user releases mouse button on backdrop
- **THEN** modal closes

#### Scenario: Mouse down inside modal, mouse up on backdrop
- **WHEN** user presses mouse button inside modal (e.g., on input field)
- **AND** user drags mouse outside modal
- **AND** user releases mouse button on backdrop
- **THEN** modal does NOT close
- **AND** user can continue editing

#### Scenario: Text selection in input field
- **WHEN** user starts text selection in input field
- **AND** user drags mouse outside modal while selecting
- **AND** user releases mouse button outside modal
- **THEN** modal does NOT close
- **AND** text selection is preserved


## ADDED Requirements (resolve-technical-debt change)

### Requirement: Search commands use structured error types
The system SHALL return `Result<T, AppError>` from search-related Tauri commands.

#### Scenario: Search command error handling
- **WHEN** search command encounters a database error
- **THEN** it returns `AppError::Database` variant
- **AND** frontend can categorize the error type

#### Scenario: Search command validation error
- **WHEN** search command receives invalid parameters
- **THEN** it returns `AppError::Validation` variant with description
- **AND** frontend can display specific validation feedback

### Requirement: Search queries use parameterized SQL
The system SHALL construct search SQL queries using `VideoQueryBuilder` with parameterized statements.

#### Scenario: Search with user input
- **WHEN** user provides search query or filter values
- **THEN** values are passed as SQL parameters
- **AND** no string concatenation occurs in SQL construction

#### Scenario: Search with sort parameter
- **WHEN** user specifies sort direction
- **THEN** the sort field and direction are validated
- **AND** invalid values are rejected with `AppError::Validation`