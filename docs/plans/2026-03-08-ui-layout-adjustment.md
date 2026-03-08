# UI Layout Adjustment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Adjust the main player column and PiP information layout so the embedded player and metadata blocks appear in the approved order, and make the description expand/collapse button a full-width bar in both contexts.

**Architecture:** Keep the existing shared `VideoMetaPanel` and add a minimal display-mode capability so it can render header-only, details-only, or full content. Update `PlayerColumn.vue` and `PipApp.vue` to compose the same shared component in different orders without touching playlist authority, embedded player protocol, or PiP close/sync logic.

**Tech Stack:** Vue 3 SFCs, TypeScript, Vite, Vitest

---

### Task 1: Add display modes to the shared metadata panel

**Files:**
- Modify: `vocaloid-search-desktop/src/components/VideoMetaPanel.vue`
- Test: `vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts`

**Step 1: Write the failing test**

```ts
import { describe, expect, test } from 'vitest'
import { resolveVideoMetaPanelSections } from '../features/playlistViews/videoMetaPanelLayout'

describe('resolveVideoMetaPanelSections', () => {
  test('returns header-only visibility for header mode', () => {
    expect(resolveVideoMetaPanelSections('header')).toEqual({
      showHeader: true,
      showDetails: false,
    })
  })

  test('returns details-only visibility for details mode', () => {
    expect(resolveVideoMetaPanelSections('details')).toEqual({
      showHeader: false,
      showDetails: true,
    })
  })

  test('returns full visibility for full mode', () => {
    expect(resolveVideoMetaPanelSections('full')).toEqual({
      showHeader: true,
      showDetails: true,
    })
  })
})
```

**Step 2: Run test to verify it fails**

Run: `npm test -- src/features/playlistViews/videoMetaPanelLayout.test.ts`
Expected: FAIL because `videoMetaPanelLayout` does not exist yet.

**Step 3: Write minimal implementation**

Create `src/features/playlistViews/videoMetaPanelLayout.ts` with:

```ts
export type VideoMetaPanelMode = 'full' | 'header' | 'details'

export function resolveVideoMetaPanelSections(mode: VideoMetaPanelMode) {
  if (mode === 'header') {
    return { showHeader: true, showDetails: false }
  }
  if (mode === 'details') {
    return { showHeader: false, showDetails: true }
  }
  return { showHeader: true, showDetails: true }
}
```

Then update `VideoMetaPanel.vue` to:
- accept a new `mode` prop defaulting to `'full'`
- use `resolveVideoMetaPanelSections(mode)`
- render header block only when `showHeader`
- render tags/description block only when `showDetails`

**Step 4: Run test to verify it passes**

Run: `npm test -- src/features/playlistViews/videoMetaPanelLayout.test.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add vocaloid-search-desktop/src/components/VideoMetaPanel.vue vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.ts vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts
git commit -m "refactor: add video metadata panel display modes"
```

### Task 2: Reorder the main player column layout

**Files:**
- Modify: `vocaloid-search-desktop/src/components/PlayerColumn.vue`
- Test: `vocaloid-search-desktop/src/features/playlistViews/playerColumnLayout.test.ts`

**Step 1: Write the failing test**

```ts
import { describe, expect, test } from 'vitest'
import { getPlayerColumnLayoutOrder } from '../features/playlistViews/playerColumnLayout'

describe('getPlayerColumnLayoutOrder', () => {
  test('returns approved main column layout order', () => {
    expect(getPlayerColumnLayoutOrder()).toEqual([
      'header',
      'player',
      'controls',
      'details',
    ])
  })
})
```

**Step 2: Run test to verify it fails**

Run: `npm test -- src/features/playlistViews/playerColumnLayout.test.ts`
Expected: FAIL because helper does not exist yet.

**Step 3: Write minimal implementation**

Create `src/features/playlistViews/playerColumnLayout.ts` with:

```ts
export function getPlayerColumnLayoutOrder() {
  return ['header', 'player', 'controls', 'details'] as const
}
```

Then update `PlayerColumn.vue` to:
- render `VideoMetaPanel` once above the iframe with `mode="header"`
- keep iframe block next
- keep playback controls next
- render `VideoMetaPanel` again below controls with `mode="details"`
- pass the same video/uploader props to both renders
- keep placeholder and player-controller logic unchanged

**Step 4: Run test to verify it passes**

Run: `npm test -- src/features/playlistViews/playerColumnLayout.test.ts`
Expected: PASS

**Step 5: Run build verification**

Run: `npm run build`
Expected: PASS

**Step 6: Commit**

```bash
git add vocaloid-search-desktop/src/components/PlayerColumn.vue vocaloid-search-desktop/src/features/playlistViews/playerColumnLayout.ts vocaloid-search-desktop/src/features/playlistViews/playerColumnLayout.test.ts
git commit -m "refactor: reorder main player column layout"
```

### Task 3: Reorder the PiP content layout

**Files:**
- Modify: `vocaloid-search-desktop/src/PipApp.vue`
- Test: `vocaloid-search-desktop/src/features/playlistViews/pipLayout.test.ts`

**Step 1: Write the failing test**

```ts
import { describe, expect, test } from 'vitest'
import { getPipContentLayoutOrder } from '../features/playlistViews/pipLayout'

describe('getPipContentLayoutOrder', () => {
  test('returns approved PiP right-column layout order', () => {
    expect(getPipContentLayoutOrder()).toEqual([
      'header',
      'player',
      'details',
    ])
  })
})
```

**Step 2: Run test to verify it fails**

Run: `npm test -- src/features/playlistViews/pipLayout.test.ts`
Expected: FAIL because helper does not exist yet.

**Step 3: Write minimal implementation**

Create `src/features/playlistViews/pipLayout.ts` with:

```ts
export function getPipContentLayoutOrder() {
  return ['header', 'player', 'details'] as const
}
```

Then update `PipApp.vue` to:
- keep the left control sidebar unchanged
- render `VideoMetaPanel` above iframe with `mode="header"`
- render iframe next
- render `VideoMetaPanel` below iframe with `mode="details"`
- remove any duplicated details render now replaced by the shared panel usage

**Step 4: Run test to verify it passes**

Run: `npm test -- src/features/playlistViews/pipLayout.test.ts`
Expected: PASS

**Step 5: Run build verification**

Run: `npm run build`
Expected: PASS

**Step 6: Commit**

```bash
git add vocaloid-search-desktop/src/PipApp.vue vocaloid-search-desktop/src/features/playlistViews/pipLayout.ts vocaloid-search-desktop/src/features/playlistViews/pipLayout.test.ts
git commit -m "refactor: reorder pip metadata layout"
```

### Task 4: Change the description expand button to a bar-style button

**Files:**
- Modify: `vocaloid-search-desktop/src/components/VideoMetaPanel.vue`
- Test: `vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts`

**Step 1: Extend the failing test**

Add a new assertion-oriented test:

```ts
test('keeps details mode available for bar-style description controls', () => {
  expect(resolveVideoMetaPanelSections('details').showDetails).toBe(true)
})
```

This test remains simple because the actual visual verification is via build/manual review, but it preserves the details-only rendering path used by the button.

**Step 2: Run test to verify it still passes or fails only if details mode is broken**

Run: `npm test -- src/features/playlistViews/videoMetaPanelLayout.test.ts`
Expected: PASS or a meaningful failure related to details visibility.

**Step 3: Write minimal implementation**

Update `VideoMetaPanel.vue` styles for `.expand-btn` so it becomes a full-width bar button:

```css
.expand-btn {
  width: 100%;
  margin-top: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border: 1px solid var(--color-border-subtle);
  border-radius: 10px;
  background: var(--color-bg-hover);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: var(--font-size-sm);
  font-weight: 600;
  text-align: center;
}
```

Also add a hover state consistent with the current UI tokens.

**Step 4: Run build verification**

Run: `npm run build`
Expected: PASS

**Step 5: Commit**

```bash
git add vocaloid-search-desktop/src/components/VideoMetaPanel.vue vocaloid-search-desktop/src/features/playlistViews/videoMetaPanelLayout.test.ts
git commit -m "style: make description toggle a bar button"
```

### Task 5: Final verification

**Files:**
- Review: `vocaloid-search-desktop/src/components/PlayerColumn.vue`
- Review: `vocaloid-search-desktop/src/PipApp.vue`
- Review: `vocaloid-search-desktop/src/components/VideoMetaPanel.vue`

**Step 1: Run all frontend tests**

Run: `npm test`
Expected: PASS with all test files green.

**Step 2: Run build**

Run: `npm run build`
Expected: PASS

**Step 3: Manual UI verification checklist**

Confirm visually:
- Main window order is `header → player → controls → tags → description`
- PiP right column order is `header → player → tags → description`
- PiP left controls remain unchanged
- Description toggle is a bar-style button in both main and PiP
- No playback controls disappear from main window

**Step 4: Commit**

```bash
git add vocaloid-search-desktop/src/components/PlayerColumn.vue vocaloid-search-desktop/src/PipApp.vue vocaloid-search-desktop/src/components/VideoMetaPanel.vue vocaloid-search-desktop/src/features/playlistViews/
git commit -m "refactor: adjust player metadata layout"
```
