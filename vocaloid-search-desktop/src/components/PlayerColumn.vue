<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import type { Video } from '../api/tauri-commands'
import UnifiedPlayer from './UnifiedPlayer.vue'

defineProps<{
  currentVideo: Video | null
  currentVideoIndex: number
  resultsCount: number
  hasNext: boolean
  pipActive: boolean
  showAutoSkip?: boolean
}>()

const emit = defineEmits<{
  (e: 'playNext'): void
  (e: 'playPrevious'): void
  (e: 'openPip'): void
  (e: 'closePip'): void
  (e: 'videoWatched', video: Video): void
}>()

// Reference to the unified player for programmatic control
const unifiedPlayerRef = ref<InstanceType<typeof UnifiedPlayer> | null>(null)

// Event listeners
let unlistenVideoWatched: (() => void) | null = null

// Handle video watched event from UnifiedPlayer
function handleVideoWatched(video: Video) {
  emit('videoWatched', video)
}

// Handle state cleared event from UnifiedPlayer
function handleStateCleared() {
  // The parent (App.vue) handles the actual state reset
  // This is just for any local cleanup if needed
}

// Lifecycle
onMounted(async () => {
  // Listen for video-watched event (for local state sync if needed)
  unlistenVideoWatched = await listen<{ video_id: string; is_watched: boolean }>('video-watched', () => {
    // Update handled by parent
  })
})

onUnmounted(() => {
  if (unlistenVideoWatched) unlistenVideoWatched()
})
</script>

<template>
  <div class="player-column">
    <UnifiedPlayer
      ref="unifiedPlayerRef"
      mode="full"
      :current-video="currentVideo"
      :current-video-index="currentVideoIndex"
      :results-count="resultsCount"
      :has-next="hasNext"
      :pip-active="pipActive"
      :show-auto-skip="showAutoSkip"
      :setup-events="false"
      @play-next="$emit('playNext')"
      @play-previous="$emit('playPrevious')"
      @open-pip="$emit('openPip')"
      @close-pip="$emit('closePip')"
      @video-watched="handleVideoWatched"
      @state-cleared="handleStateCleared"
    />
  </div>
</template>

<style scoped>
.player-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--color-bg-surface);
  overflow: hidden;
}
</style>
