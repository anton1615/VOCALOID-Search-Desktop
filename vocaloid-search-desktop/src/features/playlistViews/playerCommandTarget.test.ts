import { describe, expect, test, vi } from 'vitest'
import { resolvePlayerCommandTarget } from './playerCommandTarget'

describe('resolvePlayerCommandTarget', () => {
  test('prefers last message source when iframe window is unavailable', () => {
    const lastMessageSource = {
      postMessage: vi.fn(),
    }

    expect(resolvePlayerCommandTarget({
      lastMessageSource,
      iframeWindow: null,
    })).toBe(lastMessageSource)
  })

  test('falls back to iframe window when no message source is available', () => {
    const iframeWindow = {
      postMessage: vi.fn(),
    }

    expect(resolvePlayerCommandTarget({
      lastMessageSource: null,
      iframeWindow,
    })).toBe(iframeWindow)
  })

  test('returns null when neither message source nor iframe window is available', () => {
    expect(resolvePlayerCommandTarget({
      lastMessageSource: null,
      iframeWindow: null,
    })).toBeNull()
  })
})
