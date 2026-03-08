import type { Video } from '../../api/tauri-commands'
import { mergePagedResults } from './playlistViewState'

export interface PlaylistPageResponse {
  total: number
  has_next: boolean
  results: Video[]
}

interface Snapshot {
  results: Video[]
  totalCount: number
  page: number
  pageSize: number
  hasNext: boolean
  sortOrder: 'desc' | 'asc'
}

interface CreatePagedPlaylistControllerOptions {
  initialPage: number
  initialPageSize: number
  initialSortOrder: 'desc' | 'asc'
  fetchPage: (page: number, pageSize: number, sortOrder: 'desc' | 'asc') => Promise<PlaylistPageResponse>
}

export function createPagedPlaylistController({
  initialPage,
  initialPageSize,
  initialSortOrder,
  fetchPage,
}: CreatePagedPlaylistControllerOptions) {
  let page = initialPage
  let pageSize = initialPageSize
  let sortOrder = initialSortOrder
  let results: Video[] = []
  let totalCount = 0
  let hasNext = false

  function snapshot(): Snapshot {
    return {
      results,
      totalCount,
      page,
      pageSize,
      hasNext,
      sortOrder,
    }
  }

  return {
    setSortOrder(nextSortOrder: 'desc' | 'asc') {
      sortOrder = nextSortOrder
      page = 1
    },

    async loadFirstPage(): Promise<Snapshot> {
      page = 1
      const response = await fetchPage(page, pageSize, sortOrder)
      results = response.results
      totalCount = response.total
      hasNext = response.has_next
      return snapshot()
    },

    async loadNextPage(): Promise<Snapshot | null> {
      if (!hasNext) {
        return null
      }

      page += 1
      const response = await fetchPage(page, pageSize, sortOrder)
      results = mergePagedResults(results, response.results)
      totalCount = response.total
      hasNext = response.has_next
      return snapshot()
    },
  }
}
