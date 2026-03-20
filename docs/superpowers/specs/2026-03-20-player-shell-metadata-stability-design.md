# Player shell metadata stability design

Date: 2026-03-20
Status: approved for planning
Scope: Main window + PiP UI shell stability, shared staged metadata enrichment, and PiP compact presentation fixes.

## Context

The shared player already uses a staged playback flow: show the embedded player first, then let Rust fetch enriched metadata and update the UI once the authoritative payload is ready. That staged behavior is desirable and should remain the single mode for entering the player UI.

The current problems are:
1. `UnifiedPlayer.vue` still ties metadata content rendering directly to `metadataReady`, so shell regions can stay visually incomplete in some regain paths.
2. The PiP-close return path does not fully re-enter the same staged enrichment flow used when selecting a video from a list.
3. PiP metadata styling is still effectively using the main-window defaults because compact presentation rules were defined but not fully wired through the shared component boundary.
4. The first fixed-shell implementation used outer shell height reservation that can create extra blank space above the PiP player, which is not the approved behavior.

Relevant code paths observed during exploration:
- `vocaloid-search-desktop/src/components/UnifiedPlayer.vue`
- `vocaloid-search-desktop/src/components/VideoMetaPanel.vue`
- `vocaloid-search-desktop/src/composables/usePlayerCore.ts`
- `vocaloid-search-desktop/src/composables/useAuthoritativePlaybackSync.ts`
- `vocaloid-search-desktop/src/App.vue`
- `vocaloid-search-desktop/src/PipApp.vue`

## Goals

1. Keep the current staged mode where the embedded player appears before enriched metadata is ready.
2. Make every entry back into the main player UI reuse that same staged enrichment mode instead of inventing a separate restore path.
3. Keep header/details shells structurally stable so the UI does not jump while metadata is pending.
4. Fix the PiP-close return path so it behaves like selecting a video again: player first, Rust enrichment second, metadata UI update last.
5. Make PiP actually use the compact presentation contract: two-line title, smaller avatar, wider stat spacing, and no extra blank gap between metadata row and player.
6. Keep the unified shared-player architecture rather than splitting main and PiP into separate templates.

## Non-goals

- No redesign of the overall player architecture.
- No skeleton loading UI.
- No special snapshot-restore metadata path for PiP regain.
- No backend protocol redesign or Rust event bus rewrite.
- No unrelated refactors outside the shared player / metadata presentation path.

## Decisions

### 1. Single staged enrichment mode for all player entry paths

The system will use one staged metadata mode everywhere the main shared player is entered:
- show the iframe/player immediately,
- keep stable shell regions present,
- let Rust fetch enriched metadata,
- update metadata UI after the enriched payload returns.

This applies both to normal list selection and to regaining the main player after PiP closes.

Why:
- The user explicitly wants one consistent behavior model.
- Separate regain restore logic would create a second state path that is harder to reason about and easier to regress.
- The desired UX is “same as clicking from the list,” not “special recovery mode.”

### 2. Stable shell strategy remains, but shell reservation moves inward for PiP

Both main window and PiP continue using a stable three-part shell:

```text
┌ Header shell  ┐
├ Player        ┤
└ Details shell ┘
```

Rules:
- `header` and `details` shells remain mounted whenever `currentVideo` exists and the playback UI is present.
- When metadata is not ready, the shells remain present without skeleton content.
- When `currentVideo` is `null`, the existing empty-state behavior remains unchanged.
- The player continues to render as early as possible.

However, for PiP, the approved “fixed frame” behavior is not outer-shell min-height padding. The fixed frame should come from the internal title/meta-row layout, not from extra external header height.

Why:
- The original shell persistence idea solves layout jumping.
- But the first implementation used outer shell height reservation that incorrectly pushed the PiP player downward.
- The approved PiP behavior is: title can reserve two lines, metadata row stays stable, but the player should sit directly under the uploader/stats row without an extra blank band.

### 3. PiP-close regain is an entry back into staged enrichment, not snapshot restore

When PiP closes, the main window should not try to immediately restore full metadata from the existing snapshot. Instead, PiP-close becomes a regain trigger that sends the main window back into the same staged enrichment pipeline used by list selection.

Desired visible order:
1. PiP closes.
2. Main window regains the shared player surface.
3. The iframe/player is visible first.
4. Header/details shells are already present.
5. Rust fetches enriched metadata again.
6. Metadata UI fills in after the enriched payload arrives.

Why:
- This matches the user-approved “single mode” rule.
- It avoids a second restore-specific metadata path.
- It keeps the mental model simple: every time the main player is entered, it behaves the same way.

### 4. PiP must consume the compact presentation contract for real

The compact presentation contract is not just a helper definition; it must be wired end-to-end from `UnifiedPlayer.vue` into `VideoMetaPanel.vue`.

Compact-mode rules:
- Title is clamped to exactly 2 lines with ellipsis.
- Uploader nickname is clamped to exactly 1 line with ellipsis.
- Uploader avatar is one visual step smaller than main window.
- Stats gain explicit extra spacing so the line reads like `▶ 20 ❤️ 0 📝 0 💬 0`.
- URL treatment stays aligned with the shared surface style.
- Divider between tags and description appears only when both exist.

Why:
- The current implementation defined compact rules but still fell back to `presentationMode: 'full'` defaults in actual PiP rendering.
- The user-visible regression proves the contract must be treated as a real interface, not just a helper placeholder.

### 5. Main-window and PiP fixed-frame behavior should be implemented at content-layout level

Main window rules:
- Title clamp: 1 line.
- Uploader clamp: 1 line.
- Date/stats can use an emphasized token.
- Meta-row height remains fixed and should not expand unpredictably.

PiP rules:
- Title clamp: 2 lines.
- Uploader clamp: 1 line.
- Player sits directly below the uploader/stats row.
- No extra external gap above the player.

This means fixed visual stability should be expressed through line clamps, row-height rules, avatar sizing, and spacing contracts inside `VideoMetaPanel.vue`, not through a generic outer shell min-height that applies equally to all modes.

## Architecture impact

### `UnifiedPlayer.vue`

Responsibilities after this revision:
- Keep persistent header/details shell containers whenever `currentVideo` exists.
- Continue letting the player iframe appear before metadata is ready.
- Pass real `presentationMode` values into `VideoMetaPanel.vue` (`full` for main window, `compact` for PiP).
- Ensure PiP regain sends the main player back through the same staged enrichment path rather than special-case snapshot restoration.
- Avoid using shell sizing rules that add extra blank space above the PiP player.

### `VideoMetaPanel.vue`

Responsibilities after this revision:
- Continue acting as the shared metadata component for both main window and PiP.
- Consume the shared presentation contract declaratively.
- Express fixed-frame behavior through internal content layout (title clamp, uploader clamp, avatar size, stat spacing, meta-row height), not through an outer reserved gap.
- Keep tags/description divider rules shared.
- Stay separate from shell ownership: metadata formatting belongs here, shell lifecycle does not.

### Player regain / enrichment path

Responsibilities after this revision:
- Regaining the main player after PiP close should re-enter the same staged enrichment path as list selection.
- The main window should not treat PiP-close as a full-metadata restore event.
- Any readiness state used by the shared player must allow the UI to show player-first and metadata-later in both list selection and PiP regain paths.

Likely touchpoints:
- `src/App.vue`
- `src/components/UnifiedPlayer.vue`
- `src/composables/usePlayerCore.ts`
- shared metadata layout helpers

## Data-flow summary

### Old regain idea (rejected)

```text
PiP closes
→ main window refreshes ownership state
→ main window restores metadata directly from snapshot
→ UI returns complete in one step
```

Rejected because the user explicitly wants the same staged mode everywhere, not a special restore path.

### Approved regain path

```text
PiP closes
→ main window re-enters the shared player path
→ player iframe appears first
→ stable header/details shells are present
→ Rust fetches enriched metadata again
→ metadataReady transitions when enrichment returns
→ title/uploader/tags/description fill into existing shells
```

## Testing strategy

### 1. Unified player / shell stability tests

Validate that:
- Header and details shells remain present before metadata is ready when `currentVideo` exists.
- No-selection mode still uses the existing empty-state behavior.
- Player can still appear first.
- PiP shell sizing does not create an extra artificial gap between the metadata row and the player.

### 2. Compact presentation tests

Validate that:
- PiP really receives compact presentation mode.
- PiP title is clamped to 2 lines.
- PiP uploader name is clamped to 1 line.
- PiP avatar is smaller than main window.
- PiP stats apply the approved wider spacing treatment.
- PiP fixed frame comes from internal content layout rather than a generic outer header gap.
- Divider only appears when both tags and description exist.

### 3. PiP-close staged regain tests

Validate that:
- PiP close sends the main window back into the same staged enrichment flow used by list selection.
- The main window does not use a direct metadata restore path.
- The player is visible before enriched metadata is available.
- Metadata UI updates when Rust enrichment finishes.
- Closing PiP no longer leaves the main window stuck at “iframe only” after regain.

## Tradeoff summary

Chosen approach: one staged enrichment mode plus true compact-mode contract wiring.

Why this approach:
- Matches the user’s explicit preference for a single consistent mode.
- Solves the PiP visual regressions at the contract boundary instead of piling on more CSS overrides.
- Keeps the player architecture unified.
- Avoids a second regain-only metadata restore path.

Alternatives rejected:
- Direct metadata restore on PiP regain: simpler in isolation, but violates the approved single-mode rule.
- Outer shell min-height reservation for PiP: stabilizes space, but creates the exact unwanted blank gap above the player.

## Implementation planning notes

The implementation plan should stay scoped to:
1. Re-aligning PiP regain with the existing staged enrichment pipeline.
2. Wiring `presentationMode="compact"` end-to-end.
3. Reworking PiP fixed-frame behavior so internal text layout is stable without adding an outer blank band.
4. Regression coverage for staged regain and PiP compact presentation.

## Review status

This document supersedes the earlier regain design that treated PiP close as a placeholder-held snapshot restore path. The approved revision now reflects:
- one staged enrichment mode for all player entry paths,
- PiP-close regain behaving like list selection rather than direct restore,
- true compact-mode contract wiring,
- fixed-frame stability implemented at content-layout level instead of outer shell padding.
