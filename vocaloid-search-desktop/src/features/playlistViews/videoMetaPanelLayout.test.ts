import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, describe, expect, test, vi } from 'vitest'
import { getVideoMetaPanelLayout } from './videoMetaPanelLayout'

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
