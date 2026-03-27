## ADDED Requirements

### Requirement: Cross-list selection creates a distinct playback session
The system SHALL treat explicit selection of the same `video.id` from a different list context as a distinct playback session. This requirement applies to explicit cross-list selection among `Search`, `History`, and `WatchLater`. The active playback identity MUST remain bound to the selected list context, including playlist type, playlist version, and item index, instead of being collapsed by content id alone.

#### Scenario: Selecting the same video from another list rebinds playback ownership
- **WHEN** a video is currently active from one list and the user explicitly selects the same `video.id` from a different list
- **THEN** the active playback identity SHALL be rebound to the newly selected list context
- **THEN** subsequent playback state reads SHALL report the new list context as the owner of playback

#### Scenario: Next and previous navigation follow the newly selected list
- **WHEN** the user explicitly selects a same-id video from a different list and then invokes next or previous playback navigation
- **THEN** navigation SHALL use the item ordering of the newly selected list context
- **THEN** navigation SHALL NOT continue using the previously active list context

### Requirement: Cross-list same-video selection resets the player session
When explicit selection changes playback identity across lists, the player lifecycle SHALL start a new player session even if the selected `video.id` is unchanged. The system MUST NOT leave the prior embedded player session running as if playback had continued unchanged.

#### Scenario: Same-id cross-list selection forces a new player transition
- **WHEN** the user explicitly selects the same `video.id` from a different list context
- **THEN** the player shell and embedded playback instance SHALL transition as a new session
- **THEN** the user-visible playback experience SHALL be equivalent to selecting a different playable item

#### Scenario: Cross-list selection does not silently reuse the previous media session
- **WHEN** playback is active for a video in one list and the same `video.id` is selected from another list
- **THEN** the system SHALL NOT preserve the old media session in a way that obscures the new list binding
- **THEN** any player readiness or loading state SHALL correspond to the newly selected playback session

#### Scenario: A new session boundary is observable after rebinding
- **WHEN** explicit same-id cross-list selection rebinds active playback to a different list context
- **THEN** the frontend SHALL create a new player-session boundary for the newly active playback identity
- **THEN** local player shell state SHALL return to a pre-ready state before the new playback session becomes ready

### Requirement: Reentry and synchronization use the latest list-bound playback identity
Any playback reconstruction path, including PiP ownership return, active playback metadata reentry, and cross-window playback synchronization, SHALL use the latest active playback identity from Rust and MUST preserve the list context of the most recent explicit selection.

#### Scenario: PiP ownership return restores the latest list-bound session
- **WHEN** playback ownership returns from PiP to the main window after the user most recently selected a same-id video from another list
- **THEN** the reconstructed playback state SHALL use the latest active playback list context
- **THEN** the reconstructed session SHALL NOT revert to the previously active list merely because the video id matches

#### Scenario: Metadata updates apply only to the active list-bound session
- **WHEN** playback metadata is refreshed after a same-id cross-list selection
- **THEN** metadata updates SHALL apply only when the payload matches the active playback identity, including list context
- **THEN** metadata updates from the superseded list context SHALL be ignored

#### Scenario: Reentry reconstructs from the latest active playback identity
- **WHEN** playback is reconstructed in either main window or PiP after same-id cross-list rebinding
- **THEN** reconstruction SHALL derive from the latest Rust active playback identity
- **THEN** reconstruction SHALL NOT continue from stale local iframe or shell state that belonged to the superseded list context
