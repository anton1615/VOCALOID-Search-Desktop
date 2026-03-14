<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Video } from '../api/tauri-commands'
import VideoMetaPanel from './VideoMetaPanel.vue'
import PlayerControls from './PlayerControls.vue'
import { usePlayerCore } from '../composables/usePlayerCore'
import { formatDateTime } from '../utils/dateTime'
import { getPlayerColumnLayout } from '../features/playlistViews/playerColumnLayout'
import { getPipLayout } from '../features/playlistViews/pipLayout'

const props = withDefaults(defineProps<{
  /** Display mode: full for main window, compact for PIP */
  mode: 'full' | 'compact'
  /** Current video */
  currentVideo: Video | null
  /** Current video index */
  currentVideoIndex: number
  /** Total results count */
  resultsCount: number
  /** Whether there is a next video */
  hasNext: boolean
  /** Whether PIP is active (only for full mode) */
  pipActive?: boolean
  /** Whether to show auto-skip settings */
  showAutoSkip?: boolean
  /** Whether to set up event listeners (true for PIP, false for main window) */
  setupEvents?: boolean
}>(), {
  showAutoSkip: true,
  pipActive: false,
  setupEvents: false,
})

const emit = defineEmits<{
  (e: 'playNext'): void
  (e: 'playPrevious'): void
  (e: 'openPip'): void
  (e: 'closePip'): void
  (e: 'videoWatched', video: Video): void
  (e: 'stateCleared'): void
}>()

const { t } = useI18n()

// Determine if we're in compact mode
const isCompact = computed(() => props.mode === 'compact')

// Get layout based on mode
const layoutSections = computed(() => {
  return isCompact.value ? getPipLayout().content : getPlayerColumnLayout()
})

// Player core state management
const playerCore = usePlayerCore({
  onPlayNext: () => emit('playNext'),
  onPlayPrevious: () => emit('playPrevious'),
  onMarkWatched: (video) => {
    emit('videoWatched', { ...video, is_watched: true })
  },
  onStateCleared: () => {
    emit('stateCleared')
  },
  isPip: isCompact.value,
  setupEvents: props.setupEvents,
})

// Local state
const playbackSettingsOpen = ref(false)

// Computed values
const isPlaying = computed(() => playerCore.isPlaying.value)
const playerReady = computed(() => playerCore.playerReady.value)

// Watch for video changes from props
watch(() => props.currentVideo, async (video, oldVideo) => {
  if (video?.id !== oldVideo?.id) {
    await playerCore.handleVideoChange(video, props.currentVideoIndex, props.hasNext)
  }
}, { immediate: true })

// Watch for index changes
watch(() => props.currentVideoIndex, (index) => {
  playerCore.updateIndex(index, props.hasNext)
})

// Watch for hasNext changes
watch(() => props.hasNext, (hasNext) => {
  playerCore.updateHasNext(hasNext)
})

// Handle player events from iframe
function handleMessage(event: MessageEvent) {
  playerCore.handlePlayerMessage(event)
}

// Toggle play/pause
function togglePlayPause() {
  playerCore.togglePlayPause()
}

// Toggle settings panel
function toggleSettingsPanel() {
  playbackSettingsOpen.value = !playbackSettingsOpen.value
}

// Get user nickname for display
function getUserNickname(): string {
  return playerCore.getUserNickname()
}

// Get user icon URL for display
function getUserIconUrl(): string | null {
  return playerCore.getUserIconUrl()
}

// Lifecycle
let eventCleanup: (() => void) | null = null

onMounted(async () => {
  console.log('[UnifiedPlayer] onMounted, mode:', props.mode, 'currentVideo:', props.currentVideo?.id)
  window.addEventListener('message', handleMessage)
  await playerCore.loadSettings()
  playerCore.updatePlaybackSettings({
    autoPlay: playerCore.autoPlay.value,
    autoSkip: playerCore.autoSkip.value,
    skipThreshold: playerCore.skipThreshold.value,
  })
  
  // Set up event listeners if needed (for PIP window)
  if (props.setupEvents) {
    eventCleanup = playerCore.setupEventListeners()
  }
  console.log('[UnifiedPlayer] onMounted complete, isCompact:', isCompact.value)
})

onUnmounted(() => {
  window.removeEventListener('message', handleMessage)
  if (eventCleanup) {
    eventCleanup()
  }
})

// Expose methods for parent components
defineExpose({
  reset: () => {
    playerCore.resetState()
  },
})
</script>

<template>
  <div class="unified-player" :class="{ compact: isCompact, 'pip-active': pipActive }">
    <!-- PIP Placeholder (only in full mode) -->
    <div v-if="pipActive && !isCompact" class="pip-placeholder">
      <p>{{ t('player.pipActive') }}</p>
      <button @click="$emit('closePip')">{{ t('player.returnToMain') }}</button>
    </div>

    <template v-else>
      <!-- Compact mode: sidebar + content -->
      <template v-if="isCompact">
        <PlayerControls
          layout="vertical"
          :video-id="currentVideo?.id || null"
          :video-title="currentVideo?.title || undefined"
          :thumbnail-url="currentVideo?.thumbnail_url || undefined"
          :player-ready="playerReady"
          :is-playing="isPlaying"
          :current-index="currentVideoIndex"
          :results-count="resultsCount"
          :has-next="hasNext"
          @toggle-play-pause="togglePlayPause"
          @play-next="$emit('playNext')"
          @play-previous="$emit('playPrevious')"
        />

        <div class="player-content">
          <div v-if="!currentVideo" class="empty-state">
            <span>{{ t('player.selectVideo') }}</span>
          </div>

          <template v-else>
            <template v-for="section in layoutSections" :key="section.section">
              <VideoMetaPanel
                v-if="section.section === 'header' || section.section === 'details'"
                :video="currentVideo"
                :uploader-name="getUserNickname()"
                :uploader-icon-url="getUserIconUrl()"
                :upload-date-time="formatDateTime(currentVideo.start_time)"
                :display-mode="section.videoMetaPanelMode"
              />

              <div v-else class="video-container">
                <div class="aspect-ratio-box">
                  <iframe
                    :ref="(el) => playerCore.setIframeRef(el as HTMLIFrameElement | null)"
                    :src="`https://embed.nicovideo.jp/watch/${currentVideo.id}?jsapi=1&playerId=1`"
                    frameborder="0"
                    allow="autoplay; encrypted-media"
                    allowfullscreen
                  ></iframe>
                </div>
              </div>
            </template>
          </template>
        </div>
      </template>

      <!-- Full mode: standard layout -->
      <template v-else>
        <template v-for="section in layoutSections" :key="section.section">
          <VideoMetaPanel
            v-if="currentVideo && (section.section === 'header' || section.section === 'details')"
            :video="currentVideo"
            :uploader-name="getUserNickname()"
            :uploader-icon-url="getUserIconUrl()"
            :upload-date-time="formatDateTime(currentVideo.start_time)"
            :collapse-label="t('player.collapse')"
            :expand-label="t('player.expand')"
            :show-uploader-placeholder="true"
            :display-mode="section.videoMetaPanelMode"
          />

          <!-- Video Container -->
          <div v-else-if="section.section === 'player'" class="video-container">
            <div class="aspect-ratio-box">
              <div v-if="!currentVideo" class="empty-player">
                <span>{{ t('player.selectVideo') }}</span>
              </div>
              <iframe
                v-else
                :ref="(el) => playerCore.setIframeRef(el as HTMLIFrameElement | null)"
                :src="`https://embed.nicovideo.jp/watch/${currentVideo.id}?jsapi=1&playerId=1`"
                frameborder="0"
                allow="autoplay; encrypted-media"
                allowfullscreen
              ></iframe>
            </div>
          </div>

          <!-- Playback Controls -->
          <div v-else-if="section.section === 'controls'" class="controls-section">
            <PlayerControls
              layout="horizontal"
              :video-id="currentVideo?.id || null"
              :video-title="currentVideo?.title || undefined"
              :thumbnail-url="currentVideo?.thumbnail_url || undefined"
              :player-ready="playerReady"
              :is-playing="isPlaying"
              :current-index="currentVideoIndex"
              :results-count="resultsCount"
              :has-next="hasNext"
              :show-auto-skip="showAutoSkip"
              :auto-play="playerCore.autoPlay.value"
              :auto-skip="playerCore.autoSkip.value"
              :settings-panel-open="playbackSettingsOpen"
              @toggle-play-pause="togglePlayPause"
              @play-next="$emit('playNext')"
              @play-previous="$emit('playPrevious')"
              @toggle-settings="toggleSettingsPanel"
              @update:auto-play="playerCore.autoPlay.value = $event"
              @update:auto-skip="playerCore.autoSkip.value = $event"
            />

            <!-- PIP Button -->
            <button
              class="icon-btn pip-btn"
              :disabled="!currentVideo"
              title="PiP"
              @click="$emit('openPip')"
            >
              📺
            </button>
          </div>
        </template>
      </template>
    </template>
  </div>
</template>

<style scoped>
.unified-player {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--color-bg-surface);
  overflow-y: auto;
  overflow-x: hidden;
}

.unified-player.compact {
  flex-direction: row;
}

.pip-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  gap: var(--space-md);
}

.pip-placeholder button {
  padding: var(--space-sm) var(--space-md);
  border-radius: 6px;
  background: var(--color-accent-primary);
  color: white;
  border: none;
  cursor: pointer;
}

.player-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--color-bg-surface);
  overflow-y: auto;
  overflow-x: hidden;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.video-container {
  flex-shrink: 0;
  background: #000;
}

.aspect-ratio-box {
  position: relative;
  width: 100%;
  padding-top: 56.25%;
}

.aspect-ratio-box iframe,
.aspect-ratio-box .empty-player {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  border: none;
}

.empty-player {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-primary);
  color: var(--color-text-muted);
}

.controls-section {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-md);
  padding: var(--space-sm) var(--space-md);
  border-bottom: 1px solid var(--color-border-subtle);
  background: var(--color-bg-surface);
}

.icon-btn {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: var(--color-text-primary);
  background: transparent;
  transition: all 0.2s;
  border: 1px solid transparent;
  cursor: pointer;
}

.icon-btn:hover:not(:disabled) {
  background: var(--color-bg-hover);
  border-color: var(--color-border-subtle);
}

.icon-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.pip-btn {
  /* PIP button styling */
}
</style>
