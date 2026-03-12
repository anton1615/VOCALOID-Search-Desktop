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
  version: 0,
}

const emptyPlaylistState: PlaylistState = {
  playlist_type: 'Search',
  results: [],
  index: null,
  has_next: false,
  pip_active: false,
  playlist_version: 0,
}

describe('resolveSearchRestoreState', () => {
  test('restores existing Search playlist state when results are present', () => {
    const playlistState: PlaylistState = {
      playlist_type: 'Search',
      results: [baseVideo],
      index: 0,
      has_next: true,
      pip_active: true,
      playlist_version: 3,
    }

    const restored = resolveSearchRestoreState(playlistState, {
      ...baseSearchState,
      version: 3,
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
    expect(restored.currentVideoIndex).toBe(-1)
  })

  test('restores browsing state without reviving playback when active playlist moved away from search', () => {
    const restored = resolveSearchRestoreState({
      ...emptyPlaylistState,
      playlist_type: 'History',
      results: [baseVideo],
      index: 0,
      has_next: true,
      pip_active: true,
    }, {
      ...baseSearchState,
      has_next: true,
      total_count: 1,
      page: 2,
      results: [baseVideo],
    })

    expect(restored.shouldRunInitialSearch).toBe(false)
    expect(restored.results).toEqual([baseVideo])
    expect(restored.currentVideo).toBeNull()
    expect(restored.currentVideoIndex).toBe(-1)
    expect(restored.hasNext).toBe(true)
    expect(restored.totalCount).toBe(1)
    expect(restored.page).toBe(2)
    expect(restored.pipActive).toBe(true)
  })

  test('does not revive search playback when playlist version is stale', () => {
    const restored = resolveSearchRestoreState({
      ...emptyPlaylistState,
      playlist_type: 'Search',
      results: [baseVideo],
      index: 0,
      playlist_version: 2,
    }, {
      ...baseSearchState,
      version: 3,
      total_count: 1,
    })

    expect(restored.shouldRunInitialSearch).toBe(false)
    expect(restored.currentVideo).toBeNull()
    expect(restored.currentVideoIndex).toBe(-1)
  })

  test('keeps search browsing results instead of replacing them with active non-search playlist results', () => {
    const searchResults = [baseVideo, { ...baseVideo, id: 'sm10' }]
    const historyResults = [{ ...baseVideo, id: 'history-1' }]

    const restored = resolveSearchRestoreState({
      ...emptyPlaylistState,
      playlist_type: 'History',
      results: historyResults,
      index: 0,
      playlist_version: 1,
    }, {
      ...baseSearchState,
      version: 3,
      total_count: 2,
      page: 1,
      results: searchResults,
    } as SearchState & { results: Video[] })

    expect(restored.results).toEqual(searchResults)
    expect(restored.currentVideo).toBeNull()
    expect(restored.currentVideoIndex).toBe(-1)
  })
})
