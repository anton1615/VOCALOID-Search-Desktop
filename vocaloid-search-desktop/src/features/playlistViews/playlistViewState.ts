import type { PlaylistType, Video, VideoSelectedPayload } from '../../api/tauri-commands'

interface InitialPlaylistViewStateOptions {
  expectedPlaylistType: PlaylistType
  expectedPlaylistVersion: number
  playlistType: PlaylistType
  playlistVersion: number
  playlistIndex: number
  results: Video[]
}

interface InitialPlaylistViewState {
  selectedIndex: number
  selectedVideo: Video | null
}

export function getInitialPlaylistViewState({
  expectedPlaylistType,
  expectedPlaylistVersion,
  playlistType,
  playlistVersion,
  playlistIndex,
  results,
}: InitialPlaylistViewStateOptions): InitialPlaylistViewState {
  if (playlistType !== expectedPlaylistType || playlistVersion !== expectedPlaylistVersion) {
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

export function shouldApplyPlaylistSelection(
  expectedPlaylistType: PlaylistType,
  payload: Pick<VideoSelectedPayload, 'playlist_type'>
): boolean {
  return payload.playlist_type === expectedPlaylistType
}

export function shouldApplyPlaylistSelectionVersion(
  expectedPlaylistVersion: number,
  payload: Pick<VideoSelectedPayload, 'playlist_version'>
): boolean {
  return payload.playlist_version === expectedPlaylistVersion
}
