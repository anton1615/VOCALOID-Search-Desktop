import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'
import type { Video } from '../../api/tauri-commands'
import {
  applyPlaybackMetadataUpdate,
  createHydratedCurrentVideo,
  getInitialPlaylistViewState,
  mergePagedResults,
  shouldApplyPlaybackMetadataUpdate,
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

  test('applies staged playback metadata updates only when playlist type, version, index, and video id all match', () => {
    const payload = {
      playlist_type: 'History' as const,
      playlist_version: 3,
      index: 1,
      list_id: 'History' as const,
      video: {
        ...baseVideo,
        id: 'sm10',
        title: 'Enriched title',
      },
    }

    expect(
      shouldApplyPlaybackMetadataUpdate({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 3,
        currentVideoIndex: 1,
        currentVideoId: 'sm10',
        payload,
      })
    ).toBe(true)

    expect(
      shouldApplyPlaybackMetadataUpdate({
        expectedPlaylistType: 'Search',
        expectedPlaylistVersion: 3,
        currentVideoIndex: 1,
        currentVideoId: 'sm10',
        payload,
      })
    ).toBe(false)

    expect(
      shouldApplyPlaybackMetadataUpdate({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 4,
        currentVideoIndex: 1,
        currentVideoId: 'sm10',
        payload,
      })
    ).toBe(false)

    expect(
      shouldApplyPlaybackMetadataUpdate({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 3,
        currentVideoIndex: 0,
        currentVideoId: 'sm10',
        payload,
      })
    ).toBe(false)

    expect(
      shouldApplyPlaybackMetadataUpdate({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 3,
        currentVideoIndex: 1,
        currentVideoId: 'sm9',
        payload,
      })
    ).toBe(false)
  })

  test('playback metadata updates replace visible video fields without changing selection index', () => {
    const currentVideo = {
      ...baseVideo,
      id: 'sm10',
      title: 'Placeholder title',
      like_count: 0,
    }

    const payload = {
      playlist_type: 'History' as const,
      playlist_version: 3,
      index: 1,
      list_id: 'History' as const,
      video: {
        ...currentVideo,
        title: 'Enriched title',
        like_count: 42,
        uploader_name: 'MikuP',
      },
    }

    const updated = applyPlaybackMetadataUpdate(currentVideo, payload)

    expect(updated).toEqual(payload.video)
    expect(updated).not.toBe(currentVideo)
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

  test('sync-route playback reset does not revive stale History selection on return', () => {
    const results = [baseVideo, { ...baseVideo, id: 'sm10' }]

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'History',
        expectedPlaylistVersion: 3,
        playlistType: 'Search',
        playlistVersion: 1,
        playlistIndex: -1,
        results,
      })
    ).toEqual({ selectedIndex: -1, selectedVideo: null })
  })

  test('sync-route playback reset does not revive stale Watch Later selection on return', () => {
    const results = [baseVideo, { ...baseVideo, id: 'sm10' }]

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'WatchLater',
        expectedPlaylistVersion: 5,
        playlistType: 'Search',
        playlistVersion: 1,
        playlistIndex: -1,
        results,
      })
    ).toEqual({ selectedIndex: -1, selectedVideo: null })
  })

  test('all playlist views listen for staged playback metadata updates', () => {
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const historyViewPath = resolve(__dirname, '../../views/HistoryView.vue')
    const watchLaterViewPath = resolve(__dirname, '../../views/WatchLaterView.vue')

    const searchSource = readFileSync(searchViewPath, 'utf8')
    const historySource = readFileSync(historyViewPath, 'utf8')
    const watchLaterSource = readFileSync(watchLaterViewPath, 'utf8')

    expect(searchSource).toContain("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'")
    expect(historySource).toContain("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'")
    expect(watchLaterSource).toContain("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'")
  })

  test('playlist metadata update handlers do not perform selection-only side effects', () => {
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const historyViewPath = resolve(__dirname, '../../views/HistoryView.vue')
    const watchLaterViewPath = resolve(__dirname, '../../views/WatchLaterView.vue')

    const searchSource = readFileSync(searchViewPath, 'utf8')
    const historySource = readFileSync(historyViewPath, 'utf8')
    const watchLaterSource = readFileSync(watchLaterViewPath, 'utf8')

    const searchMetadataHandler = searchSource.slice(
      searchSource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"),
      searchSource.indexOf("unlistenVideoWatched = await listen<", searchSource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"))
    )
    const historyMetadataHandler = historySource.slice(
      historySource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"),
      historySource.indexOf("unlistenVideoWatched = await listen<", historySource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"))
    )
    const watchLaterMetadataHandler = watchLaterSource.slice(
      watchLaterSource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"),
      watchLaterSource.indexOf("unlistenWatchLaterChanged = await listen(", watchLaterSource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"))
    )

    expect(searchMetadataHandler).not.toContain('scrollIntoView')
    expect(searchMetadataHandler).not.toContain('loadMore(')
    expect(historyMetadataHandler).not.toContain('scrollIntoView')
    expect(watchLaterMetadataHandler).not.toContain('scrollIntoView')
  })

  test('playlist metadata update handlers gate on authoritative playlist state identity fields', () => {
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const historyViewPath = resolve(__dirname, '../../views/HistoryView.vue')
    const watchLaterViewPath = resolve(__dirname, '../../views/WatchLaterView.vue')

    const searchSource = readFileSync(searchViewPath, 'utf8')
    const historySource = readFileSync(historyViewPath, 'utf8')
    const watchLaterSource = readFileSync(watchLaterViewPath, 'utf8')

    const searchMetadataHandler = searchSource.slice(
      searchSource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"),
      searchSource.indexOf("unlistenVideoWatched = await listen<", searchSource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"))
    )
    const historyMetadataHandler = historySource.slice(
      historySource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"),
      historySource.indexOf("unlistenVideoWatched = await listen<", historySource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"))
    )
    const watchLaterMetadataHandler = watchLaterSource.slice(
      watchLaterSource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"),
      watchLaterSource.indexOf("unlistenWatchLaterChanged = await listen(", watchLaterSource.indexOf("listen<PlaybackVideoUpdatedPayload>('playback-video-updated'"))
    )

    expect(searchMetadataHandler).toContain('latestPlaylistState.playlist_type')
    expect(searchMetadataHandler).toContain('latestPlaylistState.index')
    expect(searchMetadataHandler).toContain('latestPlaylistState.current_video_id')
    expect(historyMetadataHandler).toContain('latestPlaylistState.playlist_type')
    expect(historyMetadataHandler).toContain('latestPlaylistState.index')
    expect(historyMetadataHandler).toContain('latestPlaylistState.current_video_id')
    expect(watchLaterMetadataHandler).toContain('latestPlaylistState.playlist_type')
    expect(watchLaterMetadataHandler).toContain('latestPlaylistState.index')
    expect(watchLaterMetadataHandler).toContain('latestPlaylistState.current_video_id')
  })

  test('uploader avatar fallback stays independent from metadata cache fields', () => {
    const avatarStatePath = resolve(__dirname, './uploaderAvatarState.ts')
    const avatarStateSource = readFileSync(avatarStatePath, 'utf8')

    expect(avatarStateSource).toContain('BLANK_AVATAR_URL')
    expect(avatarStateSource).not.toContain('uploader_name')
    expect(avatarStateSource).not.toContain('description')
  })
})
