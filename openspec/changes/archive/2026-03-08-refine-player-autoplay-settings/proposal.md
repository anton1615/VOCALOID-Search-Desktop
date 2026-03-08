## Why

The main-window player currently exposes auto-skip as a standalone checkbox while the actual playback model also includes auto-play. This makes the UI hard to understand and obscures a real behavior issue where advancing to the next video does not always lead to autoplay even when autoplay is enabled.

## What Changes

- Simplify the main-window playback settings UI by replacing the always-visible auto-skip control with a collapsed gear-triggered settings panel.
- Expose two independent playback toggles in that panel: auto-play and auto-skip.
- Keep the settings panel collapsed by default and do not persist its open/closed UI state.
- Remove the skip-threshold control from the user-facing UI while preserving the existing internal auto-skip timing logic.
- Make auto-skip and auto-play independent behaviors so auto-skip may advance to the next video without forcing the next video to start playing.
- Tighten autoplay behavior so when auto-play is enabled, next-video transitions still begin playback after the new embedded player finishes loading.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `video-playback`: Playback transitions and player controls must preserve reliable autoplay after next-video navigation while keeping auto-play semantics independent from auto-skip.
- `auto-skip`: The playback settings UI and requirement semantics must change so auto-skip remains configurable without exposing skip-threshold controls and without requiring auto-play to be enabled.

## Impact

- Affected frontend code in the main player column and embedded-player control flow.
- Affected playback settings presentation and interaction behavior in the main window.
- Affected OpenSpec requirements for `video-playback` and `auto-skip`.
- No backend API expansion is expected; the change should reuse existing playback settings storage and synchronization behavior.
