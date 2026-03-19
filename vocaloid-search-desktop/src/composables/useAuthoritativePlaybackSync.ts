import { api, type PlaylistType, type Video } from '../api/tauri-commands'

export interface AuthoritativePlaybackState {
  currentVideo: Video | null
  currentVideoIndex: number
  resultsCount: number
  hasNext: boolean
  pipActive?: boolean
  playlistType: PlaylistType
  playlistVersion: number
}

export interface AuthoritativePlaybackSync {
  refreshActivePlayback: () => Promise<void>
}

export function useAuthoritativePlaybackSync(
  syncState: (state: AuthoritativePlaybackState) => Promise<void> | void,
): AuthoritativePlaybackSync {
  async function refreshActivePlayback(): Promise<void> {
    const playlistState = await api.getPlaylistState()

    await syncState({
      currentVideo:
        playlistState.index !== null &&
        playlistState.index >= 0 &&
        playlistState.index < playlistState.results.length
          ? playlistState.results[playlistState.index]
          : null,
      currentVideoIndex: playlistState.index ?? -1,
      resultsCount: playlistState.results.length,
      hasNext: playlistState.has_next,
      pipActive: playlistState.pip_active,
      playlistType: playlistState.playlist_type,
      playlistVersion: playlistState.playlist_version,
    })
  }

  return {
    refreshActivePlayback,
  }
}
