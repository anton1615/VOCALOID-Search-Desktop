import { describe, expect, test } from 'vitest'
import {
  applyFormulaSelection,
  canPreloadSearchResults,
  cancelFormulaSelection,
  selectSortOption,
  shouldPreloadMore,
  toggleSortDirection,
} from './searchViewInteractions'

describe('searchViewInteractions', () => {
  test('toggleSortDirection flips desc to asc and asc to desc', () => {
    expect(toggleSortDirection('desc')).toBe('asc')
    expect(toggleSortDirection('asc')).toBe('desc')
  })

  test('selecting custom sort opens formula panel and copies sort weights', () => {
    expect(selectSortOption('custom', { view: 5, mylist: 3, comment: 2, like: 1 })).toEqual({
      sortField: null,
      showSortMenu: false,
      showFormulaPanel: true,
      localWeights: { view: 5, mylist: 3, comment: 2, like: 1 },
      shouldRunSearch: false,
    })
  })

  test('selecting normal sort closes menu and triggers search', () => {
    expect(selectSortOption('view', { view: 5, mylist: 3, comment: 2, like: 1 })).toEqual({
      sortField: 'view',
      showSortMenu: false,
      showFormulaPanel: false,
      localWeights: null,
      shouldRunSearch: true,
    })
  })

  test('applyFormulaSelection switches to custom sort and triggers search', () => {
    expect(applyFormulaSelection({ view: 1, mylist: 2, comment: 3, like: 4 })).toEqual({
      sortField: 'custom',
      sortWeights: { view: 1, mylist: 2, comment: 3, like: 4 },
      showFormulaPanel: false,
      shouldRunSearch: true,
    })
  })

  test('cancelFormulaSelection resets local weights and closes panel', () => {
    expect(cancelFormulaSelection()).toEqual({
      showFormulaPanel: false,
      localWeights: { view: 5, mylist: 3, comment: 2, like: 1 },
    })
  })

  test('shouldPreloadMore only triggers near end with next page available and not already loading', () => {
    expect(shouldPreloadMore({ resultsLength: 20, index: 9, hasNext: true, loadingMore: false })).toBe(true)
    expect(shouldPreloadMore({ resultsLength: 20, index: 1, hasNext: true, loadingMore: false })).toBe(false)
    expect(shouldPreloadMore({ resultsLength: 20, index: 9, hasNext: false, loadingMore: false })).toBe(false)
    expect(shouldPreloadMore({ resultsLength: 20, index: 9, hasNext: true, loadingMore: true })).toBe(false)
  })

  test('canPreloadSearchResults only allows Search playlists', () => {
    expect(canPreloadSearchResults('Search')).toBe(true)
    expect(canPreloadSearchResults('History')).toBe(false)
    expect(canPreloadSearchResults('WatchLater')).toBe(false)
  })
})
