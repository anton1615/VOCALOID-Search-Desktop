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


/**
 * Scrolls the video list to ensure the currently playing video is visible.
 * This function implements the same scroll logic used in video-selected event handlers:
 * - Ensures the previous video is visible (in upper visible area)
 * - Ensures the next-next video is visible (in lower visible area)
 * - If no next-next video, ensures the current video is visible
 * 
 * @param index - The index of the currently playing video
 * @param listContainer - The list container element (can be null)
 */
export function scrollVideoIntoView(index: number, listContainer: HTMLElement | null): void {
  if (!listContainer) {
    return
  }

  const videoElement = document.getElementById('video-' + index)
  const prevVideoElement = document.getElementById('video-' + (index - 1))
  const nextNextVideoElement = document.getElementById('video-' + (index + 2))

  const containerRect = listContainer.getBoundingClientRect()

  // Check if we need to scroll up (previous video not visible)
  if (prevVideoElement && index > 0) {
    const prevRect = prevVideoElement.getBoundingClientRect()
    if (prevRect.top < containerRect.top) {
      // Previous video is above visible area, scroll to show it
      prevVideoElement.scrollIntoView({ behavior: 'smooth', block: 'start' })
      return
    }
  }

  // Check if we need to scroll down (video 2 positions below not visible)
  if (nextNextVideoElement) {
    const nextNextRect = nextNextVideoElement.getBoundingClientRect()
    if (nextNextRect.bottom > containerRect.bottom) {
      nextNextVideoElement.scrollIntoView({ behavior: 'smooth', block: 'end' })
      return
    }
  } else if (videoElement) {
    // Less than 2 videos below, just scroll current into view
    const videoRect = videoElement.getBoundingClientRect()
    if (videoRect.bottom > containerRect.bottom || videoRect.top < containerRect.top) {
      videoElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
    }
  }
}