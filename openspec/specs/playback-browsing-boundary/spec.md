## ADDED Requirements

### Requirement: Browsing context and playback context are independent
The system SHALL maintain browsing context independently from active playback context so that navigation among browseable list views does not change the playback-bound list. Entering the sync route SHALL be treated as an explicit playback-session reset boundary that clears active playback without discarding stored browsing contexts.

#### Scenario: Switching visible list pages preserves active playback
- **WHEN** the user navigates from one browseable list view to another without explicitly selecting a new video
- **THEN** the active playback reference SHALL remain bound to the previously playing list context and item index
- **AND** the newly visible view SHALL update only its browsing context

#### Scenario: Entering the sync route clears active playback but preserves browsing contexts
- **WHEN** the user navigates to the sync route while a playback session is active
- **THEN** the backend SHALL clear the active playback reference
- **AND** any active Search playback snapshot SHALL be invalidated
- **AND** stored Search, History, and Watch Later browsing contexts SHALL remain available for later restoration

#### Scenario: Restoring a non-playing view does not rewrite playback binding
- **WHEN** a list view restores its local browsing state on mount or reload
- **AND** that list is not the one currently bound to active playback
- **THEN** the restore flow SHALL NOT change the active playback reference
- **AND** the player SHALL continue rendering the currently active playback session

#### Scenario: Returning from the sync route restores browsing state without reviving playback
- **WHEN** the user returns from the sync route to a browseable list view
- **THEN** that view SHALL restore its existing browsing state if one exists
- **AND** it SHALL NOT recreate the previously active playback selection unless a new playback session has been established after the reset

### Requirement: Playback clear events represent actual playback invalidation
The system SHALL emit playback clear events only when the current active playback session becomes invalid, either because the active list changes in an invalidating way or because an explicit sync-route reset terminates the session. Non-active list refreshes and ordinary browseable-list navigation SHALL NOT emit a playback-cleared event.

#### Scenario: Active list result change emits playback cleared
- **WHEN** the list context currently bound to active playback changes membership or ordering in a way that invalidates the active session
- **THEN** the backend SHALL clear the active playback reference
- **AND** the backend SHALL emit `active-playback-cleared`

#### Scenario: Entering the sync route emits playback cleared for the terminated session
- **WHEN** the user enters the sync route while a playback session is active
- **THEN** the backend SHALL clear the active playback reference before the sync route becomes the active surface
- **AND** the backend SHALL emit `active-playback-cleared` for the cleared playback session

#### Scenario: Non-active list refresh does not emit playback cleared
- **WHEN** a non-playing list context refreshes, restores, or bumps version
- **THEN** the backend SHALL preserve the active playback reference
- **AND** the backend SHALL NOT emit `active-playback-cleared`

### Requirement: All windows consume the same authoritative playback snapshot
The system SHALL synchronize main window and PiP window player displays from the same backend authoritative playback snapshot.

#### Scenario: Video selection refreshes both windows from authoritative state
- **WHEN** the user selects a video and backend playback state changes
- **THEN** both the main window and PiP window SHALL resolve player display state from the backend authoritative playback snapshot
- **AND** neither window SHALL rely on a window-local playback binding that can diverge from backend state

#### Scenario: Playback clear refreshes both windows from authoritative state
- **WHEN** the backend clears active playback
- **THEN** both the main window and PiP window SHALL transition to empty player state from the backend authoritative playback snapshot
- **AND** the two windows SHALL observe the same cleared playback session outcome
