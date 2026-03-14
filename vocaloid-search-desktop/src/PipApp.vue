<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { api, type Video, type VideoSelectedPayload, type PlaybackSettings } from './api/tauri-commands'
import UnifiedPlayer from './components/UnifiedPlayer.vue'

// Current video state
const currentVideo = ref<Video | null>(null)
const currentIndex = ref(-1)
const resultsCount = ref(0)
const hasNext = ref(false)

// Reference to the unified player
const unifiedPlayerRef = ref<InstanceType<typeof UnifiedPlayer> | null>(null)

// PIP window management
const pipWindow = getCurrentWindow()
let isClosing = false

// Event listeners
let unlistenVideoSelected: (() => void) | null = null
let unlistenPlaybackSettings: (() => void) | null = null
let unlistenVideoWatched: (() => void) | null = null
let unlistenSearchResultsUpdated: (() => void) | null = null
let unlistenActivePlaybackCleared: (() => void) | null = null
let unlistenResize: (() => void) | null = null
let unlistenMove: (() => void) | null = null
let unlistenClose: (() => void) | null = null

/**
 * Save PIP window state
 */
async function saveWindowState() {
  try {
    const position = await pipWindow.outerPosition()
    const size = await pipWindow.innerSize()
    await api.savePipWindowState({
      x: position.x,
      y: position.y,
      width: size.width,
      height: size.height
    })
  } catch (e) {
    console.error('[PiP] Failed to save window state:', e)
  }
}

/**
 * Handle video change from video-selected event
 */
async function handleVideoChange(video: Video, index: number, hasNextVideo: boolean) {
  console.log('[PiP] handleVideoChange called:', video.id, 'index:', index)
  currentVideo.value = video
  currentIndex.value = index
  hasNext.value = hasNextVideo
  try {
    const state = await api.getPlaylistState()
    resultsCount.value = state.results.length
  } catch (e) {
    console.error('[PiP] Failed to get playlist state:', e)
  }
}

/**
 * Play next video
 */
async function playNext() {
  if (currentIndex.value >= 0) {
    // Check if we need to preload more results
    try {
      const playlistState = await api.getPlaylistState()
      const remaining = playlistState.results.length - currentIndex.value - 1
      
      const { canPreloadSearchResults } = await import('./features/playlistViews/searchViewInteractions')
      const canPreload = canPreloadSearchResults(playlistState.playlist_type)

      if (remaining <= 5 && hasNext.value && canPreload) {
        try {
          const searchState = await api.getSearchState()
          if (searchState.has_next) {
            console.log('[PiP] Preloading more results... (remaining:', remaining, ')')
            await api.loadMore('Search', searchState.version)
          }
        } catch (e) {
          console.error('[PiP] Preload failed:', e)
        }
      } else if (!hasNext.value && canPreload) {
        try {
          const searchState = await api.getSearchState()
          if (searchState.has_next) {
            console.log('[PiP] At end, loading more...')
            await api.loadMore('Search', searchState.version)
            const newPlaylistState = await api.getPlaylistState()
            if (newPlaylistState.index !== null && newPlaylistState.index + 1 < newPlaylistState.results.length) {
              hasNext.value = true
            }
          }
        } catch (e) {
          console.error('[PiP] loadMore failed:', e)
        }
      }
      
      if (hasNext.value) {
        await api.setPlaylistIndex(currentIndex.value + 1)
      }
    } catch (e) {
      console.error('[PiP] playNext failed:', e)
    }
  }
}

/**
 * Play previous video
 */
async function playPrevious() {
  if (currentIndex.value > 0) {
    await api.setPlaylistIndex(currentIndex.value - 1)
  }
}

/**
 * Handle video watched event
 */
function handleVideoWatched(video: Video) {
  if (currentVideo.value?.id === video.id) {
    currentVideo.value = video
  }
}

/**
 * Handle state cleared event (from active-playback-cleared)
 */
function handleStateCleared() {
  console.log('[PiP] State cleared, resetting player')
  currentVideo.value = null
  currentIndex.value = -1
  hasNext.value = false
  resultsCount.value = 0
}

// Lifecycle
onMounted(async () => {
  console.log('[PiP] onMounted start')

  // Get initial state
  try {
    const state = await api.getPlaylistState()
    console.log('[PiP] getPlaylistState result:', state.results.length, 'videos, index:', state.index, 'version:', state.playlist_version)
    
    if (state.results.length > 0 && state.index !== null && state.index >= 0 && state.index < state.results.length) {
      currentVideo.value = state.results[state.index]
      currentIndex.value = state.index
      hasNext.value = state.has_next
      resultsCount.value = state.results.length
      console.log('[PiP] Initial video set:', currentVideo.value?.id)
    } else {
      console.log('[PiP] No initial video to set')
    }
  } catch (e) {
    console.error('[PiP] Failed to get initial playlist state:', e)
  }

  // Listen for video-selected event
  unlistenVideoSelected = await listen<VideoSelectedPayload>('video-selected', async (event) => {
    const payload = event.payload
    console.log('[PiP] Received video-selected event:', payload.video.id, 'index:', payload.index, 'version:', payload.playlist_version)
    
    // Get the latest playlist state to check version
    try {
      const latestPlaylistState = await api.getPlaylistState()
      console.log('[PiP] Latest playlist version:', latestPlaylistState.playlist_version)
      
      // Only accept events that match the current playlist
      // Note: We use a more lenient check - if the video is in the current results, accept it
      if (payload.playlist_type !== latestPlaylistState.playlist_type) {
        console.log('[PiP] Ignoring event - playlist type mismatch')
        return
      }
      
      // Accept the event if the version matches or if the index is valid in current results
      if (payload.playlist_version !== latestPlaylistState.playlist_version) {
        console.log('[PiP] Version mismatch, but checking if index is valid')
        // If the index is valid in the current results, still accept it
        if (payload.index >= 0 && payload.index < latestPlaylistState.results.length) {
          console.log('[PiP] Index is valid, accepting event')
        } else {
          console.log('[PiP] Index is invalid, ignoring event')
          return
        }
      }
      
      await handleVideoChange(payload.video, payload.index, payload.has_next)
    } catch (e) {
      console.error('[PiP] Error processing video-selected event:', e)
      // Still try to handle the video change
      await handleVideoChange(payload.video, payload.index, payload.has_next)
    }
  })

  // Listen for playback-settings-changed event
  unlistenPlaybackSettings = await listen<PlaybackSettings>('playback-settings-changed', () => {
    // Settings are handled by the UnifiedPlayer's usePlayerSettings composable
  })

  // Listen for video-watched event
  unlistenVideoWatched = await listen<{ video_id: string; is_watched: boolean }>('video-watched', (event) => {
    const { video_id, is_watched } = event.payload
    console.log('[PiP] Received video-watched event:', video_id, is_watched)
    if (currentVideo.value?.id === video_id) {
      currentVideo.value = { ...currentVideo.value, is_watched }
    }
  })

  // Listen for search results updates from main window
  unlistenSearchResultsUpdated = await listen('search-results-updated', async () => {
    console.log('[PiP] Received search-results-updated event')
    try {
      const playlistState = await api.getPlaylistState()
      resultsCount.value = playlistState.results.length
      if (currentIndex.value >= 0 && currentIndex.value < playlistState.results.length) {
        hasNext.value = currentIndex.value + 1 < playlistState.results.length
      }
    } catch (e) {
      console.error('[PiP] Failed to handle search-results-updated:', e)
    }
  })

  // CRITICAL: Listen for active-playback-cleared event
  // This fixes the bug where PIP window doesn't reset when search conditions change
  unlistenActivePlaybackCleared = await listen('active-playback-cleared', () => {
    console.log('[PiP] Received active-playback-cleared event')
    handleStateCleared()
  })

  // Window event handlers
  unlistenResize = await pipWindow.onResized(async () => {
    await saveWindowState()
  })

  unlistenMove = await pipWindow.onMoved(async () => {
    await saveWindowState()
  })

  unlistenClose = await pipWindow.onCloseRequested(async (event) => {
    if (isClosing) {
      console.log('[PiP] already closing, returning')
      return
    }
    isClosing = true
    console.log('[PiP] onCloseRequested triggered')
    event.preventDefault()
    console.log('[PiP] prevented default, saving state...')
    await saveWindowState()
    console.log('[PiP] saveWindowState done, notifying backend...')
    try {
      await api.notifyPipClosing()
      console.log('[PiP] notifyPipClosing done, now closing window')
    } catch (e) {
      console.error('[PiP] notifyPipClosing error:', e)
    }
    await pipWindow.close()
    console.log('[PiP] window closed')
  })
  
  console.log('[PiP] onMounted complete')
})

onUnmounted(() => {
  if (unlistenVideoSelected) unlistenVideoSelected()
  if (unlistenPlaybackSettings) unlistenPlaybackSettings()
  if (unlistenVideoWatched) unlistenVideoWatched()
  if (unlistenSearchResultsUpdated) unlistenSearchResultsUpdated()
  if (unlistenActivePlaybackCleared) unlistenActivePlaybackCleared()
  if (unlistenResize) unlistenResize()
  if (unlistenMove) unlistenMove()
  if (unlistenClose) unlistenClose()
})
</script>

<template>
  <div class="pip-container">
    <UnifiedPlayer
      ref="unifiedPlayerRef"
      mode="compact"
      :current-video="currentVideo"
      :current-video-index="currentIndex"
      :results-count="resultsCount"
      :has-next="hasNext"
      :show-auto-skip="false"
      :setup-events="false"
      @play-next="playNext"
      @play-previous="playPrevious"
      @video-watched="handleVideoWatched"
      @state-cleared="handleStateCleared"
    />
  </div>
</template>

<style scoped>
.pip-container {
  display: flex;
  height: 100vh;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
}
</style>
