## ADDED Requirements

### Requirement: Unified player component with mode support

The system SHALL provide a UnifiedPlayer component that supports two modes:
- `full`: Complete player with all controls and settings panel (main window)
- `compact`: Compact player with sidebar controls (PIP window)

#### Scenario: Full mode renders complete layout
- **WHEN** UnifiedPlayer is rendered with mode="full"
- **THEN** it SHALL display a persistent header shell for metadata presentation whenever `currentVideo` exists
- **AND** it SHALL display the video iframe
- **AND** it SHALL display playback controls with settings panel
- **AND** it SHALL display PIP button
- **AND** it SHALL display a persistent details shell for metadata presentation whenever `currentVideo` exists

#### Scenario: Compact mode renders compact layout
- **WHEN** UnifiedPlayer is rendered with mode="compact"
- **THEN** it SHALL display a sidebar with control buttons
- **AND** it SHALL display a persistent header shell for metadata presentation whenever `currentVideo` exists
- **AND** it SHALL display the video iframe
- **AND** it SHALL display a persistent details shell for metadata presentation whenever `currentVideo` exists

#### Scenario: Stable shells during staged metadata enrichment
- **WHEN** `currentVideo` exists
- **AND** enriched playback metadata for that selected playback identity is not yet ready
- **THEN** UnifiedPlayer SHALL keep the header and details shells mounted
- **AND** the shells SHALL reserve layout structure without requiring skeleton placeholders
- **AND** later metadata enrichment SHALL populate those existing shells instead of mounting new metadata regions

#### Scenario: Empty state when no video
- **WHEN** currentVideo is null
- **THEN** an empty state placeholder SHALL be displayed
- **AND** the iframe SHALL NOT be rendered
- **AND** the player SHALL NOT display always-visible empty metadata shells in place of the existing no-selection behavior

### Requirement: Shared metadata presentation varies by player mode through a declarative contract
The system SHALL define main-window and PiP metadata presentation differences through a shared, testable layout contract rather than ad hoc template splits.

#### Scenario: Main window metadata contract remains unchanged
- **WHEN** UnifiedPlayer renders metadata in `full` mode
- **THEN** the shared contract SHALL define title clamp as 1 line
- **AND** uploader-name clamp as 1 line
- **AND** a larger date/stats emphasis token that preserves the fixed meta-row height contract
- **AND** it SHALL NOT expose PiP-only compact header line-state variants in the main-window surface

#### Scenario: PiP metadata contract includes two compact header variants
- **WHEN** UnifiedPlayer renders metadata in `compact` mode
- **THEN** the shared contract SHALL define title clamp as up to 2 lines
- **AND** uploader-name clamp as 1 line
- **AND** a reduced avatar size
- **AND** explicit stat spacing equivalent to `▶ 20 ❤️ 0 📝 0 💬 0`
- **AND** a stats-first inline width-priority rule that keeps the stats cluster unwrapped before expanding uploader width
- **AND** it SHALL define distinct compact header presentation variants for rendered `single-line` and `two-line` title states while preserving the same outer header shell height

#### Scenario: Compact contract selection follows rendered title line state
- **WHEN** UnifiedPlayer renders metadata in `compact` mode
- **AND** the compact header title fits within a single rendered line at the current PiP width
- **THEN** the shared metadata panel SHALL apply the compact `single-line` header presentation contract
- **AND** the title/date row and uploader/stats row SHALL rebalance vertically without changing the outer compact header shell height

#### Scenario: Compact contract prioritizes stats completeness over uploader width
- **WHEN** UnifiedPlayer renders metadata in `compact` mode
- **AND** uploader identity and stats compete for horizontal space
- **THEN** the shared metadata panel SHALL preserve the four stats as a single inline group
- **AND** the uploader name SHALL become the first truncation target
- **AND** stable contract markers or classes SHALL make that priority observable to tests

#### Scenario: Compact contract returns to two-line layout when width changes
- **WHEN** UnifiedPlayer renders metadata in `compact` mode
- **AND** PiP window resizing causes the compact header title to occupy two rendered lines
- **THEN** the shared metadata panel SHALL apply the compact `two-line` header presentation contract
- **AND** the compact header SHALL continue to preserve the outer shell height used before the resize

#### Scenario: Shared details presentation rules
- **WHEN** UnifiedPlayer renders metadata details in either `full` or `compact` mode
- **THEN** the shared contract SHALL define URL treatment for that mode
- **AND** a divider between tags and description SHALL appear only when both content regions exist
- **AND** the contract SHALL be observable through stable DOM markers or classes so the behavior is testable

### Requirement: Player controls sub-component

The system SHALL provide a PlayerControls component that can render in horizontal or vertical layout.

#### Scenario: Horizontal layout for full mode
- **WHEN** PlayerControls is rendered with layout="horizontal"
- **THEN** controls SHALL be arranged horizontally
- **AND** settings panel SHALL be displayed below when open

#### Scenario: Vertical layout for compact mode
- **WHEN** PlayerControls is rendered with layout="vertical"
- **THEN** controls SHALL be arranged vertically
- **AND** settings panel SHALL NOT be displayed

#### Scenario: Control buttons emit correct events
- **WHEN** play/pause button is clicked
- **THEN** togglePlayPause SHALL be called
- **WHEN** next button is clicked
- **THEN** playNext event SHALL be emitted
- **WHEN** previous button is clicked
- **THEN** playPrevious event SHALL be emitted

### Requirement: Playback settings integration

The system SHALL integrate playback settings (autoPlay, autoSkip) in the unified player.

#### Scenario: Settings panel toggles visibility
- **WHEN** settings button is clicked in full mode
- **THEN** the settings panel SHALL toggle visibility

#### Scenario: Settings changes persist
- **WHEN** autoPlay or autoSkip is changed
- **THEN** the setting SHALL be persisted via API
- **AND** the playback-settings-changed event SHALL be handled

### Requirement: PIP button functionality

The system SHALL provide PIP open/close functionality in full mode.

#### Scenario: PIP button opens PIP window
- **WHEN** PIP button is clicked and PIP is not active
- **THEN** openPip event SHALL be emitted

#### Scenario: PIP placeholder when PIP is active
- **WHEN** pipActive prop is true
- **THEN** a PIP placeholder SHALL be displayed
- **AND** a button to return to main window SHALL be shown

### Requirement: Event propagation to parent

The system SHALL propagate relevant events to parent components.

#### Scenario: Video watched event propagates
- **WHEN** a video is marked as watched
- **THEN** videoWatched event SHALL be emitted with the video

#### Scenario: State cleared event propagates
- **WHEN** player state is cleared due to active-playback-cleared
- **THEN** stateCleared event SHALL be emitted

### Requirement: Description toggle reflects actual metadata truncation
The system SHALL render the description expand/collapse control in a shared player metadata panel whenever the collapsed description content is visually truncated in that rendered panel instance.

#### Scenario: Toggle appears when collapsed description overflows
- **WHEN** `VideoMetaPanel` renders a video description in collapsed mode
- **AND** the rendered description content exceeds the collapsed visual clamp for that panel instance
- **THEN** the system displays an expand control for that description
- **AND** the user can reveal the full description without changing playback state

#### Scenario: Toggle stays hidden when collapsed description fits
- **WHEN** `VideoMetaPanel` renders a video description in collapsed mode
- **AND** the rendered description content fits fully within the collapsed visual clamp
- **THEN** the system does NOT display an expand/collapse control

#### Scenario: Sanitized line breaks still preserve access to full content
- **WHEN** a description contains preserved line breaks or allowed sanitized markup that increases rendered height
- **AND** the collapsed rendering becomes visually truncated even though the raw description text does not cross a fixed character threshold
- **THEN** the system still displays the expand control
- **AND** the full sanitized description remains accessible after expansion

### Requirement: Description toggle behavior stays consistent across shared player surfaces
The system SHALL apply the same truncation-driven description toggle contract in each player surface that renders the shared metadata panel.

#### Scenario: Main window metadata panel uses truncation-driven toggle logic
- **WHEN** `UnifiedPlayer` renders `VideoMetaPanel` in the main window layout
- **THEN** the description toggle visibility is determined from the rendered overflow state of that main-window panel instance

#### Scenario: PiP metadata panel uses truncation-driven toggle logic
- **WHEN** `UnifiedPlayer` renders `VideoMetaPanel` in the PiP layout
- **THEN** the description toggle visibility is determined from the rendered overflow state of that PiP panel instance
- **AND** PiP users retain access to the full description whenever the collapsed description is truncated
