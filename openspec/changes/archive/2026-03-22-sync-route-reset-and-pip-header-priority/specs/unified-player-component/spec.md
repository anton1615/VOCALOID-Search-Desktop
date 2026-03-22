## MODIFIED Requirements

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
