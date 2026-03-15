import { describe, test, expect, vi, beforeEach } from 'vitest'
import { useSearch } from './useSearch'

vi.mock('../api/tauri-commands', () => ({
  api: {
    search: vi.fn().mockResolvedValue({
      results: [
        { id: 'sm1', title: 'Video 1', view_count: 1000 },
        { id: 'sm2', title: 'Video 2', view_count: 2000 },
      ],
      total: 2,
      has_next: false,
    }),
    loadMore: vi.fn().mockResolvedValue({
      results: [{ id: 'sm3', title: 'Video 3', view_count: 3000 }],
      has_next: false,
    }),
    getSearchState: vi.fn().mockResolvedValue({ version: 1 }),
  },
}))

vi.mock('../features/playlistViews/searchViewState', () => ({
  buildSearchRequest: vi.fn().mockReturnValue({}),
}))

describe('useSearch', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  test('initial state is empty', () => {
    const { results, loading, totalCount, page, hasNext, loadingMore, query } = useSearch()

    expect(results.value).toEqual([])
    expect(loading.value).toBe(false)
    expect(totalCount.value).toBe(0)
    expect(page.value).toBe(1)
    expect(hasNext.value).toBe(false)
    expect(loadingMore.value).toBe(false)
    expect(query.value).toBe('')
  })

  test('search updates state', async () => {
    const { results, totalCount, hasNext, search, query } = useSearch()

    query.value = 'VOCALOID'

    await search({
      sortField: 'start_time',
      sortOrder: 'desc',
      excludeWatched: false,
      showFormulaFilter: false,
      formulaWeights: { view: 5, mylist: 3, comment: 2, like: 1 },
      formulaMinScore: 0,
      sortWeights: { view: 5, mylist: 3, comment: 2, like: 1 },
      viewGte: undefined,
      viewLte: undefined,
      mylistGte: undefined,
      mylistLte: undefined,
      commentGte: undefined,
      commentLte: undefined,
      likeGte: undefined,
      likeLte: undefined,
      startTimeGte: '',
      startTimeLte: '',
    })

    expect(results.value).toHaveLength(2)
    expect(totalCount.value).toBe(2)
    expect(hasNext.value).toBe(false)
  })

  test('loadMore does nothing when hasNext is false', async () => {
    const { loadMore, page, results } = useSearch()

    await loadMore()

    expect(page.value).toBe(1)
    expect(results.value).toEqual([])
  })

  test('clearResults resets state', () => {
    const { results, totalCount, page, hasNext, clearResults } = useSearch()

    results.value = [{ id: 'sm1' }] as any
    totalCount.value = 1
    page.value = 2
    hasNext.value = true

    clearResults()

    expect(results.value).toEqual([])
    expect(totalCount.value).toBe(0)
    expect(page.value).toBe(1)
    expect(hasNext.value).toBe(false)
  })
})
