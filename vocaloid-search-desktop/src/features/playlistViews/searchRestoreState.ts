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
  if (playlistState.results.length === 0) {
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
    : searchState.results ?? []

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
