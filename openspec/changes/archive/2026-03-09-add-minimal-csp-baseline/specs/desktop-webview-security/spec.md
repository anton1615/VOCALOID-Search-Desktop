## ADDED Requirements

### Requirement: Desktop WebView enforces a baseline Content Security Policy
The desktop application SHALL configure a non-null Content Security Policy for its Tauri WebView instead of leaving CSP disabled.

#### Scenario: Desktop app starts with a configured CSP
- **WHEN** the desktop app loads its Tauri WebView configuration
- **THEN** the WebView uses an explicit CSP baseline
- **AND** the CSP value is not `null`

### Requirement: Baseline CSP remains compatible with current development workflow
The desktop application SHALL keep the current Tauri development workflow functional under the configured CSP baseline.

#### Scenario: Development app loads with CSP enabled
- **WHEN** a developer starts the desktop app against the local frontend dev server
- **THEN** the app shell loads successfully
- **AND** the current frontend UI remains usable without requiring manual CSP changes for routine development

### Requirement: Baseline CSP remains compatible with packaged desktop playback flows
The desktop application SHALL keep the current packaged playback experience functional under the configured CSP baseline.

#### Scenario: Embedded player loads in packaged app
- **WHEN** a user opens the desktop app and selects a video to play in the main window or PiP window
- **THEN** the embedded NicoNico player loads successfully under the configured CSP baseline
- **AND** the existing playback controls remain functional

### Requirement: Deferred hardening concerns are documented explicitly
The desktop application SHALL document security hardening concerns that are intentionally excluded from this change so future hardening work remains discoverable.

#### Scenario: Deferred shell hardening is recorded
- **WHEN** this CSP baseline change is documented in the spec set
- **THEN** the documentation states that `shell:allow-open` remains unchanged in this change
- **AND** the shell permission review is identified as future hardening work rather than part of the current implementation
