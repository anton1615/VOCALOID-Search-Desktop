## MODIFIED Requirements

### Requirement: Backend notifies frontend when active playback is cleared
The system SHALL emit an event when `active_playback` is cleared due to active-list mutations or an explicit sync-route reset.

#### Scenario: Search list mutation clears active playback and notifies
- **WHEN** active playback is bound to Search list context
- **AND** the user changes Search query, sort, filters, or another condition that changes results
- **THEN** the backend clears `active_playback`
- **AND** the backend emits `active-playback-cleared` event with the cleared list context ID

#### Scenario: History list mutation clears active playback and notifies
- **WHEN** active playback is bound to History list context
- **AND** the user changes History sort order
- **THEN** the backend clears `active_playback`
- **AND** the backend emits `active-playback-cleared` event with `History`

#### Scenario: Watch Later list mutation clears active playback and notifies
- **WHEN** active playback is bound to Watch Later list context
- **AND** the user changes Watch Later sort order
- **THEN** the backend clears `active_playback`
- **AND** the backend emits `active-playback-cleared` event with `WatchLater`

#### Scenario: Sync-route entry clears active playback and notifies
- **WHEN** the user enters the sync route while active playback is bound to Search, History, or Watch Later
- **THEN** the backend clears `active_playback`
- **AND** any active Search playback snapshot is invalidated
- **AND** the backend emits `active-playback-cleared` event with the cleared playback list context ID

### Requirement: Frontend responds to playback reset notification
The system SHALL refresh playback state from Rust when receiving the `active-playback-cleared` event.

#### Scenario: Main window refreshes currentVideo on event
- **WHEN** the frontend receives `active-playback-cleared` event
- **THEN** `App.vue` calls `refreshActivePlayback()`
- **AND** `currentVideo` is updated from `getPlaylistState()`
- **AND** if `active_playback` is `None`, `currentVideo` becomes `null`

#### Scenario: PiP refreshes to empty playback state without closing the window
- **WHEN** the PiP window receives `active-playback-cleared` after sync-route entry
- **THEN** PiP refreshes its playback state from `getPlaylistState()`
- **AND** it renders the no-selection player state
- **AND** the PiP window remains open

#### Scenario: Player surfaces reset when currentVideo becomes null
- **WHEN** `currentVideo` becomes `null` after playback reset
- **THEN** the main-window player surface hides the iframe or remains absent according to the current route
- **AND** the PiP player surface stops rendering an active video iframe
