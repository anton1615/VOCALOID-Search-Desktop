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

  test('wires PiP player events through UnifiedPlayer shared event handling', () => {
    const pipAppPath = resolve(__dirname, '../../PipApp.vue')
    const source = readFileSync(pipAppPath, 'utf8')

    expect(source).toContain(':setup-events="true"')
    expect(source).toContain('@playback-state-changed="handlePlaybackStateChanged"')
    expect(source).not.toContain("listen<VideoSelectedPayload>('video-selected'")
    expect(source).not.toContain("listen('active-playback-cleared'")
  })

  test('wires main window player events through UnifiedPlayer shared event handling', () => {
    const playerColumnPath = resolve(__dirname, '../../components/PlayerColumn.vue')
    const appPath = resolve(__dirname, '../../App.vue')
    const playerColumnSource = readFileSync(playerColumnPath, 'utf8')
    const appSource = readFileSync(appPath, 'utf8')

    expect(playerColumnSource).toContain(':setup-events="true"')
    expect(playerColumnSource).toContain('@playback-state-changed="handlePlaybackStateChanged"')
    expect(appSource).toContain('@playback-state-changed="handlePlaybackStateChanged"')
    expect(appSource).not.toContain("listen<VideoSelectedPayload>('video-selected'")
    expect(appSource).not.toContain("listen('active-playback-cleared'")
  })
})
