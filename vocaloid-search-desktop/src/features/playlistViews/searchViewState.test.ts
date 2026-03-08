import { describe, expect, test } from 'vitest'
import {
  buildSearchRequest,
  createSearchPersistenceState,
  restoreSearchPersistenceState,
  type SearchPersistenceState,
} from './searchViewState'

const baseState: SearchPersistenceState = {
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
}

describe('searchViewState', () => {
  test('creates persistence state snapshot from live search controls', () => {
    expect(createSearchPersistenceState({ ...baseState, excludeWatched: true }).excludeWatched).toBe(true)
  })

  test('restores persisted search state with defaults for missing values', () => {
    const restored = restoreSearchPersistenceState({ sortField: 'view', excludeWatched: true })

    expect(restored.sortField).toBe('view')
    expect(restored.sortOrder).toBe('desc')
    expect(restored.excludeWatched).toBe(true)
    expect(restored.startTimeGte).toBe('')
    expect(restored.formulaWeights.view).toBe(5)
  })

  test('builds search request with filters and formula filter only when active', () => {
    const request = buildSearchRequest({
      query: 'miku',
      page: 1,
      pageSize: 50,
      state: {
        ...baseState,
        excludeWatched: true,
        showFormulaFilter: true,
        formulaMinScore: 100,
        viewGte: 1000,
        startTimeGte: '2024-01-01',
      },
    })

    expect(request).toEqual({
      query: 'miku',
      page: 1,
      page_size: 50,
      sort: {
        by: 'start_time',
        direction: 'desc',
        weights: undefined,
      },
      exclude_watched: true,
      filters: {
        view: { gte: 1000 },
        start_time: { gte: '2024-01-01' },
      },
      formula_filter: {
        view_weight: 5,
        mylist_weight: 3,
        comment_weight: 2,
        like_weight: 1,
        min_score: 100,
      },
    })
  })

  test('omits empty filters and formula filter when inactive', () => {
    const request = buildSearchRequest({
      query: '',
      page: 1,
      pageSize: 50,
      state: baseState,
    })

    expect(request.filters).toBeUndefined()
    expect(request.formula_filter).toBeUndefined()
    expect(request.query).toBeUndefined()
  })
})
