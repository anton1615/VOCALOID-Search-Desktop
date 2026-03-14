<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { api } from '../api/tauri-commands'

const props = defineProps<{
  videoId: string | null
  videoTitle?: string
  thumbnailUrl?: string | null
  disabled?: boolean
}>()

const isInList = ref(false)
const loading = ref(false)

async function checkStatus() {
  if (!props.videoId) {
    isInList.value = false
    return
  }
  try {
    isInList.value = await api.isInWatchLater(props.videoId)
  } catch (e) {
    console.error('Failed to check watch later status:', e)
  }
}

async function toggle() {
  if (!props.videoId || loading.value) return
  
  loading.value = true
  try {
    if (isInList.value) {
      await api.removeFromWatchLater(props.videoId)
      isInList.value = false
    } else {
      await api.addToWatchLater(props.videoId, props.videoTitle || '', props.thumbnailUrl || null)
      isInList.value = true
    }
  } catch (e) {
    console.error('Failed to toggle watch later:', e)
  } finally {
    loading.value = false
  }
}

// Listen for watch-later-changed events to sync state
let unlisten: (() => void) | null = null

onMounted(async () => {
  await checkStatus()
  unlisten = await listen<string>('watch-later-changed', (event) => {
    if (event.payload === props.videoId) {
      checkStatus()
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})

// Watch for videoId changes
watch(() => props.videoId, () => {
  checkStatus()
})
</script>

<template>
  <button 
    class="watch-later-btn icon-btn" 
    :disabled="disabled || !videoId || loading"
    @click="toggle"
    :title="isInList ? 'Remove from Watch Later' : 'Add to Watch Later'"
  >
    <span v-if="loading" class="loading-spinner">⋯</span>
    <svg v-else class="heart-icon" :class="{ filled: isInList }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
    </svg>
  </button>
</template>

<style scoped>
.watch-later-btn {
  font-size: 1.2em;
}

.watch-later-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.heart-icon {
  width: 20px;
  height: 20px;
  stroke: var(--color-text-muted);
  transition: all 0.2s ease;
}

.heart-icon.filled {
  fill: #ef4444;
  stroke: #ef4444;
}

.heart-icon:not(.filled):hover {
  stroke: var(--color-accent-primary);
}

.loading-spinner {
  display: inline-block;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
