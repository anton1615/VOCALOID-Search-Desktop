import { describe, expect, test } from 'vitest'
import type { PlaylistState, SearchState, Video } from '../../api/tauri-commands'
import { resolveSearchRestoreState } from './searchRestoreState'

const baseVideo: Video = {
  id: 'sm9',
  title: 'Test Video',
  thumbnail_url: 'https://example.com/thumb.jpg',
  watch_url: null,
  view_count: 0,
  comment_count: 0,
  mylist_count: 0,
  like_count: 0,
  start_time: null,
  tags: [],
  duration: null,
  uploader_id: null,
  uploader_name: null,
  description: null,
  is_watched: false,
}

const baseSearchState: SearchState = {
  query: '',
  exclude_watched: false,
  page: 1,
  page_size: 50,
  has_next: false,
  total_count: 0,
}

const emptyPlaylistState: PlaylistState = {
  playlist_type: 'Search',
  results: [],
  index: -1,
  has_next: false,
  pip_active: false,
}

describe('resolveSearchRestoreState', () => {
  test('restores existing Search playlist state when results are present', () => {
    const playlistState: PlaylistState = {
      playlist_type: 'Search',
      results: [baseVideo],
      index: 0,
      has_next: true,
      pip_active: true,
    }

    const restored = resolveSearchRestoreState(playlistState, {
      ...baseSearchState,
      has_next: true,
      total_count: 1,
      page: 3,
    })

    expect(restored).toEqual({
      shouldRunInitialSearch: false,
      results: [baseVideo],
      currentVideo: baseVideo,
      currentVideoIndex: 0,
      hasNext: true,
      totalCount: 1,
      page: 3,
      pipActive: true,
    })
  })

  test('falls back to initial search when no existing playlist results exist', () => {
    const restored = resolveSearchRestoreState(emptyPlaylistState, baseSearchState)

    expect(restored.shouldRunInitialSearch).toBe(true)
    expect(restored.results).toEqual([])
    expect(restored.currentVideo).toBeNull()
    expect(restored.currentVideoIndex).toBe(-1)
  })

  test('does not restore current video when playlist index is out of range', () => {
    const restored = resolveSearchRestoreState({
      ...emptyPlaylistState,
      results: [baseVideo],
      index: 5,
    }, {
      ...baseSearchState,
      total_count: 1,
    })

    expect(restored.shouldRunInitialSearch).toBe(false)
    expect(restored.currentVideo).toBeNull()
    expect(restored.currentVideoIndex).toBe(5)
  })
})
