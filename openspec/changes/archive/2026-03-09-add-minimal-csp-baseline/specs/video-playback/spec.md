## ADDED Requirements

### Requirement: Embedded playback remains compatible with desktop WebView security baseline
The embedded NicoNico playback flow SHALL remain functional when the desktop application enforces its baseline WebView Content Security Policy.

#### Scenario: Main window playback remains functional under CSP baseline
- **WHEN** the desktop app enforces its baseline WebView Content Security Policy
- **AND** a user selects a video in the main window
- **THEN** the embedded player iframe loads successfully
- **AND** playback controls and player event handling continue to work

#### Scenario: PiP playback remains functional under CSP baseline
- **WHEN** the desktop app enforces its baseline WebView Content Security Policy
- **AND** a user opens playback in the PiP window
- **THEN** the embedded player iframe loads successfully in the PiP window
- **AND** PiP playback behavior remains functional
