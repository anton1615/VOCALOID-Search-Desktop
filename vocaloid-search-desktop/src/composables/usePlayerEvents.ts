import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PlaybackSettings, PlaybackVideoUpdatedPayload, VideoSelectedPayload } from '../api/tauri-commands'

/**
 * Options for player events
 */
export interface PlayerEventsOptions {
  /** Whether this is the PIP window */
  isPip?: boolean
  /** Called when a video is selected */
  onVideoSelected: (payload: VideoSelectedPayload) => Promise<void>
  /** Called when playback metadata is updated for the current playback identity */
  onPlaybackMetadataUpdated?: (payload: PlaybackVideoUpdatedPayload) => Promise<void> | void
  /** Called when playback settings change */
  onPlaybackSettingsChanged: (settings: PlaybackSettings) => void
  /** Called when a video is marked as watched */
  onVideoWatched: (videoId: string, isWatched: boolean) => void
  /** Called when active playback is cleared (search conditions changed) */
  onActivePlaybackCleared: () => void
}

/**
 * Return type for usePlayerEvents
 */
export interface PlayerEvents {
  setupEventListeners: () => () => void
}

/**
 * Composable for handling all player-related events from the backend.
 * This ensures consistent event handling across main and PIP windows.
 */
export function usePlayerEvents(options: PlayerEventsOptions): PlayerEvents {
  const {
    isPip = false,
    onVideoSelected,
    onPlaybackSettingsChanged,
    onVideoWatched,
    onActivePlaybackCleared,
    onPlaybackMetadataUpdated,
  } = options

  let unlistenVideoSelected: UnlistenFn | null = null
  let unlistenPlaybackVideoUpdated: UnlistenFn | null = null
  let unlistenPlaybackSettings: UnlistenFn | null = null
  let unlistenVideoWatched: UnlistenFn | null = null
  let unlistenActivePlaybackCleared: UnlistenFn | null = null

  /**
   * Set up all event listeners.
   * Returns a cleanup function to remove all listeners.
   */
  function setupEventListeners(): () => void {
    // Listen for video-selected event
    listen<VideoSelectedPayload>('video-selected', async (event) => {
      const payload = event.payload
      console.log(`[${isPip ? 'PiP' : 'Main'}] Received video-selected event:`, payload.video.id, 'index:', payload.index)
      await onVideoSelected(payload)
    }).then((unlisten) => {
      unlistenVideoSelected = unlisten
    })

    // Listen for playback-video-updated event
    listen<PlaybackVideoUpdatedPayload>('playback-video-updated', async (event) => {
      const payload = event.payload
      console.log(`[${isPip ? 'PiP' : 'Main'}] Received playback-video-updated event:`, payload.video.id, 'index:', payload.index)
      await onPlaybackMetadataUpdated?.(payload)
    }).then((unlisten) => {
      unlistenPlaybackVideoUpdated = unlisten
    })

    // Listen for playback-settings-changed event
    listen<PlaybackSettings>('playback-settings-changed', (event) => {
      const settings = event.payload
      console.log(`[${isPip ? 'PiP' : 'Main'}] Received playback-settings-changed event`)
      onPlaybackSettingsChanged(settings)
    }).then((unlisten) => {
      unlistenPlaybackSettings = unlisten
    })

    // Listen for video-watched event
    listen<{ video_id: string; is_watched: boolean }>('video-watched', (event) => {
      const { video_id, is_watched } = event.payload
      console.log(`[${isPip ? 'PiP' : 'Main'}] Received video-watched event:`, video_id, is_watched)
      onVideoWatched(video_id, is_watched)
    }).then((unlisten) => {
      unlistenVideoWatched = unlisten
    })

    // Listen for active-playback-cleared event (CRITICAL: This fixes the PIP reset bug)
    listen('active-playback-cleared', () => {
      console.log(`[${isPip ? 'PiP' : 'Main'}] Received active-playback-cleared event`)
      onActivePlaybackCleared()
    }).then((unlisten) => {
      unlistenActivePlaybackCleared = unlisten
    })

    // Return cleanup function
    return () => {
      if (unlistenVideoSelected) unlistenVideoSelected()
      if (unlistenPlaybackVideoUpdated) unlistenPlaybackVideoUpdated()
      if (unlistenPlaybackSettings) unlistenPlaybackSettings()
      if (unlistenVideoWatched) unlistenVideoWatched()
      if (unlistenActivePlaybackCleared) unlistenActivePlaybackCleared()
    }
  }

  return {
    setupEventListeners,
  }
}
