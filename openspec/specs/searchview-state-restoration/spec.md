## MODIFIED Requirements

### Requirement: SearchView restores state from Rust on mount
The system SHALL restore SearchView browsing state from its Rust-managed Search list context on mount, without implicitly restoring playback unless Search is still the active playback list context at the same version.

#### Scenario: Restore Search browsing state when returning to the page
- **WHEN** the user returns to SearchView after switching to another tab
- **AND** Rust has an existing Search list context
- **THEN** SearchView restores Search results, sorting, filters, and pagination progress from that list context
- **AND** it does NOT execute a new search unless the Search list context is absent or explicitly invalidated

#### Scenario: Initial search when no Search context exists
- **WHEN** SearchView mounts for the first time
- **AND** Rust has no existing Search list context
- **THEN** SearchView executes a new search to create the Search list context
- **AND** the resulting context becomes the source for later Search browsing restoration

#### Scenario: Do not revive stale Search playback
- **WHEN** the user returns to SearchView
- **AND** active playback has already moved to another list context or another Search version
- **THEN** SearchView restores browsing state only
- **AND** it does not mark a Search item as currently playing

#### Scenario: Returning from the sync route restores browsing without reviving the cleared selection
- **WHEN** the user returns to SearchView after entering the sync route
- **AND** the prior Search browsing context still exists
- **THEN** SearchView restores the saved Search results, query, sorting, filters, and pagination progress
- **AND** it does not recreate the previously playing Search selection unless a new playback session has since been established

### Requirement: SearchView state persists across tab switches
The system SHALL preserve Search browsing state across tab switches and sync-route playback resets independently of active playback.

#### Scenario: Search browsing state survives tab switch during other-list playback
- **WHEN** the user leaves SearchView while playback later switches to History, Watch Later, or another list
- **THEN** Rust retains the Search list context for later browsing restoration
- **AND** SearchView can restore its prior browsing state without reclaiming playback

#### Scenario: Search browsing state survives sync-route playback reset
- **WHEN** the user enters the sync route while Search browsing state exists
- **THEN** Rust retains the Search list context and saved Search state
- **AND** returning to SearchView restores that browsing state without reviving playback

#### Scenario: Search query changes reset active Search playback
- **WHEN** active playback is currently bound to the Search list context
- **AND** the user changes Search query, sort, or filters
- **THEN** the Search list context advances to a new version
- **AND** active playback is reset instead of continuing on the changed Search result set
