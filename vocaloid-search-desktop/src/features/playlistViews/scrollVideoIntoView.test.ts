import { afterEach, describe, expect, test, vi } from 'vitest'
import { scrollVideoIntoView } from './playlistViewState'

describe('scrollVideoIntoView', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  test('function exists and is exported', () => {
    expect(scrollVideoIntoView).toBeDefined()
    expect(typeof scrollVideoIntoView).toBe('function')
  })

  test('does nothing when listContainer is null', () => {
    // Should not throw when container is null
    expect(() => scrollVideoIntoView(0, null)).not.toThrow()
    expect(() => scrollVideoIntoView(5, null)).not.toThrow()
    expect(() => scrollVideoIntoView(-1, null)).not.toThrow()
  })

  test('handles negative index gracefully', () => {
    // Should not throw with negative index
    expect(() => scrollVideoIntoView(-1, null)).not.toThrow()
  })

  test('function signature matches spec', () => {
    // Verify the function accepts the expected parameters
    // index: number, listContainer: HTMLElement | null
    const fn = scrollVideoIntoView
    expect(fn.length).toBe(2) // Function accepts 2 parameters
  })

  test('scrolls previous row into view when it is above the visible range', () => {
    const previousScrollIntoView = vi.fn()
    const currentScrollIntoView = vi.fn()
    const nextNextScrollIntoView = vi.fn()
    const listContainer = {
      getBoundingClientRect: () => ({ top: 100, bottom: 300 }),
    } as HTMLElement

    vi.stubGlobal('document', {
      getElementById: (id: string) => {
        if (id === 'video-1') {
          return {
            getBoundingClientRect: () => ({ top: 60, bottom: 120 }),
            scrollIntoView: previousScrollIntoView,
          }
        }
        if (id === 'video-2') {
          return {
            getBoundingClientRect: () => ({ top: 120, bottom: 180 }),
            scrollIntoView: currentScrollIntoView,
          }
        }
        if (id === 'video-4') {
          return {
            getBoundingClientRect: () => ({ top: 220, bottom: 260 }),
            scrollIntoView: nextNextScrollIntoView,
          }
        }
        return null
      },
    })

    scrollVideoIntoView(2, listContainer)

    expect(previousScrollIntoView).toHaveBeenCalledWith({ behavior: 'smooth', block: 'start' })
    expect(currentScrollIntoView).not.toHaveBeenCalled()
    expect(nextNextScrollIntoView).not.toHaveBeenCalled()
  })

  test('scrolls the next-next row into view when the lower buffer leaves the viewport', () => {
    const previousScrollIntoView = vi.fn()
    const currentScrollIntoView = vi.fn()
    const nextNextScrollIntoView = vi.fn()
    const listContainer = {
      getBoundingClientRect: () => ({ top: 100, bottom: 300 }),
    } as HTMLElement

    vi.stubGlobal('document', {
      getElementById: (id: string) => {
        if (id === 'video-1') {
          return {
            getBoundingClientRect: () => ({ top: 120, bottom: 180 }),
            scrollIntoView: previousScrollIntoView,
          }
        }
        if (id === 'video-2') {
          return {
            getBoundingClientRect: () => ({ top: 180, bottom: 220 }),
            scrollIntoView: currentScrollIntoView,
          }
        }
        if (id === 'video-4') {
          return {
            getBoundingClientRect: () => ({ top: 280, bottom: 340 }),
            scrollIntoView: nextNextScrollIntoView,
          }
        }
        return null
      },
    })

    scrollVideoIntoView(2, listContainer)

    expect(nextNextScrollIntoView).toHaveBeenCalledWith({ behavior: 'smooth', block: 'end' })
    expect(previousScrollIntoView).not.toHaveBeenCalled()
    expect(currentScrollIntoView).not.toHaveBeenCalled()
  })

  test('falls back to scrolling the current row when there is no next-next row', () => {
    const currentScrollIntoView = vi.fn()
    const listContainer = {
      getBoundingClientRect: () => ({ top: 100, bottom: 300 }),
    } as HTMLElement

    vi.stubGlobal('document', {
      getElementById: (id: string) => {
        if (id === 'video-0') {
          return {
            getBoundingClientRect: () => ({ top: 280, bottom: 360 }),
            scrollIntoView: currentScrollIntoView,
          }
        }
        return null
      },
    })

    scrollVideoIntoView(0, listContainer)

    expect(currentScrollIntoView).toHaveBeenCalledWith({ behavior: 'smooth', block: 'nearest' })
  })
})
