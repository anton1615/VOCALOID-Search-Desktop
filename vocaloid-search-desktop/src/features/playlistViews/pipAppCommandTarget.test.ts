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

  test('passes playback identity through shared player shells for metadata update filtering', () => {
    const playerColumnPath = resolve(__dirname, '../../components/PlayerColumn.vue')
    const unifiedPlayerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')
    const appPath = resolve(__dirname, '../../App.vue')
    const pipAppPath = resolve(__dirname, '../../PipApp.vue')

    const playerColumnSource = readFileSync(playerColumnPath, 'utf8')
    const unifiedPlayerSource = readFileSync(unifiedPlayerPath, 'utf8')
    const appSource = readFileSync(appPath, 'utf8')
    const pipAppSource = readFileSync(pipAppPath, 'utf8')

    expect(appSource).toContain('const playlistType = ref<PlaylistType>(\'Search\')')
    expect(appSource).toContain('const playlistVersion = ref(1)')
    expect(appSource).toContain(':playlist-type="playlistType"')
    expect(appSource).toContain(':playlist-version="playlistVersion"')
    expect(pipAppSource).toContain('const playlistType = ref<PlaylistType>(\'Search\')')
    expect(pipAppSource).toContain('const playlistVersion = ref(1)')
    expect(pipAppSource).toContain(':playlist-type="playlistType"')
    expect(pipAppSource).toContain(':playlist-version="playlistVersion"')
    expect(playerColumnSource).toContain(':playlist-type="playlistType"')
    expect(playerColumnSource).toContain(':playlist-version="playlistVersion"')
    expect(unifiedPlayerSource).toContain('playlistType: props.playlistType,')
    expect(unifiedPlayerSource).toContain('playlistVersion: props.playlistVersion,')
  })

  test('rebuilds both main-window and PiP player shells from authoritative playback identity fields, not only current video id', () => {
    const unifiedPlayerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')
    const source = readFileSync(unifiedPlayerPath, 'utf8')

    expect(source).toContain('props.playlistType,')
    expect(source).toContain('props.playlistVersion,')
    expect(source).toContain('props.currentVideoIndex,')
    expect(source).toContain('props.currentVideo?.id ?? null,')
    expect(source).not.toContain('if (video?.id !== oldVideo?.id)')
  })

  test('routes next and previous controls through authoritative Rust playback navigation instead of rebrowsing by index', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const pipAppPath = resolve(__dirname, '../../PipApp.vue')
    const apiPath = resolve(__dirname, '../../api/tauri-commands.ts')
    const appSource = readFileSync(appPath, 'utf8')
    const pipSource = readFileSync(pipAppPath, 'utf8')
    const apiSource = readFileSync(apiPath, 'utf8')

    expect(apiSource).toContain("return invoke('play_next')")
    expect(apiSource).toContain("return invoke('play_previous')")
    expect(appSource).toContain('await api.playNext()')
    expect(appSource).toContain('await api.playPrevious()')
    expect(pipSource).toContain('await api.playNext()')
    expect(pipSource).toContain('await api.playPrevious()')
    expect(appSource).not.toContain('await api.setPlaylistIndex(currentVideoIndex.value + 1)')
    expect(appSource).not.toContain('await api.setPlaylistIndex(currentVideoIndex.value - 1)')
    expect(pipSource).not.toContain('await api.setPlaylistIndex(currentIndex.value + 1)')
    expect(pipSource).not.toContain('await api.setPlaylistIndex(currentIndex.value - 1)')
  })

  test('routes PiP-close ownership regain through a single staged metadata re-entry path', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const tauriApiPath = resolve(__dirname, '../../api/tauri-commands.ts')
    const source = readFileSync(appPath, 'utf8')
    const apiSource = readFileSync(tauriApiPath, 'utf8')

    expect(apiSource).toContain("return invoke('reenter_active_playback_metadata')")
    expect(source).toContain('async function handlePipOwnershipRegained() {\n  await api.reenterActivePlaybackMetadata()\n}')
    expect(source).toContain('function handlePlaybackStateChanged() {')
    expect(source).toContain('void refreshActivePlayback()')
    expect(source).toContain('async function handlePipClosed() {')
    expect(source).toContain('await handlePipOwnershipRegained()')
    expect(source).not.toContain('function handlePipClosed() {\n  pipActive.value = false\n}')
    expect(source).not.toContain('pipActive.value = false')
  })
})
