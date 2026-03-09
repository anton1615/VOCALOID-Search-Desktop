import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'

describe('PipApp command target handling', () => {
  test('resolves player commands through the last message source before falling back to iframe window', () => {
    const pipAppPath = resolve(__dirname, '../../PipApp.vue')
    const source = readFileSync(pipAppPath, 'utf8')

    expect(source).toContain("import { resolvePlayerCommandTarget } from './features/playlistViews/playerCommandTarget'")
    expect(source).toContain("import { rememberPlayerMessageSource, type PostMessageTarget } from './features/playlistViews/playerMessageSource'")
    expect(source).toContain('let lastPlayerMessageSource: PostMessageTarget | null = null')
    expect(source).toContain('const target = resolvePlayerCommandTarget({')
    expect(source).toContain('lastMessageSource: lastPlayerMessageSource,')
    expect(source).toContain('iframeWindow: iframeRef.value?.contentWindow ?? null,')
    expect(source).toContain('lastPlayerMessageSource = rememberPlayerMessageSource(event.source)')
  })
})
