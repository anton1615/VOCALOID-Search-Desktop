const defaultWeights = { view: 5, mylist: 3, comment: 2, like: 1 }

export function toggleSortDirection(current: string): 'asc' | 'desc' {
  return current === 'desc' ? 'asc' : 'desc'
}

export function selectSortOption(
  field: string,
  currentSortWeights: { view: number; mylist: number; comment: number; like: number }
) {
  if (field === 'custom') {
    return {
      sortField: null,
      showSortMenu: false,
      showFormulaPanel: true,
      localWeights: { ...currentSortWeights },
      shouldRunSearch: false,
    }
  }

  return {
    sortField: field,
    showSortMenu: false,
    showFormulaPanel: false,
    localWeights: null,
    shouldRunSearch: true,
  }
}

export function applyFormulaSelection(localWeights: { view: number; mylist: number; comment: number; like: number }) {
  return {
    sortField: 'custom',
    sortWeights: { ...localWeights },
    showFormulaPanel: false,
    shouldRunSearch: true,
  }
}

export function cancelFormulaSelection() {
  return {
    showFormulaPanel: false,
    localWeights: { ...defaultWeights },
  }
}

export function shouldPreloadMore({
  resultsLength,
  index,
  hasNext,
  loadingMore,
}: {
  resultsLength: number
  index: number
  hasNext: boolean
  loadingMore: boolean
}): boolean {
  const remaining = resultsLength - index - 1
  return remaining <= 10 && hasNext && !loadingMore
}
