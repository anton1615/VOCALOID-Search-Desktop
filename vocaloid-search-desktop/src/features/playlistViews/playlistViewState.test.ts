import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'
import type { Video } from '../../api/tauri-commands'
import {
  createHydratedCurrentVideo,
  getInitialPlaylistViewState,
  mergePagedResults,
  shouldApplyPlaylistSelection,
  shouldApplyPlaylistSelectionVersion,
} from './playlistViewState'

const baseVideo: Video = {
  id: 'sm9',
  title: 'Test Video',
  thumbnail_url: 'https://example.com/thumb.jpg',
  watch_url: null,
  view_count: 0,
  comment_count: 0,
  mylist_count: 0,
  like_count: 0,
  start_time: '2026-03-08T00:00:00Z',
  tags: [],
  duration: null,
  uploader_id: null,
  uploader_name: null,
  description: null,
  is_watched: false,
}

describe('playlistViewState shared logic', () => {
  test('restores saved playlist item only when playlist type, version, and index all match', () => {
    const results = [baseVideo, { ...baseVideo, id: 'sm10' }]

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 3,
        playlistType: 'History',
        playlistVersion: 3,
        playlistIndex: 1,
        results,
      })
    ).toEqual({ selectedIndex: 1, selectedVideo: results[1] })

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 3,
        playlistType: 'WatchLater',
        playlistVersion: 3,
        playlistIndex: 1,
        results,
      })
    ).toEqual({ selectedIndex: -1, selectedVideo: null })

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 3,
        playlistType: 'History',
        playlistVersion: 2,
        playlistIndex: 1,
        results,
      })
    ).toEqual({ selectedIndex: -1, selectedVideo: null })

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 3,
        playlistType: 'History',
        playlistVersion: 3,
        playlistIndex: 5,
        results,
      })
    ).toEqual({ selectedIndex: -1, selectedVideo: null })

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 3,
        playlistType: 'History',
        playlistVersion: 3,
        playlistIndex: -1,
        results,
      })
    ).toEqual({ selectedIndex: -1, selectedVideo: null })
  })

  test('falls back to placeholder video with cleared start_time when hydration fails', () => {
    expect(createHydratedCurrentVideo(baseVideo, null)).toEqual({
      ...baseVideo,
      start_time: null,
    })
  })

  test('prefers hydrated video when fetch succeeds', () => {
    const hydrated = {
      ...baseVideo,
      description: 'hydrated',
      uploader_id: '123',
      start_time: '2020-01-01T00:00:00Z',
    }

    expect(createHydratedCurrentVideo(baseVideo, hydrated)).toEqual(hydrated)
  })

  test('appends paged results without mutating the original array', () => {
    const existing = [baseVideo]
    const incoming = [{ ...baseVideo, id: 'sm10' }]

    const merged = mergePagedResults(existing, incoming)

    expect(merged).toEqual([baseVideo, incoming[0]])
    expect(existing).toEqual([baseVideo])
  })

  test('applies playlist selection only when payload playlist type matches expected view', () => {
    expect(shouldApplyPlaylistSelection('Search', { playlist_type: 'Search' })).toBe(true)
    expect(shouldApplyPlaylistSelection('Search', { playlist_type: 'History' })).toBe(false)
    expect(shouldApplyPlaylistSelection('WatchLater', { playlist_type: 'Search' })).toBe(false)
  })

  test('applies playlist selection only when payload playlist version matches expected view version', () => {
    expect(shouldApplyPlaylistSelectionVersion(3, { playlist_version: 3 })).toBe(true)
    expect(shouldApplyPlaylistSelectionVersion(3, { playlist_version: 2 })).toBe(false)
  })

  test('only explicit play handlers rebind the active playlist source', () => {
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const historyViewPath = resolve(__dirname, '../../views/HistoryView.vue')
    const watchLaterViewPath = resolve(__dirname, '../../views/WatchLaterView.vue')

    const searchSource = readFileSync(searchViewPath, 'utf8')
    const historySource = readFileSync(historyViewPath, 'utf8')
    const watchLaterSource = readFileSync(watchLaterViewPath, 'utf8')

    expect(searchSource).toContain("await api.setPlaylistType('Search')")
    expect(historySource).toContain("await api.setPlaylistType('History')")
    expect(watchLaterSource).toContain("await api.setPlaylistType('WatchLater')")

    expect(searchSource).not.toContain("onMounted(async () => {\n  await api.setPlaylistType('Search')")
    expect(historySource).not.toContain("onMounted(async () => {\n  await api.setPlaylistType('History')")
    expect(watchLaterSource).not.toContain("onMounted(async () => {\n  await api.setPlaylistType('WatchLater')")
  })
})
