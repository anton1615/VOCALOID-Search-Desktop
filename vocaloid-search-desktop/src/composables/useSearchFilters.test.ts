import { describe, test, expect } from 'vitest'
import { useSearchFilters } from './useSearchFilters'

describe('useSearchFilters', () => {
  test('initial sort state', () => {
    const { sortField, sortOrder } = useSearchFilters()

    expect(sortField.value).toBe('start_time')
    expect(sortOrder.value).toBe('desc')
  })

  test('toggleSortOrder switches between asc and desc', () => {
    const { sortOrder, toggleSortOrder } = useSearchFilters()

    expect(sortOrder.value).toBe('desc')
    
    toggleSortOrder()
    expect(sortOrder.value).toBe('asc')
    
    toggleSortOrder()
    expect(sortOrder.value).toBe('desc')
  })

  test('hasActiveFilters is false initially', () => {
    const { hasActiveFilters } = useSearchFilters()

    expect(hasActiveFilters.value).toBe(false)
  })

  test('hasActiveFilters is true when viewGte is set', () => {
    const { hasActiveFilters, viewGte } = useSearchFilters()

    viewGte.value = 1000

    expect(hasActiveFilters.value).toBe(true)
  })

  test('hasActiveFilters is true when startTimeGte is set', () => {
    const { hasActiveFilters, startTimeGte } = useSearchFilters()

    startTimeGte.value = '2024-01-01'

    expect(hasActiveFilters.value).toBe(true)
  })

  test('hasActiveFilters is true when formula filter is active', () => {
    const { hasActiveFilters, showFormulaFilter, formulaMinScore } = useSearchFilters()

    showFormulaFilter.value = true
    formulaMinScore.value = 100

    expect(hasActiveFilters.value).toBe(true)
  })

  test('resetFilters clears all filters', () => {
    const { 
      viewGte, viewLte, mylistGte, mylistLte,
      commentGte, commentLte, likeGte, likeLte,
      startTimeGte, startTimeLte, showFormulaFilter, formulaMinScore,
      resetFilters, hasActiveFilters 
    } = useSearchFilters()

    viewGte.value = 1000
    viewLte.value = 10000
    mylistGte.value = 100
    startTimeGte.value = '2024-01-01'
    showFormulaFilter.value = true
    formulaMinScore.value = 100

    expect(hasActiveFilters.value).toBe(true)

    resetFilters()

    expect(viewGte.value).toBeUndefined()
    expect(viewLte.value).toBeUndefined()
    expect(mylistGte.value).toBeUndefined()
    expect(mylistLte.value).toBeUndefined()
    expect(commentGte.value).toBeUndefined()
    expect(commentLte.value).toBeUndefined()
    expect(likeGte.value).toBeUndefined()
    expect(likeLte.value).toBeUndefined()
    expect(startTimeGte.value).toBe('')
    expect(startTimeLte.value).toBe('')
    expect(showFormulaFilter.value).toBe(false)
    expect(formulaMinScore.value).toBe(0)
    expect(hasActiveFilters.value).toBe(false)
  })

  test('getFilterState returns current filter state', () => {
    const { sortField, sortOrder, excludeWatched, viewGte, getFilterState } = useSearchFilters()

    sortField.value = 'view'
    sortOrder.value = 'asc'
    excludeWatched.value = true
    viewGte.value = 1000

    const state = getFilterState()

    expect(state.sortField).toBe('view')
    expect(state.sortOrder).toBe('asc')
    expect(state.excludeWatched).toBe(true)
    expect(state.viewGte).toBe(1000)
  })
})
