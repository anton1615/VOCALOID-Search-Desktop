import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'

describe('PipApp command target handling', () => {
  test('resolves player commands through the last message source before falling back to iframe window', () => {
    // After refactoring, the command target handling is in usePlayerCore composable
    const composablePath = resolve(__dirname, '../../composables/usePlayerCore.ts')
    const source = readFileSync(composablePath, 'utf8')

    expect(source).toContain("import { resolvePlayerCommandTarget } from '../features/playlistViews/playerCommandTarget'")
    expect(source).toContain("import { rememberPlayerMessageSource, clearPlayerMessageSource, type PostMessageTarget } from '../features/playlistViews/playerMessageSource'")
    expect(source).toContain('let lastPlayerMessageSource: PostMessageTarget | null = null')
    expect(source).toContain('const target = resolvePlayerCommandTarget({')
    expect(source).toContain('lastMessageSource: lastPlayerMessageSource,')
    expect(source).toContain('iframeWindow: iframeRef?.contentWindow ?? null,')
    expect(source).toContain('lastPlayerMessageSource = rememberPlayerMessageSource(event.source)')
  })

  test('guards PiP playback updates with playlist version-aware playlist state checks', () => {
    // After refactoring, the playlist version checks are in PipApp.vue
    const pipAppPath = resolve(__dirname, '../../PipApp.vue')
    const source = readFileSync(pipAppPath, 'utf8')

    expect(source).toContain('const latestPlaylistState = await api.getPlaylistState()')
    expect(source).toContain('payload.playlist_type !== latestPlaylistState.playlist_type')
    expect(source).toContain('payload.playlist_version !== latestPlaylistState.playlist_version')
    expect(source).toContain('await handleVideoChange(payload.video, payload.index, payload.has_next)')
  })
})
