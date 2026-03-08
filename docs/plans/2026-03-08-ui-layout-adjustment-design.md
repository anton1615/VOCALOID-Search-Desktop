# UI Layout Adjustment Design

## Scope

Adjust the main player column and PiP information layout without changing playback behavior, playlist synchronization, or PiP close behavior.

## Approved Layout

### Main window right column

From top to bottom:
1. Title / uploader information
2. Embedded player
3. Playback controls (play, previous, next, watch later, PiP)
4. Tags
5. Description

### PiP window

- Keep the left-side playback controls exactly where they are now
- The right-side content order becomes:
  1. Title / uploader information
  2. Embedded player
  3. Tags
  4. Description

## Component Strategy

Use the existing shared `VideoMetaPanel` instead of duplicating template blocks back into `PlayerColumn.vue` and `PipApp.vue`.

Add a minimal display mode so `VideoMetaPanel` can render:
- header-only
- details-only
- full

This allows:
- main window: header-only → iframe → playback controls → details-only
- PiP: header-only → iframe → details-only

## Styling Change

The description expand/collapse button should be changed to a full-width bar style in both main window and PiP.

## Non-Goals

- No playback logic changes
- No playlist-state changes
- No PiP synchronization changes
- No player controller changes
- No metadata fetch behavior changes

## Verification

After the UI change:
- Build must pass
- Existing unit tests must remain green
- Main window layout order must match the approved sequence
- PiP layout order must match the approved sequence
- Expand/collapse button must appear as a bar-style button in both places
