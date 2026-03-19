<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'

import { api, type Video } from './api/tauri-commands'
import UnifiedPlayer from './components/UnifiedPlayer.vue'
import { useAuthoritativePlaybackSync } from './composables/useAuthoritativePlaybackSync'

const currentVideo = ref<Video | null>(null)
const currentIndex = ref(-1)
const resultsCount = ref(0)
const hasNext = ref(false)

const pipWindow = getCurrentWindow()
let isClosing = false

function syncPipPlayback(state: {
  currentVideo: Video | null
  currentVideoIndex: number
  resultsCount: number
  hasNext: boolean
}) {
  currentVideo.value = state.currentVideo
  currentIndex.value = state.currentVideoIndex
  resultsCount.value = state.resultsCount
  hasNext.value = state.hasNext
}

const { refreshActivePlayback } = useAuthoritativePlaybackSync(syncPipPlayback)

async function saveWindowState() {
  try {
    const position = await pipWindow.outerPosition()
    const size = await pipWindow.innerSize()
    await api.savePipWindowState({
      x: position.x,
      y: position.y,
      width: size.width,
      height: size.height,
    })
  } catch (e) {
    console.error('[PiP] Failed to save window state:', e)
  }
}

async function playNext() {
  if (currentIndex.value < 0) return

  try {
    const playlistState = await api.getPlaylistState()
    const remaining = playlistState.results.length - currentIndex.value - 1
    const { canPreloadSearchResults } = await import('./features/playlistViews/searchViewInteractions')
    const canPreload = canPreloadSearchResults(playlistState.playlist_type)

    if (remaining <= 5 && hasNext.value && canPreload) {
      try {
        const searchState = await api.getSearchState()
        if (searchState.loading) {
          console.log('[PiP] Blocked preload: search in progress')
        } else if (searchState.has_next) {
          console.log('[PiP] Preloading more results... (remaining:', remaining, ')')
          await api.loadMore('Search', searchState.version)
          await refreshActivePlayback()
        }
      } catch (e) {
        console.error('[PiP] Preload failed:', e)
      }
    } else if (!hasNext.value && canPreload) {
      try {
        const searchState = await api.getSearchState()
        if (searchState.loading) {
          console.log('[PiP] Blocked loadMore: search in progress')
        } else if (searchState.has_next) {
          console.log('[PiP] At end, loading more...')
          await api.loadMore('Search', searchState.version)
          await refreshActivePlayback()
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

async function playPrevious() {
  if (currentIndex.value > 0) {
    await api.setPlaylistIndex(currentIndex.value - 1)
  }
}

function handleVideoWatched(video: Video) {
  if (currentVideo.value?.id === video.id) {
    currentVideo.value = video
  }
}

function handlePlaybackStateChanged() {
  void refreshActivePlayback()
}

function handleStateCleared() {
  // Shared player path notifies via playbackStateChanged; refresh handles the authoritative state.
}

let unlistenSearchResultsUpdated: (() => void) | null = null
let unlistenResize: (() => void) | null = null
let unlistenMove: (() => void) | null = null
let unlistenClose: (() => void) | null = null

onMounted(async () => {
  await refreshActivePlayback()

  unlistenSearchResultsUpdated = await listen('search-results-updated', async () => {
    console.log('[PiP] Received search-results-updated event')
    await refreshActivePlayback()
  })

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
})

onUnmounted(() => {
  if (unlistenSearchResultsUpdated) unlistenSearchResultsUpdated()
  if (unlistenResize) unlistenResize()
  if (unlistenMove) unlistenMove()
  if (unlistenClose) unlistenClose()
})
</script>

<template>
  <div class="pip-container">
    <UnifiedPlayer
      mode="compact"
      :current-video="currentVideo"
      :current-video-index="currentIndex"
      :results-count="resultsCount"
      :has-next="hasNext"
      :show-auto-skip="false"
      :setup-events="true"
      @play-next="playNext"
      @play-previous="playPrevious"
      @video-watched="handleVideoWatched"
      @state-cleared="handleStateCleared"
      @playback-state-changed="handlePlaybackStateChanged"
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
