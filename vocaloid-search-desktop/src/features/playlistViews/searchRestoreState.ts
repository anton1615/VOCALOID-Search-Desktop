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

  return {
    shouldRunInitialSearch: false,
    results: playlistState.results,
    currentVideo:
      playlistState.index >= 0 && playlistState.index < playlistState.results.length
        ? playlistState.results[playlistState.index]
        : null,
    currentVideoIndex: playlistState.index,
    hasNext: searchState.has_next,
    totalCount: searchState.total_count,
    page: searchState.page,
    pipActive: playlistState.pip_active,
  }
}
