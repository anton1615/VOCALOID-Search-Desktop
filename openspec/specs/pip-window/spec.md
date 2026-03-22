## ADDED Requirements

### Requirement: User can open PiP window for video playback

The system SHALL allow users to open a Picture-in-Picture window that displays the current video player in a compact, always-on-top window.

#### Scenario: Open PiP from main window
- **WHEN** user clicks "Open in PiP" button in the player area
- **THEN** system creates a new popup window with the video player
- **AND** the main window's player area shows "Playing in PiP window" placeholder
- **AND** the PiP window is set to always-on-top
- **AND** PiP immediately loads and displays the current video from shared state

#### Scenario: PiP window layout
- **WHEN** PiP window is opened
- **THEN** the window displays:
  - Left sidebar (60px, fixed): Previous, Play/Pause, Next buttons, Settings toggle
  - Right column (scrollable): Video title, uploader name, video attributes, Niconico embed player, tags, description
- **AND** window size is approximately 450x500 pixels minimum
- **AND** the entire right column is scrollable while left sidebar stays fixed

#### Scenario: PiP displays current video immediately
- **WHEN** PiP window opens
- **THEN** it calls `get_playlist_state()` to fetch current playlist
- **AND** displays the video at current `playlist_index`
- **AND** loads the Niconico embed player for that video

### Requirement: User can control playback from PiP window

The system SHALL provide playback controls in the PiP window that synchronize with main window via shared state.

#### Scenario: Play/pause from PiP
- **WHEN** user clicks play/pause button in PiP window
- **THEN** the video plays or pauses accordingly
- **AND** the play state is synchronized with main window

#### Scenario: Skip to next video from PiP
- **WHEN** user clicks "next" button in PiP window
- **THEN** PiP calls `set_playlist_index(currentIndex + 1)`
- **AND** the backend broadcasts `video-selected` event
- **AND** both PiP and main window update to show the new video

#### Scenario: Previous video from PiP
- **WHEN** user clicks "previous" button in PiP window
- **THEN** PiP calls `set_playlist_index(currentIndex - 1)`
- **AND** the backend broadcasts `video-selected` event
- **AND** both windows update to show the previous video

#### Scenario: Select video from main window while PiP is open
- **WHEN** PiP window is open
- **AND** user clicks a video in the main window's search results
- **THEN** main window calls `set_playlist_index(videoIndex)`
- **AND** PiP receives `video-selected` event
- **AND** PiP loads and plays the selected video

### Requirement: PiP window can be closed

The system SHALL allow users to close the PiP window and return playback to the main window.

#### Scenario: Close PiP window
- **WHEN** user closes the PiP window
- **THEN** the PiP window is destroyed
- **AND** the main window SHALL regain playback UI ownership through a single authoritative refresh path
- **AND** the main window's player area SHALL return to normal using refreshed shared playback state

#### Scenario: Placeholder remains until playback state is refreshed
- **WHEN** the main window receives the PiP-close notification
- **THEN** the PiP placeholder SHALL remain visible while authoritative playback state is refreshed
- **AND** the main window SHALL NOT reveal a transient player-only regain state before refreshed metadata is available

#### Scenario: Main window restores refreshed playback metadata after PiP close
- **WHEN** PiP closes while a video is still the active playback item
- **THEN** the main window SHALL refresh the authoritative playback snapshot before leaving placeholder mode
- **AND** the shared player SHALL reappear with the refreshed playback identity and metadata applied
- **AND** the regain transition SHALL complete only after the refreshed snapshot is available

### Requirement: PiP window is always on top

The system SHALL keep the PiP window visible above other application windows.

#### Scenario: PiP window stays on top
- **WHEN** PiP window is open
- **AND** user focuses another application window
- **THEN** the PiP window remains visible on top of other windows

### Requirement: PiP window supports autoSkip

The system SHALL support autoSkip functionality in PiP window with the same behavior as main window.

#### Scenario: AutoSkip setting shared
- **WHEN** PiP window opens
- **THEN** it reads `autoSkip` and `skipThreshold` from shared state
- **AND** displays the settings in the left sidebar

#### Scenario: AutoSkip triggers in PiP
- **WHEN** video is playing in PiP
- **AND** remaining time is less than or equal to `skipThreshold` seconds
- **AND** `autoSkip` is enabled
- **THEN** PiP calls `set_playlist_index(currentIndex + 1)`
- **AND** next video starts playing

#### Scenario: Toggle autoSkip from PiP
- **WHEN** user toggles autoSkip in PiP window
- **THEN** PiP calls `set_playback_settings({ autoSkip: newValue })`
- **AND** main window's autoSkip setting updates accordingly

### Requirement: Auto-skip to next video on end

The system SHALL automatically play the next video when the current video ends in PiP window (if auto-play is enabled).

#### Scenario: Auto-play next video in PiP
- **WHEN** current video ends in PiP window
- **AND** auto-play setting is enabled
- **THEN** the next video in the search results automatically starts playing

### Requirement: PiP refactors preserve shared playback synchronization
The system SHALL preserve the existing synchronization contract between the PiP window, Rust shared state, and the main window during internal frontend refactors.

#### Scenario: PiP stays synchronized after controller extraction
- **WHEN** PiP playback logic is reorganized behind reusable abstractions
- **THEN** PiP still loads the current shared playlist entry on open
- **AND** it still responds to `video-selected` events from shared playlist changes
- **AND** its playback controls continue to update the same shared playlist state used by the main window

#### Scenario: PiP close flow remains behaviorally unchanged
- **WHEN** PiP-related frontend structure is refactored
- **THEN** closing PiP still clears the active PiP state and restores the main window player area correctly
- **AND** the refactor does not bypass the existing close-notification flow relied on by the shared session model

## MODIFIED Requirements

### Requirement: PiP compact header rebalances within a fixed shell
The system SHALL keep the PiP compact header shell height stable while allowing the internal metadata header layout to rebalance based on the title’s rendered line count and the available width of the uploader/stats row.

#### Scenario: Single-line compact title rebalances the fixed header shell
- **WHEN** the PiP window displays a compact metadata header
- **AND** the rendered title occupies one line at the current PiP width
- **THEN** the PiP header SHALL use the compact single-line presentation variant
- **AND** the title/date row and uploader/stats row SHALL rebalance within the existing fixed shell height
- **AND** the embedded player position SHALL remain stable

#### Scenario: Two-line compact title preserves the existing fixed header shell
- **WHEN** the PiP window displays a compact metadata header
- **AND** the rendered title occupies two lines at the current PiP width
- **THEN** the PiP header SHALL use the compact two-line presentation variant
- **AND** the PiP header SHALL preserve the same fixed shell height used by the single-line variant
- **AND** the embedded player position SHALL remain stable

#### Scenario: Constrained compact meta row keeps stats inline and truncates uploader first
- **WHEN** the PiP compact header has limited horizontal space in the uploader/stats row
- **THEN** the four stats SHALL remain on a single inline row
- **AND** the uploader identity segment SHALL yield width first through truncation
- **AND** the stats SHALL NOT reflow into icon-on-top number-on-bottom stacks

#### Scenario: PiP resize re-evaluates compact title line state and meta-row distribution
- **WHEN** the PiP window is resized while a compact metadata header is visible
- **THEN** the PiP metadata presentation SHALL re-evaluate the title’s rendered line state for the new width
- **AND** it SHALL switch between the compact single-line and two-line presentation variants as needed
- **AND** the uploader/stats row SHALL continue applying the stats-first width priority contract
- **AND** it SHALL NOT require a text-length heuristic to decide the active variant

#### Scenario: PiP rebalance does not alter shared playback synchronization
- **WHEN** the PiP compact header switches presentation variants or compresses the uploader/stats row
- **THEN** the active playback item, authoritative metadata ownership, and main-window playback synchronization SHALL remain unchanged
- **AND** the rebalance SHALL remain a PiP-only presentation concern
