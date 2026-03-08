## Context

The current main-window player settings UI exposes auto-skip as a standalone control even though the playback model already includes both auto-play and auto-skip. In practice, this creates two problems: users cannot easily discover or reason about the difference between the two behaviors, and autoplay failures after next-video transitions are difficult to diagnose because the UI does not reflect the real control model.

The existing frontend already stores and synchronizes playback settings through the same Rust-backed playback settings flow used by other windows. The change should therefore reuse the current settings state and player controller instead of introducing a new settings subsystem. PiP is intentionally out of scope for the gear-triggered settings UI; only the main window should gain the collapsed settings affordance.

## Goals / Non-Goals

**Goals:**
- Present main-window playback settings behind a gear-triggered panel that is collapsed by default.
- Show two independent toggles in that panel: auto-play and auto-skip.
- Preserve existing playback-settings persistence and cross-window synchronization.
- Remove the skip-threshold control from the user-facing UI without changing the internal auto-skip trigger logic.
- Make autoplay behavior reliable after next-video transitions when auto-play is enabled.
- Preserve the valid independent state where auto-skip advances to the next video but auto-play remains disabled.

**Non-Goals:**
- Do not add new backend commands or a new playback settings storage model.
- Do not add new threshold customization UI or alter the existing threshold calculation logic.
- Do not add a matching gear/settings UI to PiP.
- Do not redesign unrelated playback controls, metadata layout, or playlist navigation behavior.

## Decisions

### Use the existing playback settings model and only change how it is presented
The player already tracks `autoPlay`, `autoSkip`, and `skipThreshold`, and the backend already broadcasts shared playback setting changes. Reusing that model keeps this change focused and avoids introducing a second source of truth.

**Alternatives considered:**
- Add a separate local-only settings UI state model in the component. Rejected because it would duplicate existing playback settings state and risk divergence.
- Move all settings ownership into a new frontend controller. Rejected because the underlying cross-window synchronization already exists and does not need replacement.

### Keep the gear panel UI state local, collapsed by default, and non-persistent
The user explicitly wants the settings panel to default to collapsed and not remember its open/closed state. This keeps the interface compact without turning a temporary UI preference into persistent application state.

**Alternatives considered:**
- Persist panel visibility in localStorage. Rejected because it adds state that the user does not want remembered.
- Leave settings always visible. Rejected because it keeps the same clutter and ambiguity that motivated the change.

### Keep auto-play and auto-skip fully independent in both UI and controller semantics
The new UI should make it obvious that auto-play and auto-skip are separate toggles. The controller and specs should reflect the same model: auto-skip may trigger next-video navigation even when auto-play is disabled, and auto-play only decides whether the newly loaded video starts playing automatically.

**Alternatives considered:**
- Disable auto-skip when auto-play is off. Rejected because the user explicitly wants the settings to remain independent.
- Show warning or helper text coupling the two options. Rejected because the desired behavior is already clear enough once both toggles are visible.

### Preserve the internal skip-threshold logic but remove its UI surface
The current auto-skip behavior already meets the user’s real-world needs, so the UI should stop exposing threshold controls while the player continues using the existing internal value and logic.

**Alternatives considered:**
- Remove threshold support entirely. Rejected because that would change behavior beyond the requested scope.
- Keep the threshold input hidden only when collapsed. Rejected because the user explicitly wants the threshold control gone from the user-facing UI.

### Verify next-video autoplay through focused controller and component behavior tests
The most important regression risk is the transition path from `playNext()` to new-player load completion. Tests should explicitly cover the cases where auto-play is enabled versus disabled, especially when next-video navigation was triggered by auto-skip.

**Alternatives considered:**
- Rely on manual testing only. Rejected because the bug is timing-sensitive and easy to reintroduce.
- Test only the UI toggles. Rejected because the key failure mode is the event chain behind next-video autoplay, not checkbox rendering.

## Risks / Trade-offs

- **Autoplay timing remains iframe-sensitive** → Keep the fix scoped to the existing player controller flow and add regression tests for the load-complete path so timing assumptions are documented.
- **Removing threshold UI hides a previously configurable setting** → Preserve the underlying setting and logic so behavior stays stable even though the customization surface is removed.
- **Main window and PiP settings UI will diverge visually** → Accept this intentionally because only the main window is meant to host the gear-triggered settings panel.
- **Legacy prop naming such as `showAutoSkip` may become misleading** → Update interface naming during implementation if needed so the component contract matches the broader playback-settings UI.

## Migration Plan

1. Update the relevant OpenSpec delta specs for `video-playback` and `auto-skip`.
2. Adjust the main-window player controls to use a gear-triggered settings panel with the two toggles.
3. Remove the threshold input from the user-facing player settings UI while keeping the internal threshold value intact.
4. Tighten next-video autoplay handling and add regression tests.
5. Run the focused frontend test suite covering the player controller and main player controls.

## Open Questions

- None at proposal time. The user has already confirmed the UI scope, persistence behavior, and independence rules for auto-play versus auto-skip.
