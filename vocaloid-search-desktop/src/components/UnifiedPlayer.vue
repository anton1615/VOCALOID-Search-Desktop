<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { api, type PlaybackIdentityPayload, type PlaylistType, type Video } from '../api/tauri-commands'
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
  /** Playback identity type for metadata update filtering */
  playlistType: PlaylistType
  /** Playback identity version for metadata update filtering */
  playlistVersion: number
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
  (e: 'playbackStateChanged'): void
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
    api.markWatched(video.id, video.title, video.thumbnail_url)
    emit('videoWatched', { ...video, is_watched: true })
  },
  onStateCleared: () => {
    emit('stateCleared')
  },
  onPlaybackStateChanged: () => {
    emit('playbackStateChanged')
  },
  getPlaybackIdentity: (): PlaybackIdentityPayload => ({
    playlistType: props.playlistType,
    playlistVersion: props.playlistVersion,
    currentIndex: props.currentVideoIndex,
    videoId: props.currentVideo?.id ?? null,
  }),
  isPip: isCompact.value,
  setupEvents: props.setupEvents,
})

// Local state
const playbackSettingsOpen = ref(false)

// Computed values
const currentVideo = computed(() => playerCore.currentVideo.value)
const currentVideoIndex = computed(() => playerCore.currentIndex.value)
const hasNextVideo = computed(() => playerCore.hasNext.value)
const metadataReady = computed(() => playerCore.metadataReady.value)
const isPlaying = computed(() => playerCore.isPlaying.value)
const playerReady = computed(() => playerCore.playerReady.value)
const playbackSessionKey = computed(() => playerCore.playbackSessionKey.value)

// Watch for authoritative playback identity changes from props
watch(
  () => [
    props.playlistType,
    props.playlistVersion,
    props.currentVideoIndex,
    props.currentVideo?.id ?? null,
    props.hasNext,
  ] as const,
  async () => {
    await playerCore.handleVideoChange(props.currentVideo, props.currentVideoIndex, props.hasNext)
  },
  { immediate: true },
)

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
          :has-next="hasNextVideo"
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
              <div
                v-if="section.section === 'header'"
                data-shell="header"
                class="player-shell player-shell-header"
                :class="{ 'player-shell-pending': !metadataReady, 'player-shell-header-pending-compact': !metadataReady && isCompact }"
              >
                <div v-if="!metadataReady && isCompact" class="player-shell-header-frame player-shell-header-frame-compact">
                  <div class="player-shell-header-frame-title player-shell-header-frame-title-compact"></div>
                  <div class="player-shell-header-frame-meta player-shell-header-frame-meta-compact"></div>
                </div>
                <VideoMetaPanel
                  v-if="metadataReady"
                  :video="currentVideo"
                  :uploader-name="getUserNickname()"
                  :uploader-icon-url="getUserIconUrl()"
                  :upload-date-time="formatDateTime(currentVideo.start_time)"
                  :display-mode="section.videoMetaPanelMode"
                  :presentation-mode="isCompact ? 'compact' : 'full'"
                />
              </div>

              <div
                v-else-if="section.section === 'details'"
                data-shell="details"
                class="player-shell player-shell-details"
                :class="{ 'player-shell-pending': !metadataReady }"
              >
                <VideoMetaPanel
                  v-if="metadataReady"
                  :video="currentVideo"
                  :uploader-name="getUserNickname()"
                  :uploader-icon-url="getUserIconUrl()"
                  :upload-date-time="formatDateTime(currentVideo.start_time)"
                  :display-mode="section.videoMetaPanelMode"
                  :presentation-mode="isCompact ? 'compact' : 'full'"
                />
              </div>

              <div v-else-if="section.section === 'player'" class="video-container">
                <div class="aspect-ratio-box" :key="playbackSessionKey">
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
          <div
            v-if="currentVideo && section.section === 'header'"
            data-shell="header"
            class="player-shell player-shell-header"
            :class="{ 'player-shell-pending': !metadataReady, 'player-shell-header-pending-full': !metadataReady && !isCompact }"
          >
            <div v-if="!metadataReady && !isCompact" class="player-shell-header-frame">
              <div class="player-shell-header-frame-title"></div>
              <div class="player-shell-header-frame-meta"></div>
            </div>
            <VideoMetaPanel
              v-if="metadataReady"
              :video="currentVideo"
              :uploader-name="getUserNickname()"
              :uploader-icon-url="getUserIconUrl()"
              :upload-date-time="formatDateTime(currentVideo.start_time)"
              :collapse-label="t('player.collapse')"
              :expand-label="t('player.expand')"
              :show-uploader-placeholder="true"
              :display-mode="section.videoMetaPanelMode"
              :presentation-mode="isCompact ? 'compact' : 'full'"
            />
          </div>

          <div
            v-else-if="currentVideo && section.section === 'details'"
            data-shell="details"
            class="player-shell player-shell-details"
            :class="{ 'player-shell-pending': !metadataReady }"
          >
            <VideoMetaPanel
              v-if="metadataReady"
              :video="currentVideo"
              :uploader-name="getUserNickname()"
              :uploader-icon-url="getUserIconUrl()"
              :upload-date-time="formatDateTime(currentVideo.start_time)"
              :collapse-label="t('player.collapse')"
              :expand-label="t('player.expand')"
              :show-uploader-placeholder="true"
              :display-mode="section.videoMetaPanelMode"
              :presentation-mode="isCompact ? 'compact' : 'full'"
            />
          </div>

          <!-- Video Container -->
          <div v-else-if="section.section === 'player'" class="video-container">
            <div class="aspect-ratio-box" :key="playbackSessionKey">
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
              :has-next="hasNextVideo"
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

.player-shell {
  flex-shrink: 0;
  min-width: 0;
}

.player-shell-header-pending-full {
  min-height: 86px;
}

.player-shell-header-pending-compact {
  min-height: 64px;
}

.player-shell-header-frame {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  padding: var(--space-md);
  border-bottom: 1px solid var(--color-border-subtle);
}

.player-shell-header-frame-compact {
  padding: var(--space-sm) var(--space-md);
  gap: var(--space-xs);
}

.player-shell-header-frame-title {
  min-height: calc(1.4em * 1);
}

.player-shell-header-frame-title-compact {
  min-height: calc(1.4em * 2);
}

.player-shell-header-frame-meta {
  min-height: 32px;
}

.player-shell-header-frame-meta-compact {
  min-height: 24px;
}

.player-shell-details {
  min-height: 112px;
}

.player-shell-pending {
  background: var(--color-bg-surface);
}

.player-shell-header-frame-title,
.player-shell-header-frame-meta {
  background: transparent;
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
