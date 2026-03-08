## 1. Playback settings UI

- [x] 1.1 Update the main-window player controls to replace the always-visible auto-skip control with a gear-triggered playback settings panel.
- [x] 1.2 Add independent auto-play and auto-skip toggles to the panel, keep it collapsed by default, and avoid persisting the panel open/closed state.
- [x] 1.3 Remove the user-facing skip-threshold input while preserving the existing internal skip-threshold value and playback-settings wiring.

## 2. Autoplay transition behavior

- [x] 2.1 Add or update focused tests that cover next-video transitions with auto-play enabled versus disabled, including transitions triggered by auto-skip.
- [x] 2.2 Adjust the embedded-player control flow so newly loaded videos reliably begin playback after next-video transitions when auto-play is enabled.
- [x] 2.3 Verify that auto-skip can still advance to the next video while leaving the next video paused when auto-play is disabled.

## 3. Verification

- [x] 3.1 Run the focused frontend test suite covering the embedded player controller and any touched player-control UI logic.
- [x] 3.2 Manually verify in the main window that the gear panel defaults to collapsed, reveals the two toggles, hides threshold controls, preserves the intended auto-play/auto-skip behavior, and restores the toggle values after restart.
