import { describe, expect, test } from 'vitest'
import type { Video } from '../../api/tauri-commands'
import {
  createPagedPlaylistController,
  type PlaylistPageResponse,
} from './pagedPlaylistController'

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

describe('createPagedPlaylistController', () => {
  test('loads first page and replaces results', async () => {
    const controller = createPagedPlaylistController({
      initialPage: 1,
      initialPageSize: 50,
      initialSortOrder: 'desc',
      fetchPage: async () => ({
        total: 2,
        has_next: true,
        results: [baseVideo, { ...baseVideo, id: 'sm10' }],
      }),
    })

    const snapshot = await controller.loadFirstPage()

    expect(snapshot.results.map(video => video.id)).toEqual(['sm9', 'sm10'])
    expect(snapshot.totalCount).toBe(2)
    expect(snapshot.hasNext).toBe(true)
    expect(snapshot.page).toBe(1)
  })

  test('loads next page and appends results', async () => {
    const pages: PlaylistPageResponse[] = [
      {
        total: 3,
        has_next: true,
        results: [baseVideo, { ...baseVideo, id: 'sm10' }],
      },
      {
        total: 3,
        has_next: false,
        results: [{ ...baseVideo, id: 'sm11' }],
      },
    ]

    const controller = createPagedPlaylistController({
      initialPage: 1,
      initialPageSize: 50,
      initialSortOrder: 'desc',
      fetchPage: async (page) => pages[page - 1],
    })

    await controller.loadFirstPage()
    const snapshot = await controller.loadNextPage()

    expect(snapshot?.results.map(video => video.id)).toEqual(['sm9', 'sm10', 'sm11'])
    expect(snapshot?.hasNext).toBe(false)
    expect(snapshot?.page).toBe(2)
  })

  test('does not load next page when no next page exists', async () => {
    const controller = createPagedPlaylistController({
      initialPage: 1,
      initialPageSize: 50,
      initialSortOrder: 'desc',
      fetchPage: async () => ({
        total: 1,
        has_next: false,
        results: [baseVideo],
      }),
    })

    await controller.loadFirstPage()

    expect(await controller.loadNextPage()).toBeNull()
  })

  test('resets page when sort order changes', async () => {
    const calls: Array<{ page: number; sortOrder: string }> = []

    const controller = createPagedPlaylistController({
      initialPage: 3,
      initialPageSize: 50,
      initialSortOrder: 'desc',
      fetchPage: async (page, _pageSize, sortOrder) => {
        calls.push({ page, sortOrder })
        return {
          total: 1,
          has_next: false,
          results: [baseVideo],
        }
      },
    })

    controller.setSortOrder('asc')
    await controller.loadFirstPage()

    expect(calls).toEqual([{ page: 1, sortOrder: 'asc' }])
  })
})
