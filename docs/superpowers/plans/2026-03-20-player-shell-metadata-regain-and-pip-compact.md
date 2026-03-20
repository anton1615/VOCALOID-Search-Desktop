# Player Shell Metadata Regain and PiP Compact Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make PiP close re-enter the same staged metadata enrichment flow as list selection, while fixing PiP compact presentation so the avatar is smaller, stats have wider spacing, the title keeps a true two-line frame, the uploader stays on one line, the tags/description divider still follows shared rules, and the player sits directly below the uploader/stats row.

**Architecture:** Keep the shared `UnifiedPlayer.vue` / `VideoMetaPanel.vue` architecture, but correct two boundaries: regain should re-enter staged enrichment instead of restoring metadata directly, and PiP must receive a real `presentationMode="compact"` contract. Shell persistence stays in `UnifiedPlayer.vue`; content formatting and fixed-frame behavior stay in `VideoMetaPanel.vue`; staged readiness behavior remains centered in `usePlayerCore.ts`; `PipApp.vue` remains the PiP-side event/close source that must keep its close notification contract intact.

**Tech Stack:** Vue 3, TypeScript, Vitest, Vite, Tauri event flow, shared composables

---

## File structure / responsibility map

- **Modify:** `vocaloid-search-desktop/src/features/playlistViews/pipAppCommandTarget.test.ts`
  - Source-contract tests for the PiP close source (`PipApp.vue`) and main-window regain entry-point (`App.vue`).
- **Modify:** `vocaloid-search-desktop/src/features/playlistViews/playerColumnLayout.test.ts`
  - Shared-player source-contract regression tests for shell persistence, player-first rendering, and no stuck iframe-only regain state.
- **Modify:** `vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts`
  - Source-contract tests for compact presentation wiring, uploader clamp, divider rules, and PiP-specific layout semantics.
- **Modify:** `vocaloid-search-desktop/src/components/UnifiedPlayer.vue`
  - Keep shell persistence, pass `presentationMode`, and remove PiP outer-gap behavior.
- **Modify:** `vocaloid-search-desktop/src/components/VideoMetaPanel.vue`
  - Apply compact presentation contract through internal content layout rather than outer shell height.
- **Modify:** `vocaloid-search-desktop/src/composables/usePlayerCore.ts`
  - Adjust staged readiness behavior so PiP regain can re-enter the same player-first → metadata-later flow as list selection.
- **Modify:** `vocaloid-search-desktop/src/composables/usePlayerCore.test.ts`
  - Behavioral tests for staged readiness re-entry and matching enrichment completion.
- **Verify:** `vocaloid-search-desktop/src/composables/usePlayerEvents.test.ts`
  - Regression coverage to ensure metadata update event forwarding still matches the staged contract.
- **Modify:** `vocaloid-search-desktop/src/App.vue`
  - Keep regain entry-point logic aligned with the staged enrichment pipeline rather than snapshot restore.
- **Verify / maybe modify:** `vocaloid-search-desktop/src/PipApp.vue`
  - Preserve PiP close notification semantics as the source side of the regain path.
- **Modify:** `D:/Downloads/vocaloid-search-alt/openspec/changes/stabilize-player-shell-metadata/tasks.md`
  - Update checkbox progress as work completes.

---

### Task 1: Lock the PiP close source and main-window regain entry-point contract

**Files:**
- Modify: `vocaloid-search-desktop/src/features/playlistViews/pipAppCommandTarget.test.ts`
- Verify / maybe modify: `vocaloid-search-desktop/src/PipApp.vue`
- Modify: `vocaloid-search-desktop/src/App.vue`
- Test: `vocaloid-search-desktop/src/features/playlistViews/pipAppCommandTarget.test.ts`

- [ ] **Step 1: Write the failing source-contract test**

Add assertions that:
- `PipApp.vue` still calls `api.notifyPipClosing()` before window close,
- `App.vue` routes `pip-closed` through a single regain handler,
- `App.vue` entry-point code does not introduce direct metadata-restore wording/structure,
- this task does **not** assert readiness ownership — that belongs to `usePlayerCore.ts` in Task 4.

```ts
test('PiP close source notifies backend and main window regains through one staged entry-point', () => {
  const pipAppPath = resolve(__dirname, '../../PipApp.vue')
  const appPath = resolve(__dirname, '../../App.vue')

  const pipAppSource = readFileSync(pipAppPath, 'utf8')
  const appSource = readFileSync(appPath, 'utf8')

  expect(pipAppSource).toContain('await api.notifyPipClosing()')
  expect(appSource).toContain('async function handlePipOwnershipRegained() {')
  expect(appSource).not.toContain('restore metadata directly from snapshot')
})
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd /d/Downloads/vocaloid-search-alt/vocaloid-search-desktop && npx vitest run src/features/playlistViews/pipAppCommandTarget.test.ts -t "PiP close source notifies backend and main window regains through one staged entry-point"
```
Expected: FAIL before the contract is fully aligned.

- [ ] **Step 3: Write minimal implementation**

Update only the regain/source contract code needed:
- preserve `PipApp.vue` close notification flow,
- keep `App.vue` regain entry-point single and staged,
- avoid any direct restore semantics,
- do not move readiness-state ownership into `App.vue`.

- [ ] **Step 4: Run test to verify it passes**

Run the same command from Step 2.
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/features/playlistViews/pipAppCommandTarget.test.ts src/PipApp.vue src/App.vue
git commit -m "fix: align PiP close source with staged regain entry"
```

---

### Task 2: Pass compact presentation mode through UnifiedPlayer

**Files:**
- Modify: `vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts`
- Modify: `vocaloid-search-desktop/src/components/UnifiedPlayer.vue`
- Test: `vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts`

- [ ] **Step 1: Write the failing compact-mode wiring test**

Add assertions that PiP `VideoMetaPanel` invocations receive `presentation-mode="compact"`, while main window invocations receive `presentation-mode="full"`.

```ts
test('UnifiedPlayer passes compact and full presentation modes to VideoMetaPanel', () => {
  const unifiedPlayerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')
  const source = readFileSync(unifiedPlayerPath, 'utf8')

  expect(source).toContain(':presentation-mode="isCompact ? \'compact\' : \'full\'"')
})
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd /d/Downloads/vocaloid-search-alt/vocaloid-search-desktop && npx vitest run src/features/playlistViews/videoMetaPanelLayout.test.ts -t "UnifiedPlayer passes compact and full presentation modes to VideoMetaPanel"
```
Expected: FAIL because `UnifiedPlayer.vue` currently does not pass the prop.

- [ ] **Step 3: Write minimal implementation**

Update each `VideoMetaPanel` usage in `UnifiedPlayer.vue` to pass presentation mode explicitly.

```vue
:presentation-mode="isCompact ? 'compact' : 'full'"
```

- [ ] **Step 4: Run test to verify it passes**

Run the same command from Step 2.
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/features/playlistViews/videoMetaPanelLayout.test.ts src/components/UnifiedPlayer.vue
git commit -m "fix: wire compact presentation mode into shared player"
```

---

### Task 3: Add shell and PiP compact semantic regressions at the shared-player layer

**Files:**
- Modify: `vocaloid-search-desktop/src/features/playlistViews/playerColumnLayout.test.ts`
- Modify: `vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts`
- Modify: `vocaloid-search-desktop/src/components/UnifiedPlayer.vue`
- Modify: `vocaloid-search-desktop/src/components/VideoMetaPanel.vue`
- Test: `vocaloid-search-desktop/src/features/playlistViews/playerColumnLayout.test.ts`
- Test: `vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts`

- [ ] **Step 1: Write the failing shell/presentation semantic tests**

Add tests that explicitly cover:
- persistent header/details shells while metadata is pending,
- player-first rendering,
- no stuck iframe-only regain state at the shared-player source-contract layer,
- compact title two-line contract,
- compact uploader one-line contract,
- compact avatar/sizing semantics,
- wider compact stats spacing semantics,
- divider still requiring both tags and description,
- no PiP-only outer shell reservation that inserts a blank band above the player.

Example shell regression:

```ts
test('shared player keeps persistent shells and player-first rendering during staged regain', () => {
  const playerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')
  const source = readFileSync(playerPath, 'utf8')

  expect(source).toContain('data-shell="header"')
  expect(source).toContain('data-shell="details"')
  expect(source).toContain('player-shell-pending')
})
```

Example compact semantic regression:

```ts
test('PiP compact contract preserves uploader clamp and shared divider rules without adding an outer gap', () => {
  const panelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
  const playerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')

  const panelSource = readFileSync(panelPath, 'utf8')
  const playerSource = readFileSync(playerPath, 'utf8')

  expect(panelSource).toContain(':data-uploader-clamp="presentationContract.uploaderClampLines"')
  expect(panelSource).toContain('presentationContract.showTagDescriptionDivider && visibleTags.length && video.description')
  expect(playerSource).not.toContain('min-height: 86px;')
})
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```bash
cd /d/Downloads/vocaloid-search-alt/vocaloid-search-desktop && npx vitest run src/features/playlistViews/playerColumnLayout.test.ts src/features/playlistViews/videoMetaPanelLayout.test.ts
```
Expected: FAIL in the newly added assertions before the implementation is corrected.

- [ ] **Step 3: Write minimal implementation**

Adjust only the shared-player/presentation pieces needed:
- remove/relax PiP-breaking outer shell reservation in `UnifiedPlayer.vue`,
- ensure `VideoMetaPanel.vue` semantic markers/props reflect the compact contract,
- keep player immediately below uploader/stats row,
- preserve shared divider rule.

- [ ] **Step 4: Run tests to verify they pass**

Run the same command from Step 2.
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/features/playlistViews/playerColumnLayout.test.ts src/features/playlistViews/videoMetaPanelLayout.test.ts src/components/UnifiedPlayer.vue src/components/VideoMetaPanel.vue
git commit -m "fix: tighten shared shells and PiP compact presentation"
```

---

### Task 4: Re-enter staged metadata readiness after PiP regain in usePlayerCore

**Files:**
- Modify: `vocaloid-search-desktop/src/composables/usePlayerCore.test.ts`
- Modify: `vocaloid-search-desktop/src/composables/usePlayerCore.ts`
- Test: `vocaloid-search-desktop/src/composables/usePlayerCore.test.ts`

- [ ] **Step 1: Write the failing behavioral test**

Add a real behavioral test in `usePlayerCore.test.ts` that covers the full regain-style staged sequence for the same playback identity:
- pending again for the same identity,
- still pending when enrichment has not arrived,
- ready again only after matching metadata update arrives,
- stale/non-matching metadata update does not satisfy readiness.

```ts
test('same playback identity can re-enter pending and only matching enrichment makes metadata ready again', async () => {
  const player = usePlayerCore({
    onPlayNext: vi.fn(),
    onMarkWatched: vi.fn(),
    onPlaybackStateChanged: vi.fn(),
    setupEvents: true,
    getPlaybackIdentity: () => ({
      playlistType: 'History',
      playlistVersion: 4,
      currentIndex: 1,
      videoId: 'sm9',
    }),
  })

  await capturedEventOptions.onVideoSelected({
    playlist_type: 'History',
    playlist_version: 4,
    index: 1,
    has_next: true,
    video: { id: 'sm9', title: 'selected title' },
  })

  expect(player.metadataReady.value).toBe(false)

  await player.handleVideoChange(player.currentVideo.value, 1, true)
  expect(player.metadataReady.value).toBe(false)

  await capturedEventOptions.onPlaybackMetadataUpdated({
    playlist_type: 'History',
    playlist_version: 5,
    index: 1,
    list_id: 'History',
    video: { id: 'sm9', title: 'stale title' },
  })
  expect(player.metadataReady.value).toBe(false)

  await capturedEventOptions.onPlaybackMetadataUpdated({
    playlist_type: 'History',
    playlist_version: 4,
    index: 1,
    list_id: 'History',
    video: { id: 'sm9', title: 'updated title' },
  })
  expect(player.metadataReady.value).toBe(true)
})
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd /d/Downloads/vocaloid-search-alt/vocaloid-search-desktop && npx vitest run src/composables/usePlayerCore.test.ts -t "same playback identity can re-enter pending and only matching enrichment makes metadata ready again"
```
Expected: FAIL until regain-style staged readiness is implemented.

- [ ] **Step 3: Write minimal implementation**

Update `usePlayerCore.ts` so staged readiness can re-enter pending for regain while preserving:
- mounted shells,
- early player rendering,
- later metadata fill-in only after matching enrichment,
- stale/non-matching enrichment remaining ignored.

Keep the change narrowly focused on readiness state and identity comparison logic.

- [ ] **Step 4: Run test to verify it passes**

Run the same command from Step 2.
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/composables/usePlayerCore.test.ts src/composables/usePlayerCore.ts
git commit -m "fix: re-enter staged metadata readiness after PiP regain"
```

---

### Task 5: Run regression tests for all three change axes plus existing shared-player behavior

**Files:**
- Test: `vocaloid-search-desktop/src/composables/usePlayerCore.test.ts`
- Test: `vocaloid-search-desktop/src/composables/usePlayerEvents.test.ts`
- Test: `vocaloid-search-desktop/src/features/playlistViews/playerColumnLayout.test.ts`
- Test: `vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts`
- Test: `vocaloid-search-desktop/src/features/playlistViews/pipAppCommandTarget.test.ts`
- Modify: `D:/Downloads/vocaloid-search-alt/openspec/changes/stabilize-player-shell-metadata/tasks.md`

- [ ] **Step 1: Run the targeted regression suite required by this project area**

Run:
```bash
cd /d/Downloads/vocaloid-search-alt/vocaloid-search-desktop && npx vitest run src/composables/usePlayerCore.test.ts src/composables/usePlayerEvents.test.ts src/features/playlistViews/playerColumnLayout.test.ts src/features/playlistViews/videoMetaPanelLayout.test.ts src/features/playlistViews/pipAppCommandTarget.test.ts
```
Expected: PASS, 0 failed

- [ ] **Step 2: Update OpenSpec task checklist**

Mark the relevant checkboxes in:
```text
D:/Downloads/vocaloid-search-alt/openspec/changes/stabilize-player-shell-metadata/tasks.md
```

Set completed:
- 3.1
- 3.2
- 4.1
- 4.2
- 4.3

- [ ] **Step 3: Commit**

```bash
git add src/composables/usePlayerCore.test.ts src/composables/usePlayerEvents.test.ts src/features/playlistViews/playerColumnLayout.test.ts src/features/playlistViews/videoMetaPanelLayout.test.ts src/features/playlistViews/pipAppCommandTarget.test.ts ../openspec/changes/stabilize-player-shell-metadata/tasks.md
git commit -m "test: cover PiP staged regain and compact metadata regressions"
```

---

### Task 6: Run final typecheck and production build

**Files:**
- Verify only: `vocaloid-search-desktop/`
- Modify: `D:/Downloads/vocaloid-search-alt/openspec/changes/stabilize-player-shell-metadata/tasks.md`

- [ ] **Step 1: Run Vue TypeScript typecheck**

Run:
```bash
cd /d/Downloads/vocaloid-search-alt/vocaloid-search-desktop && npx vue-tsc --noEmit
```
Expected: exit 0, no type errors

- [ ] **Step 2: Run production build**

Run:
```bash
cd /d/Downloads/vocaloid-search-alt/vocaloid-search-desktop && npm run build
```
Expected: build completes successfully and writes `dist/`

- [ ] **Step 3: Update OpenSpec verification checkboxes**

Mark completed:
- 5.1
- 5.2
- 5.3

- [ ] **Step 4: Commit**

```bash
git add ../openspec/changes/stabilize-player-shell-metadata/tasks.md
git commit -m "chore: verify PiP staged regain and compact metadata changes"
```
