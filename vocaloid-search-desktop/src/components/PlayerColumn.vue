<script setup lang="ts">
import { ref } from 'vue'
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
  (e: 'playbackStateChanged'): void
}>()

const unifiedPlayerRef = ref<InstanceType<typeof UnifiedPlayer> | null>(null)

function handlePlaybackStateChanged() {
  emit('playbackStateChanged')
}

function handleVideoWatched(video: Video) {
  emit('videoWatched', video)
}

function handleStateCleared() {
  // Parent refreshes authoritative playback state.
}
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
      :setup-events="true"
      @play-next="$emit('playNext')"
      @play-previous="$emit('playPrevious')"
      @open-pip="$emit('openPip')"
      @close-pip="$emit('closePip')"
      @video-watched="handleVideoWatched"
      @state-cleared="handleStateCleared"
      @playback-state-changed="handlePlaybackStateChanged"
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
