import { describe, expect, test } from 'vitest'
import { clearPlayerMessageSource, rememberPlayerMessageSource, type PostMessageTarget } from './playerMessageSource'

function asMessageEventSource(source: PostMessageTarget): MessageEventSource {
  return source as unknown as MessageEventSource
}

describe('playerMessageSource', () => {
  test('remembers a postMessage-capable message source without wrapping it in reactivity', () => {
    const source: PostMessageTarget = {
      postMessage: () => {},
    }

    expect(rememberPlayerMessageSource(asMessageEventSource(source))).toBe(source)
  })

  test('ignores null or non-postMessage sources', () => {
    expect(rememberPlayerMessageSource(null)).toBeNull()
    expect(rememberPlayerMessageSource({} as Window)).toBeNull()
  })

  test('clears remembered source on video change', () => {
    const source: PostMessageTarget = {
      postMessage: () => {},
    }

    expect(clearPlayerMessageSource(source)).toBeNull()
  })
})
