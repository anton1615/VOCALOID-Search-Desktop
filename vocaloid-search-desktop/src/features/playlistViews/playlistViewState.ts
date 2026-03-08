import type { PlaylistType, Video } from '../../api/tauri-commands'

interface InitialPlaylistViewStateOptions {
  expectedPlaylistType: PlaylistType
  playlistType: PlaylistType
  playlistIndex: number
  results: Video[]
}

interface InitialPlaylistViewState {
  selectedIndex: number
  selectedVideo: Video | null
}

export function getInitialPlaylistViewState({
  expectedPlaylistType,
  playlistType,
  playlistIndex,
  results,
}: InitialPlaylistViewStateOptions): InitialPlaylistViewState {
  if (playlistType !== expectedPlaylistType) {
    return {
      selectedIndex: -1,
      selectedVideo: null,
    }
  }

  if (playlistIndex < 0 || playlistIndex >= results.length) {
    return {
      selectedIndex: -1,
      selectedVideo: null,
    }
  }

  return {
    selectedIndex: playlistIndex,
    selectedVideo: results[playlistIndex],
  }
}

export function createHydratedCurrentVideo(baseVideo: Video, hydratedVideo: Video | null): Video {
  if (hydratedVideo) {
    return hydratedVideo
  }

  return {
    ...baseVideo,
    start_time: null,
  }
}

export function mergePagedResults(existing: Video[], incoming: Video[]): Video[] {
  return [...existing, ...incoming]
}
