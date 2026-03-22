import type { PlaylistState, SearchState, Video } from '../../api/tauri-commands'

interface SearchRestoreSnapshot {
  shouldRunInitialSearch: boolean
  results: Video[]
  currentVideo: Video | null
  currentVideoIndex: number
  hasNext: boolean
  totalCount: number
  page: number
  pipActive: boolean
}

export function resolveSearchRestoreState(
  playlistState: PlaylistState,
  searchState: SearchState
): SearchRestoreSnapshot {
  const savedBrowsingResults = searchState.results ?? []
  const hasSavedBrowsingState = savedBrowsingResults.length > 0
    || searchState.version > 0
    || searchState.page > 1
    || searchState.total_count > 0
    || searchState.query.length > 0
    || searchState.exclude_watched
    || Boolean(searchState.sort)
    || Boolean(searchState.filters)
    || Boolean(searchState.formula_filter)

  if (playlistState.results.length === 0 && !hasSavedBrowsingState) {
    return {
      shouldRunInitialSearch: true,
      results: [],
      currentVideo: null,
      currentVideoIndex: -1,
      hasNext: false,
      totalCount: 0,
      page: 1,
      pipActive: playlistState.pip_active,
    }
  }

  const hasActiveSearchPlayback =
    playlistState.playlist_type === 'Search' &&
    playlistState.playlist_version === searchState.version
  const activeIndex = playlistState.index
  const hasValidSearchSelection =
    hasActiveSearchPlayback &&
    activeIndex !== null &&
    activeIndex >= 0 &&
    activeIndex < playlistState.results.length

  const browsingResults = hasActiveSearchPlayback
    ? playlistState.results
    : savedBrowsingResults

  return {
    shouldRunInitialSearch: false,
    results: browsingResults,
    currentVideo: hasValidSearchSelection && activeIndex !== null
      ? playlistState.results[activeIndex]
      : null,
    currentVideoIndex: hasValidSearchSelection && activeIndex !== null ? activeIndex : -1,
    hasNext: searchState.has_next,
    totalCount: searchState.total_count,
    page: searchState.page,
    pipActive: playlistState.pip_active,
  }
}
