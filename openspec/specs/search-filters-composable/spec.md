## ADDED Requirements

### Requirement: Search filters composable manages sort state
The system SHALL provide a Vue composable `useSearchFilters` that manages sort and filter UI state.

#### Scenario: Default sort configuration
- **WHEN** composable is first initialized
- **THEN** sort.by SHALL have a default value
- **AND** sort.direction SHALL have a default value

#### Scenario: Sort field change
- **WHEN** user changes sort field
- **THEN** sort.by SHALL update to the new value
- **AND** sort.direction SHALL remain unchanged (or reset to default)

#### Scenario: Sort direction toggle
- **WHEN** user toggles sort direction
- **THEN** sort.direction SHALL switch between asc and desc

### Requirement: Search filters composable manages filter state
The system SHALL manage filter parameters through the composable, including duration min/max ranges expressed in seconds.

#### Scenario: Filter parameter update
- **WHEN** a filter parameter is changed
- **THEN** the corresponding reactive state SHALL update
- **AND** the change SHALL be reflected in buildSearchRequest output

#### Scenario: Duration filter parameter update
- **WHEN** duration minimum or maximum is changed to a valid duration value
- **THEN** the corresponding duration filter state SHALL update as seconds
- **AND** buildSearchRequest SHALL include `filters.duration.gte`, `filters.duration.lte`, or both according to the current values

#### Scenario: Filter reset
- **WHEN** resetFilters function is called
- **THEN** all filter parameters SHALL return to default values
- **AND** duration minimum and maximum SHALL return to unset values
- **AND** sort configuration SHALL return to default values

### Requirement: Search filters composable provides reactive filter params
The system SHALL provide computed filter parameters for search requests.

#### Scenario: Computed filter params update
- **WHEN** any filter state changes
- **THEN** the computed filterParams SHALL reflect the current state
- **AND** the format SHALL match the API expected format

#### Scenario: Duration filter params use numeric range format
- **WHEN** duration minimum or maximum is set
- **THEN** computed filter params SHALL include `duration` using the same `{ gte, lte }` numeric range format as other numeric filters
- **AND** unset duration bounds SHALL be omitted from the request

### Requirement: Exclude watched toggle
The system SHALL provide a toggle for excluding watched videos.

#### Scenario: Exclude watched is toggled
- **WHEN** excludeWatched is set to true
- **THEN** buildSearchRequest SHALL include exclude_watched: true

#### Scenario: Exclude watched is off
- **WHEN** excludeWatched is set to false
- **THEN** buildSearchRequest SHALL include exclude_watched: false
