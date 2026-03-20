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
      urlTreatment: 'surface',
      showTagDescriptionDivider: true,
      emphasizedMeta: false,
      fixedMetaRowHeight: true,
    })
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
    expect(source).toContain("const presentationContract = computed(() => getVideoMetaPresentationContract(props.presentationMode))")
    expect(source).toContain(':data-presentation-mode="props.presentationMode"')
    expect(source).toContain(':data-title-clamp="presentationContract.titleClampLines"')
    expect(source).toContain(':data-uploader-clamp="presentationContract.uploaderClampLines"')
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
