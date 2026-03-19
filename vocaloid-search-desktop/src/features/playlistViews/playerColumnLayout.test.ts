import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'
import { getPlayerColumnLayout } from './playerColumnLayout'

describe('playerColumnLayout', () => {
  test('returns the right column sections in header, player, controls, details order', () => {
    expect(getPlayerColumnLayout()).toEqual([
      {
        section: 'header',
        videoMetaPanelMode: 'header',
      },
      {
        section: 'player',
      },
      {
        section: 'controls',
      },
      {
        section: 'details',
        videoMetaPanelMode: 'details',
      },
    ])
  })

  test('app shell owns the shared main-window player region', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const historyViewPath = resolve(__dirname, '../../views/HistoryView.vue')
    const watchLaterViewPath = resolve(__dirname, '../../views/WatchLaterView.vue')

    const appSource = readFileSync(appPath, 'utf8')
    const searchSource = readFileSync(searchViewPath, 'utf8')
    const historySource = readFileSync(historyViewPath, 'utf8')
    const watchLaterSource = readFileSync(watchLaterViewPath, 'utf8')

    expect(appSource).toContain('PlayerColumn')
    expect(searchSource).not.toContain('<PlayerColumn')
    expect(historySource).not.toContain('<PlayerColumn')
    expect(watchLaterSource).not.toContain('<PlayerColumn')
  })

  test('playback views no longer hydrate metadata directly after selection', () => {
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const historyViewPath = resolve(__dirname, '../../views/HistoryView.vue')
    const watchLaterViewPath = resolve(__dirname, '../../views/WatchLaterView.vue')

    const searchSource = readFileSync(searchViewPath, 'utf8')
    const historySource = readFileSync(historyViewPath, 'utf8')
    const watchLaterSource = readFileSync(watchLaterViewPath, 'utf8')

    expect(searchSource).not.toContain('api.getUserInfo')
    expect(historySource).not.toContain('api.fetchFullVideoInfo')
    expect(historySource).not.toContain('api.updatePlaylistVideo')
    expect(historySource).not.toContain('api.getUserInfo')
    expect(watchLaterSource).not.toContain('api.fetchFullVideoInfo')
    expect(watchLaterSource).not.toContain('api.updatePlaylistVideo')
    expect(watchLaterSource).not.toContain('api.getUserInfo')
  })

  test('app shell and shared player no longer fetch playback metadata directly', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const playerInfoPath = resolve(__dirname, '../../composables/usePlayerInfo.ts')
    const playerCorePath = resolve(__dirname, '../../composables/usePlayerCore.ts')

    const appSource = readFileSync(appPath, 'utf8')
    const playerInfoSource = readFileSync(playerInfoPath, 'utf8')
    const playerCoreSource = readFileSync(playerCorePath, 'utf8')

    expect(appSource).not.toContain('api.fetchFullVideoInfo')
    expect(playerInfoSource).not.toContain('api.getUserInfo')
    expect(playerCoreSource).not.toContain('fetchUserInfo(')
  })

  test('shared player gates metadata panel on metadata readiness state', () => {
    const playerPath = resolve(__dirname, '../../components/UnifiedPlayer.vue')
    const playerCorePath = resolve(__dirname, '../../composables/usePlayerCore.ts')

    const playerSource = readFileSync(playerPath, 'utf8')
    const playerCoreSource = readFileSync(playerCorePath, 'utf8')

    expect(playerSource).toContain('metadataReady')
    expect(playerCoreSource).toContain('metadataReady')
  })

  test('split layout keeps the list pane scrollable after shell extraction', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')

    const appSource = readFileSync(appPath, 'utf8')
    const searchSource = readFileSync(searchViewPath, 'utf8')

    expect(appSource).toContain('.list-pane {')
    expect(appSource).toContain('flex: 1;')
    expect(searchSource).toContain('.list-column {')
    expect(searchSource).toContain('height: 100%;')
    expect(searchSource).toContain('overflow: hidden;')
    expect(searchSource).toContain('.video-list {')
    expect(searchSource).toContain('overflow-y: auto;')
  })

  test('search view uses a dedicated list container ref for scrolling instead of document query selection', () => {
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const searchSource = readFileSync(searchViewPath, 'utf8')

    expect(searchSource).toContain('const listContainerRef = ref<HTMLElement | null>(null)')
    expect(searchSource).toContain('const listContainer = listContainerRef.value')
    expect(searchSource).not.toContain("document.querySelector('.video-list')")
    expect(searchSource).toContain('ref="listContainerRef"')
  })
})
