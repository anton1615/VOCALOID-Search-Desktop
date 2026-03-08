import { describe, expect, test } from 'vitest'
import type { Video } from '../../api/tauri-commands'
import {
  createHydratedCurrentVideo,
  getInitialPlaylistViewState,
  mergePagedResults,
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
  test('restores saved playlist item only when playlist type matches and index is in range', () => {
    const results = [baseVideo, { ...baseVideo, id: 'sm10' }]

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'History',
        playlistType: 'History',
        playlistIndex: 1,
        results,
      })
    ).toEqual({ selectedIndex: 1, selectedVideo: results[1] })

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'History',
        playlistType: 'WatchLater',
        playlistIndex: 1,
        results,
      })
    ).toEqual({ selectedIndex: -1, selectedVideo: null })

    expect(
      getInitialPlaylistViewState({
        expectedPlaylistType: 'History',
        playlistType: 'History',
        playlistIndex: 5,
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
})
