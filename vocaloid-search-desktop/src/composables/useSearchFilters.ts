import { ref, reactive, computed, type Ref, type Reactive } from 'vue'
import { toggleSortDirection } from '../features/playlistViews/searchViewInteractions'
import type { SearchPersistenceState } from '../features/playlistViews/searchViewState'

export interface UseSearchFiltersReturn {
  sortField: Ref<string>
  sortOrder: Ref<string>
  excludeWatched: Ref<boolean>
  showAdvancedFilter: Ref<boolean>
  showFormulaFilter: Ref<boolean>
  sortWeights: Reactive<{ view: number; mylist: number; comment: number; like: number }>
  formulaWeights: Reactive<{ view: number; mylist: number; comment: number; like: number }>
  formulaMinScore: Ref<number>
  viewGte: Ref<number | undefined>
  viewLte: Ref<number | undefined>
  mylistGte: Ref<number | undefined>
  mylistLte: Ref<number | undefined>
  commentGte: Ref<number | undefined>
  commentLte: Ref<number | undefined>
  likeGte: Ref<number | undefined>
  likeLte: Ref<number | undefined>
  startTimeGte: Ref<string>
  startTimeLte: Ref<string>
  hasActiveFilters: Ref<boolean>
  toggleSortOrder: () => void
  resetFilters: () => void
  getFilterState: () => SearchPersistenceState
}

export function useSearchFilters(): UseSearchFiltersReturn {
  const sortField = ref('start_time')
  const sortOrder = ref('desc')
  const excludeWatched = ref(false)
  const showAdvancedFilter = ref(false)
  const showFormulaFilter = ref(false)

  const sortWeights = reactive({ view: 5, mylist: 3, comment: 2, like: 1 })
  const formulaWeights = reactive({ view: 5, mylist: 3, comment: 2, like: 1 })
  const formulaMinScore = ref(0)

  const viewGte = ref<number | undefined>(undefined)
  const viewLte = ref<number | undefined>(undefined)
  const mylistGte = ref<number | undefined>(undefined)
  const mylistLte = ref<number | undefined>(undefined)
  const commentGte = ref<number | undefined>(undefined)
  const commentLte = ref<number | undefined>(undefined)
  const likeGte = ref<number | undefined>(undefined)
  const likeLte = ref<number | undefined>(undefined)
  const startTimeGte = ref('')
  const startTimeLte = ref('')

  const hasActiveFilters = computed(() => {
    return viewGte.value !== undefined || viewLte.value !== undefined ||
           mylistGte.value !== undefined || mylistLte.value !== undefined ||
           commentGte.value !== undefined || commentLte.value !== undefined ||
           likeGte.value !== undefined || likeLte.value !== undefined ||
           startTimeGte.value !== '' || startTimeLte.value !== '' ||
           (showFormulaFilter.value && formulaMinScore.value > 0)
  })

  function toggleSortOrder(): void {
    sortOrder.value = toggleSortDirection(sortOrder.value)
  }

  function resetFilters(): void {
    viewGte.value = undefined
    viewLte.value = undefined
    mylistGte.value = undefined
    mylistLte.value = undefined
    commentGte.value = undefined
    commentLte.value = undefined
    likeGte.value = undefined
    likeLte.value = undefined
    startTimeGte.value = ''
    startTimeLte.value = ''
    showFormulaFilter.value = false
    formulaMinScore.value = 0
    Object.assign(formulaWeights, { view: 5, mylist: 3, comment: 2, like: 1 })
  }

  function getFilterState(): SearchPersistenceState {
    return {
      sortField: sortField.value,
      sortOrder: sortOrder.value,
      excludeWatched: excludeWatched.value,
      showFormulaFilter: showFormulaFilter.value,
      formulaWeights: { ...formulaWeights },
      formulaMinScore: formulaMinScore.value,
      sortWeights: { ...sortWeights },
      viewGte: viewGte.value,
      viewLte: viewLte.value,
      mylistGte: mylistGte.value,
      mylistLte: mylistLte.value,
      commentGte: commentGte.value,
      commentLte: commentLte.value,
      likeGte: likeGte.value,
      likeLte: likeLte.value,
      startTimeGte: startTimeGte.value,
      startTimeLte: startTimeLte.value,
    }
  }

  return {
    sortField,
    sortOrder,
    excludeWatched,
    showAdvancedFilter,
    showFormulaFilter,
    sortWeights,
    formulaWeights,
    formulaMinScore,
    viewGte,
    viewLte,
    mylistGte,
    mylistLte,
    commentGte,
    commentLte,
    likeGte,
    likeLte,
    startTimeGte,
    startTimeLte,
    hasActiveFilters,
    toggleSortOrder,
    resetFilters,
    getFilterState,
  }
}
