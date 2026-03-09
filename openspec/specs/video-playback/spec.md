## MODIFIED Requirements

### Requirement: User can control playback via Niconico embed player

The system SHALL use Niconico's embed player (iframe) for video playback, controlled via postMessage API.

#### Scenario: Load video player
- **WHEN** user selects a video to play
- **THEN** system loads iframe with URL `https://embed.nicovideo.jp/watch/{videoId}?jsapi=1&playerId=1`
- **AND** player is displayed in the player area (main window or PiP window)

#### Scenario: Play/pause control
- **WHEN** user clicks play/pause button
- **THEN** system sends postMessage to iframe with `eventName: "play"` or `"pause"`

#### Scenario: Detect video end
- **WHEN** video playback ends (playerStatus === 4)
- **THEN** system receives playerStatusChange event from iframe
- **AND** if auto-play is enabled, next video loads automatically

#### Scenario: Auto-play starts playback after next-video transition
- **WHEN** the active playlist advances to a next video because of playback end, manual next navigation, or auto-skip
- **AND** auto-play is enabled
- **THEN** the newly loaded embedded player starts playback after its load-complete event

#### Scenario: Auto-play disabled leaves next video paused
- **WHEN** the active playlist advances to a next video because of playback end, manual next navigation, or auto-skip
- **AND** auto-play is disabled
- **THEN** the newly loaded embedded player remains loaded but does not automatically start playback

#### Scenario: Auto-skip sponsor segment
- **WHEN** video is near end (remaining time <= skip threshold)
- **AND** auto-skip is enabled
- **THEN** system automatically loads next video

## ADDED Requirements

### Requirement: Playback works in PiP window

The system SHALL support video playback in the PiP window with same controls as main window.

#### Scenario: Play in PiP window
- **WHEN** PiP window is open
- **THEN** video plays in PiP window iframe
- **AND** playback controls work identically to main window

#### Scenario: PiP window receives player events
- **WHEN** video status changes in PiP window
- **THEN** PiP window handles the event locally
- **AND** notifies Rust backend of state changes

#### Scenario: PiP playback remains functional under desktop CSP baseline
- **WHEN** the desktop app enforces its baseline WebView Content Security Policy
- **AND** a user opens playback in the PiP window
- **THEN** the embedded player iframe loads successfully in the PiP window
- **AND** PiP playback behavior remains functional
- **AND** PiP autoplay continues to use the embedded player command target reliably after load-complete events

### Requirement: Watch history tracked locally

The system SHALL track watched videos for the single local user without authentication.

#### Scenario: Mark video as watched
- **WHEN** video starts playing
- **THEN** video ID is added to `watched` table with timestamp

#### Scenario: Display watched status
- **WHEN** video has been watched before
- **THEN** video item in list shows watched indicator

## ADDED Requirements (fix-frontend-alignment change)

### Requirement: playNext has cooldown protection

The system SHALL prevent rapid repeated calls to playNext function using a time-based cooldown.

#### Scenario: Cooldown prevents duplicate calls
- **WHEN** playNext is called
- **THEN** system records the timestamp of the call
- **AND** subsequent calls within 1 second are ignored
- **AND** only the first call executes

#### Scenario: Multiple events trigger playNext
- **WHEN** video ends (playerStatus === 4)
- **AND** autoSkip triggers near end (remaining <= threshold)
- **THEN** only one playNext executes
- **AND** no duplicate video skipping occurs

### Requirement: User can expand and collapse long descriptions

The system SHALL allow users to expand and collapse video descriptions that exceed a certain length.

#### Scenario: Long description is collapsed by default
- **WHEN** video description is longer than 200 characters
- **THEN** description is truncated by default
- **AND** "展開" (expand) button is displayed

#### Scenario: User expands description
- **WHEN** user clicks expand button
- **THEN** full description is displayed
- **AND** button text changes to "收起" (collapse)

#### Scenario: User collapses description
- **WHEN** user clicks collapse button
- **THEN** description is truncated again
- **AND** button text changes back to "展開"

### Requirement: User can configure auto-skip threshold

The system SHALL allow users to configure the time threshold for auto-skipping sponsor segments.

#### Scenario: Default skip threshold
- **WHEN** user has not configured skip threshold
- **THEN** default threshold is 3 seconds

#### Scenario: User adjusts skip threshold
- **WHEN** user changes skip threshold in settings
- **THEN** new threshold is saved to localStorage
- **AND** threshold persists across sessions

#### Scenario: Skip threshold options
- **WHEN** user views skip threshold options
- **THEN** available options are: 0, 1, 2, 3, 4, 5, 10 seconds

### Requirement: Playback behavior remains stable through player-controller refactors
The system SHALL preserve the existing embedded-player control behavior when player logic is moved behind reusable frontend abstractions.

#### Scenario: Main player behavior survives abstraction
- **WHEN** the main window player control logic is refactored into reusable modules
- **THEN** play/pause, next/previous navigation, watched-state updates, and playback-end handling continue to behave the same as before the refactor

#### Scenario: Shared player abstraction preserves iframe protocol
- **WHEN** a reusable player controller is introduced
- **THEN** it still communicates with the Niconico embed player through the same postMessage protocol expected by the current playback flow
- **AND** existing auto-play and auto-skip behavior continues to work in both main and PiP playback contexts
