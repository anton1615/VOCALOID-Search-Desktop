import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, describe, expect, test, vi } from 'vitest'
import {
  getVideoMetaPanelLayout,
  getVideoMetaPresentationContract,
} from './videoMetaPanelLayout'

describe('videoMetaPanelLayout', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  test('returns both sections for full mode', () => {
    expect(getVideoMetaPanelLayout('full')).toEqual({
      showHeader: true,
      showDetails: true,
    })
  })

  test('returns only header for header mode', () => {
    expect(getVideoMetaPanelLayout('header')).toEqual({
      showHeader: true,
      showDetails: false,
    })
  })

  test('returns only details for details mode', () => {
    expect(getVideoMetaPanelLayout('details')).toEqual({
      showHeader: false,
      showDetails: true,
    })
  })

  test('defines the full-mode metadata presentation contract', () => {
    expect(getVideoMetaPresentationContract('full')).toEqual({
      titleClampLines: 1,
      uploaderClampLines: 1,
      avatarSize: 'md',
      statsGap: 'normal',
      statsInlineSpacing: false,
      statsFirstInlinePriority: false,
      uploaderTruncatesBeforeStats: false,
      urlTreatment: 'surface',
      showTagDescriptionDivider: true,
      emphasizedMeta: true,
      fixedMetaRowHeight: true,
    })
  })

  test('defines the compact-mode metadata presentation contract', () => {
    expect(getVideoMetaPresentationContract('compact')).toEqual({
      titleClampLines: 2,
      uploaderClampLines: 1,
      avatarSize: 'sm',
      statsGap: 'spacious',
      statsInlineSpacing: true,
      statsFirstInlinePriority: true,
      uploaderTruncatesBeforeStats: true,
      urlTreatment: 'surface',
      showTagDescriptionDivider: true,
      emphasizedMeta: false,
      fixedMetaRowHeight: true,
      compactHeaderLineState: 'two-line',
    })
  })

  test('defines the compact single-line metadata presentation contract as a distinct variant', () => {
    expect(getVideoMetaPresentationContract('compact', 'single-line')).toEqual({
      titleClampLines: 2,
      uploaderClampLines: 1,
      avatarSize: 'sm',
      statsGap: 'spacious',
      statsInlineSpacing: true,
      statsFirstInlinePriority: true,
      uploaderTruncatesBeforeStats: true,
      urlTreatment: 'surface',
      showTagDescriptionDivider: true,
      emphasizedMeta: false,
      fixedMetaRowHeight: true,
      compactHeaderLineState: 'single-line',
    })
  })

  test('classifies compact title line state from actual content height and line height', async () => {
    const module = await import('./videoMetaPanelLayout') as {
      getCompactTitleLineState?: (metrics: { titleHeight: number, lineHeight: number, tolerance?: number }) => 'single-line' | 'two-line'
      getCompactTitleContentHeight?: (metrics: { elementHeight: number, minHeight: number }) => number
    }

    expect(module).toHaveProperty('getCompactTitleLineState')
    expect(module).toHaveProperty('getCompactTitleContentHeight')
    if (!module.getCompactTitleLineState || !module.getCompactTitleContentHeight) {
      return
    }

    expect(module.getCompactTitleContentHeight({
      elementHeight: 44,
      minHeight: 40,
    })).toBe(4)

    expect(module.getCompactTitleLineState({
      titleHeight: module.getCompactTitleContentHeight({
        elementHeight: 44,
        minHeight: 40,
      }),
      lineHeight: 20,
      tolerance: 2,
    })).toBe('single-line')

    expect(module.getCompactTitleLineState({
      titleHeight: module.getCompactTitleContentHeight({
        elementHeight: 63,
        minHeight: 40,
      }),
      lineHeight: 20,
      tolerance: 2,
    })).toBe('two-line')
  })

  test('does not let two-line frame min-height force compact titles into two-line state', async () => {
    const module = await import('./videoMetaPanelLayout') as {
      getCompactTitleContentHeight?: (metrics: { elementHeight: number, minHeight: number }) => number
      getCompactTitleLineState?: (metrics: { titleHeight: number, lineHeight: number, tolerance?: number }) => 'single-line' | 'two-line'
    }

    expect(module).toHaveProperty('getCompactTitleContentHeight')
    expect(module).toHaveProperty('getCompactTitleLineState')
    if (!module.getCompactTitleContentHeight || !module.getCompactTitleLineState) {
      return
    }

    const contentHeight = module.getCompactTitleContentHeight({
      elementHeight: 40,
      minHeight: 40,
    })

    expect(module.getCompactTitleLineState({
      titleHeight: contentHeight + 20,
      lineHeight: 20,
      tolerance: 2,
    })).toBe('single-line')
  })

  test('subtracts compact title frame min-height before classifying content lines', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain('const minHeight = Number.parseFloat(computedStyle.minHeight)')
    expect(source).toContain('const contentHeight = getCompactTitleContentHeight({')
    expect(source).toContain('elementHeight: titleElement.getBoundingClientRect().height,')
    expect(source).toContain('minHeight,')
  })

  test('classifies compact title line state from rendered content height and line height', async () => {
    const module = await import('./videoMetaPanelLayout') as {
      getCompactTitleLineState?: (metrics: { titleHeight: number, lineHeight: number, tolerance?: number }) => 'single-line' | 'two-line'
    }

    expect(module).toHaveProperty('getCompactTitleLineState')
    if (!module.getCompactTitleLineState) {
      return
    }

    expect(module.getCompactTitleLineState({
      titleHeight: 22,
      lineHeight: 20,
      tolerance: 2,
    })).toBe('single-line')

    expect(module.getCompactTitleLineState({
      titleHeight: 25,
      lineHeight: 20,
      tolerance: 2,
    })).toBe('two-line')
  })
  test('observes compact title resize events and disconnects the observer on cleanup', async () => {
    let observedElement: object | null = null
    let disconnected = false

    class FakeResizeObserver {
      static latestCallback: (() => void) | null = null

      constructor(onResize: () => void) {
        FakeResizeObserver.latestCallback = onResize
      }

      observe(element: object) {
        observedElement = element
      }

      disconnect() {
        disconnected = true
      }
    }

    vi.stubGlobal('ResizeObserver', FakeResizeObserver)

    const module = await import('./videoMetaPanelLayout') as {
      observeCompactTitleResize?: (element: HTMLElement, onResize: () => void) => () => void
    }
    const element = {} as HTMLElement
    let resizeCalls = 0

    expect(module).toHaveProperty('observeCompactTitleResize')
    if (!module.observeCompactTitleResize) {
      return
    }

    const stopObserving = module.observeCompactTitleResize(element, () => {
      resizeCalls += 1
    })

    expect(observedElement).toBe(element)
    const registeredCallback = FakeResizeObserver.latestCallback
    if (!registeredCallback) {
      throw new Error('Expected compact title resize observer callback to be registered')
    }
    registeredCallback()
    expect(resizeCalls).toBe(1)

    stopObserving()
    expect(disconnected).toBe(true)
  })

  test('shows the description toggle when collapsed content overflows even below the legacy character threshold', async () => {
    const module = await import('./videoMetaPanelLayout') as {
      shouldShowDescriptionToggle: (metrics: { scrollHeight: number, clientHeight: number }) => boolean
    }

    expect(module.shouldShowDescriptionToggle({
      scrollHeight: 180,
      clientHeight: 100,
    })).toBe(true)
  })

  test('hides the description toggle when collapsed content fits within the panel clamp', async () => {
    const module = await import('./videoMetaPanelLayout') as {
      shouldShowDescriptionToggle: (metrics: { scrollHeight: number, clientHeight: number }) => boolean
    }

    expect(module.shouldShowDescriptionToggle({
      scrollHeight: 100,
      clientHeight: 100,
    })).toBe(false)
  })

  test('observes description container resize events and disconnects the observer on cleanup', async () => {
    let observedElement: object | null = null
    let disconnected = false

    class FakeResizeObserver {
      static latestCallback: (() => void) | null = null

      constructor(onResize: () => void) {
        FakeResizeObserver.latestCallback = onResize
      }

      observe(element: object) {
        observedElement = element
      }

      disconnect() {
        disconnected = true
      }
    }

    vi.stubGlobal('ResizeObserver', FakeResizeObserver)

    const module = await import('./videoMetaPanelLayout') as {
      observeDescriptionToggleResize: (element: HTMLElement, onResize: () => void) => () => void
    }
    const element = {} as HTMLElement
    let resizeCalls = 0

    const stopObserving = module.observeDescriptionToggleResize(element, () => {
      resizeCalls += 1
    })

    expect(observedElement).toBe(element)
    const registeredCallback = FakeResizeObserver.latestCallback
    if (!registeredCallback) {
      throw new Error('Expected resize observer callback to be registered')
    }
    registeredCallback()
    expect(resizeCalls).toBe(1)

    stopObserving()
    expect(disconnected).toBe(true)
  })

  test('video meta panel consumes the shared presentation contract through stable DOM markers and classes', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain('getVideoMetaPresentationContract')
    expect(source).toContain('presentationMode?: VideoMetaPanelPresentationMode')
    expect(source).toContain("presentationMode: 'full'")
    expect(source).toContain("const presentationContract = computed(() => getVideoMetaPresentationContract(props.presentationMode, compactHeaderLineState.value))")
    expect(source).toContain(':data-presentation-mode="props.presentationMode"')
    expect(source).toContain(':data-title-clamp="presentationContract.titleClampLines"')
    expect(source).toContain(':data-uploader-clamp="presentationContract.uploaderClampLines"')
    expect(source).toContain(':data-stats-inline-priority="presentationContract.statsFirstInlinePriority ? \'stats-first\' : undefined"')
    expect(source).toContain(':data-uploader-priority="presentationContract.uploaderTruncatesBeforeStats ? \'truncate-first\' : undefined"')
    expect(source).toContain('title-clamp-${presentationContract.titleClampLines}')
    expect(source).toContain('title-frame-${presentationContract.titleClampLines}')
    expect(source).toContain(':class="[`uploader-clamp-${presentationContract.uploaderClampLines}`]"')
    expect(source).toContain(':class="[\'avatar\', `avatar-${presentationContract.avatarSize}`]"')
    expect(source).toContain(':class="[\'meta-row\', { \'meta-row-emphasized\': presentationContract.emphasizedMeta, \'meta-row-fixed-height\': presentationContract.fixedMetaRowHeight }]"')
    expect(source).toContain(':class="[\'stats\', `stats-gap-${presentationContract.statsGap}`, { \'stats-inline-spacing\': presentationContract.statsInlineSpacing }]"')
    expect(source).toContain(':class="[\'url-section\', `url-treatment-${presentationContract.urlTreatment}`]"')
    expect(source).toContain('presentationContract.showTagDescriptionDivider && visibleTags.length && video.description')
    expect(source).toContain('class="tag-description-divider"')
    expect(source).toContain('.title-clamp-1')
    expect(source).toContain('.title-clamp-2')
    expect(source).toContain('.uploader-clamp-1')
    expect(source).toContain('.avatar-sm')
    expect(source).toContain('.avatar-md')
    expect(source).toContain('.meta-row-emphasized')
    expect(source).toContain('.meta-row-fixed-height')
    expect(source).toContain('.stats-gap-normal')
    expect(source).toContain('.stats-gap-spacious')
    expect(source).toContain('.stats-inline-spacing')
    expect(source).toContain('.url-treatment-surface')
    expect(source).toContain('.tag-description-divider')
  })

  test('UnifiedPlayer passes compact and full presentation modes to VideoMetaPanel', () => {
    const unifiedPlayerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')
    const source = readFileSync(unifiedPlayerPath, 'utf8')

    expect(source).toContain(':presentation-mode="isCompact ? \'compact\' : \'full\'"')
  })

  test('VideoMetaPanel wires compact header line-state measurement through stable DOM markers', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain('getCompactTitleLineState')
    expect(source).toContain('observeCompactTitleResize')
    expect(source).toContain('const titleRef = ref<HTMLElement | null>(null)')
    expect(source).toContain("const compactHeaderLineState = ref<CompactHeaderLineState>('two-line')")
    expect(source).toContain(':data-compact-header-line-state="props.presentationMode === \'compact\' ? compactHeaderLineState : undefined"')
    expect(source).toContain('ref="titleRef"')
    expect(source).toContain("getVideoMetaPresentationContract(props.presentationMode, compactHeaderLineState.value)")
  })

  test('VideoMetaPanel limits compact title line-state wiring to compact header rendering', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain("if (props.presentationMode !== 'compact' || !layout.value.showHeader)")
    expect(source).toContain('window.getComputedStyle(titleElement)')
    expect(source).toContain('await nextTick()')
  })

  test('VideoMetaPanel lets the fixed compact header container control vertical spacing instead of hardcoded inner padding', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'] {")
    expect(source).toContain('--compact-player-header-height: 82px;')
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'] .player-header")
    expect(source).toContain('height: var(--compact-player-header-height);')
    expect(source).toContain('display: flex;')
    expect(source).toContain('flex-direction: column;')
    expect(source).toContain('padding: 0 var(--space-md);')
    expect(source).toContain('box-sizing: border-box;')
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'] .meta-row")
    expect(source).toContain('min-height: 24px;')
    expect(source).not.toContain('min-height: calc((var(--space-md) * 2) + (1.4em * 2) + var(--space-sm) + 32px + 1px);')
    expect(source).not.toContain('height: 100%;')
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'] .header-row")
    expect(source).toContain('margin-bottom: 0;')
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'] .player-header")
    expect(source).toContain('justify-content: space-evenly;')
    expect(source).not.toContain(".video-meta-panel[data-presentation-mode='compact'][data-compact-header-line-state='two-line'] .player-header")
    expect(source).not.toContain('justify-content: flex-start;')
    expect(source).not.toContain(".video-meta-panel[data-presentation-mode='compact'][data-compact-header-line-state='two-line'] .player-header {\n  justify-content: flex-start;\n  gap: var(--space-sm);\n}")
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'][data-compact-header-line-state='single-line'] .title-frame-2")
    expect(source).toContain('min-height: calc(1.4em * 1);')
  })

  test('VideoMetaPanel keeps the same fixed compact player-header height for two-line titles while preserving the two-line title frame', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'][data-compact-header-line-state='two-line'] .title-frame-2")
    expect(source).toContain('min-height: calc(1.4em * 2);')
  })

  test('UnifiedPlayer keeps PiP compact shell wiring without debug marker styles', () => {
    const unifiedPlayerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')
    const source = readFileSync(unifiedPlayerPath, 'utf8')

    expect(source).toContain("'player-shell-pending': !metadataReady")
    expect(source).toContain('data-shell="header"')
    expect(source).toContain('data-shell="details"')
    expect(source).toContain('class="video-container"')
    expect(source).not.toContain('.unified-player.compact [data-shell="header"] {')
    expect(source).not.toContain('outline: 2px solid #ff4d4f;')
    expect(source).not.toContain('.unified-player.compact .video-container {')
    expect(source).not.toContain('outline: 2px solid #40a9ff;')
    expect(source).not.toContain('.unified-player.compact [data-shell="details"] {')
    expect(source).not.toContain('outline: 2px solid #52c41a;')
    expect(source).not.toContain('player-shell-header-compact')
  })

  test('PiP uses an internal two-line title frame without reserving outer shell gap above the player', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const unifiedPlayerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')
    const videoMetaPanelSource = readFileSync(videoMetaPanelPath, 'utf8')
    const unifiedPlayerSource = readFileSync(unifiedPlayerPath, 'utf8')

    expect(videoMetaPanelSource).toContain('.title-frame-2')
    expect(videoMetaPanelSource).toContain('title-frame-${presentationContract.titleClampLines}')
    expect(unifiedPlayerSource).not.toContain('.player-shell-header {\n  min-height: 86px;\n}')
    expect(unifiedPlayerSource).toContain('.player-shell-header-pending-full {')
    expect(unifiedPlayerSource).toContain('min-height: 86px;')
  })

  test('PiP compact contract keeps uploader on one line and applies wider stat spacing', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain('.uploader-clamp-1')
    expect(source).toContain('.avatar-sm')
    expect(source).toContain('.stats-gap-spacious')
    expect(source).toContain('.stats-inline-spacing')
  })

  test('PiP compact contract exposes stats-first inline priority and uploader-first truncation markers', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain('data-stats-inline-priority')
    expect(source).toContain('data-uploader-priority')
    expect(source).toContain("'stats-first'")
    expect(source).toContain("'truncate-first'")
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'][data-stats-inline-priority='stats-first'] .stats")
    expect(source).toContain('flex-wrap: nowrap;')
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'][data-uploader-priority='truncate-first'] .uploader-info")
    expect(source).toContain('flex: 1 1 auto;')
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'][data-uploader-priority='truncate-first'] .user-name")
    expect(source).toContain('min-width: 0;')
    expect(source).toContain('overflow: hidden;')
    expect(source).toContain('text-overflow: ellipsis;')
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'][data-stats-inline-priority='stats-first'] .stat")
    expect(source).toContain('white-space: nowrap;')
  })

  test('PiP compact stats-first contract reduces stat gap under narrow widths before risking overflow', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain("@media (max-width: 420px)")
    expect(source).toContain(".video-meta-panel[data-presentation-mode='compact'][data-stats-inline-priority='stats-first'] .stats-gap-spacious")
    expect(source).toContain('gap: var(--space-md);')
  })

  test('PiP pending header keeps a compact frame without falling back to the full-mode shell reservation', () => {
    const unifiedPlayerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')
    const source = readFileSync(unifiedPlayerPath, 'utf8')

    expect(source).toContain("'player-shell-header-pending-compact': !metadataReady && isCompact")
    expect(source).toContain('class="player-shell-header-frame player-shell-header-frame-compact"')
    expect(source).toContain('v-if="!metadataReady && isCompact"')
  })

  test('uses overflow-based wiring instead of the legacy raw description length gate', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain('.expand-btn {')
    expect(source).toContain('display: block;')
    expect(source).toContain('width: 100%;')
    expect(source).toContain('margin-top: var(--space-sm);')
    expect(source).toContain('padding: var(--space-xs);')
    expect(source).toContain('border-radius: 4px;')
    expect(source).toContain('background: transparent;')
    expect(source).toContain('color: var(--color-accent-primary);')
    expect(source).toContain('font-size: var(--font-size-sm);')
    expect(source).toContain('font-weight: 600;')
    expect(source).toContain('text-align: center;')
    expect(source).toContain('.expand-btn:hover')
    expect(source).not.toContain('video.description.length > 200')
    expect(source).toContain('shouldShowDescriptionToggle')
    expect(source).toContain('observeDescriptionToggleResize')
    expect(source).toContain('onBeforeUnmount')
  })
})
