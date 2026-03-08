import type { SearchRequest } from '../../api/tauri-commands'

export interface SearchPersistenceState {
  sortField: string
  sortOrder: string
  excludeWatched: boolean
  showFormulaFilter: boolean
  formulaWeights: {
    view: number
    mylist: number
    comment: number
    like: number
  }
  formulaMinScore: number
  sortWeights: {
    view: number
    mylist: number
    comment: number
    like: number
  }
  viewGte?: number
  viewLte?: number
  mylistGte?: number
  mylistLte?: number
  commentGte?: number
  commentLte?: number
  likeGte?: number
  likeLte?: number
  startTimeGte: string
  startTimeLte: string
}

const defaultWeights = {
  view: 5,
  mylist: 3,
  comment: 2,
  like: 1,
}

export function createSearchPersistenceState(state: SearchPersistenceState): SearchPersistenceState {
  return {
    ...state,
    formulaWeights: { ...state.formulaWeights },
    sortWeights: { ...state.sortWeights },
  }
}

export function restoreSearchPersistenceState(saved: Partial<SearchPersistenceState> | null | undefined): SearchPersistenceState {
  return {
    sortField: saved?.sortField || 'start_time',
    sortOrder: saved?.sortOrder || 'desc',
    excludeWatched: !!saved?.excludeWatched,
    showFormulaFilter: !!saved?.showFormulaFilter,
    formulaWeights: { ...defaultWeights, ...(saved?.formulaWeights || {}) },
    formulaMinScore: saved?.formulaMinScore || 0,
    sortWeights: { ...defaultWeights, ...(saved?.sortWeights || {}) },
    viewGte: saved?.viewGte,
    viewLte: saved?.viewLte,
    mylistGte: saved?.mylistGte,
    mylistLte: saved?.mylistLte,
    commentGte: saved?.commentGte,
    commentLte: saved?.commentLte,
    likeGte: saved?.likeGte,
    likeLte: saved?.likeLte,
    startTimeGte: saved?.startTimeGte || '',
    startTimeLte: saved?.startTimeLte || '',
  }
}

export function buildSearchRequest({
  query,
  page,
  pageSize,
  state,
}: {
  query: string
  page: number
  pageSize: number
  state: SearchPersistenceState
}): SearchRequest {
  const filters: NonNullable<SearchRequest['filters']> = {}

  if (Number.isFinite(state.viewGte) || Number.isFinite(state.viewLte)) {
    filters.view = {}
    if (Number.isFinite(state.viewGte)) filters.view.gte = state.viewGte
    if (Number.isFinite(state.viewLte)) filters.view.lte = state.viewLte
  }
  if (Number.isFinite(state.mylistGte) || Number.isFinite(state.mylistLte)) {
    filters.mylist = {}
    if (Number.isFinite(state.mylistGte)) filters.mylist.gte = state.mylistGte
    if (Number.isFinite(state.mylistLte)) filters.mylist.lte = state.mylistLte
  }
  if (Number.isFinite(state.commentGte) || Number.isFinite(state.commentLte)) {
    filters.comment = {}
    if (Number.isFinite(state.commentGte)) filters.comment.gte = state.commentGte
    if (Number.isFinite(state.commentLte)) filters.comment.lte = state.commentLte
  }
  if (Number.isFinite(state.likeGte) || Number.isFinite(state.likeLte)) {
    filters.like = {}
    if (Number.isFinite(state.likeGte)) filters.like.gte = state.likeGte
    if (Number.isFinite(state.likeLte)) filters.like.lte = state.likeLte
  }
  if (state.startTimeGte || state.startTimeLte) {
    filters.start_time = {}
    if (state.startTimeGte) filters.start_time.gte = state.startTimeGte
    if (state.startTimeLte) filters.start_time.lte = state.startTimeLte
  }

  const hasFilters = Object.keys(filters).length > 0

  return {
    query: query || undefined,
    page,
    page_size: pageSize,
    sort: {
      by: state.sortField,
      direction: state.sortOrder,
      weights: state.sortField === 'custom' ? state.sortWeights : undefined,
    },
    exclude_watched: state.excludeWatched,
    filters: hasFilters ? filters : undefined,
    formula_filter: state.showFormulaFilter && state.formulaMinScore > 0
      ? {
          view_weight: state.formulaWeights.view,
          mylist_weight: state.formulaWeights.mylist,
          comment_weight: state.formulaWeights.comment,
          like_weight: state.formulaWeights.like,
          min_score: state.formulaMinScore,
        }
      : undefined,
  }
}
