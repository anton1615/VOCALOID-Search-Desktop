## MODIFIED Requirements

### Requirement: User can enable automatic skip of video endings

The system SHALL allow users to automatically skip to the next video when the current video is near its end, useful for bypassing sponsor segments or credits.

#### Scenario: Auto-skip is disabled by default
- **WHEN** user first uses the application
- **THEN** auto-skip is disabled
- **AND** user must manually enable it in playback settings

#### Scenario: User enables auto-skip
- **WHEN** user toggles auto-skip setting to enabled
- **THEN** auto-skip becomes active
- **AND** setting is saved through the existing playback settings flow

#### Scenario: Auto-skip triggers near end without requiring auto-play
- **WHEN** auto-skip is enabled
- **AND** video has less than or equal to skip threshold seconds remaining
- **AND** video has played for more than 10 seconds
- **THEN** system automatically loads next video
- **AND** whether the next video starts playing is determined independently by the auto-play setting

#### Scenario: Auto-skip respects skip threshold
- **WHEN** the internal skip threshold is N seconds
- **AND** remaining time is N seconds or less
- **THEN** auto-skip triggers

### Requirement: Auto-skip settings are persisted

The system SHALL persist auto-skip settings across application sessions.

#### Scenario: Settings persist after restart
- **WHEN** user closes application
- **AND** user reopens application
- **THEN** auto-skip setting is restored from the existing playback settings storage
- **AND** the internal skip threshold remains available to the playback logic

### Requirement: Auto-skip indicator visible in UI

The system SHALL expose playback settings from the main-window player through a collapsed settings panel instead of an always-visible standalone auto-skip control.

#### Scenario: Gear button reveals playback toggles in main window
- **WHEN** user is viewing the main-window player controls
- **THEN** playback settings are collapsed by default behind a gear-triggered panel
- **AND** opening the panel reveals independent auto-play and auto-skip toggles

#### Scenario: Skip threshold control is not shown to the user
- **WHEN** user opens the playback settings panel
- **THEN** no skip-threshold selector or numeric threshold input is shown
- **AND** existing auto-skip timing behavior is preserved internally
