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
