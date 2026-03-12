import { describe, expect, test } from 'vitest'
import { scrollVideoIntoView } from './playlistViewState'

describe('scrollVideoIntoView', () => {
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
})
