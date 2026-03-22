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
