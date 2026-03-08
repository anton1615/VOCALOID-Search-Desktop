<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { api, type Video, type UserInfo, type PlaybackSettings, getUploaderAvatarUrl } from '../api/tauri-commands'
import WatchLaterButton from './WatchLaterButton.vue'
import VideoMetaPanel from './VideoMetaPanel.vue'
import { formatDateTime } from '../utils/dateTime'
import { createEmbeddedPlayerController } from '../features/playlistViews/embeddedPlayerController'
import { getPlayerColumnLayout } from '../features/playlistViews/playerColumnLayout'
import {
  createMainWindowPlaybackSettingsViewModel,
  updatePlaybackSettings,
} from '../features/playlistViews/playerPlaybackSettings'
import { resolvePlayerCommandTarget } from '../features/playlistViews/playerCommandTarget'
import { clearPlayerMessageSource, rememberPlayerMessageSource, type PostMessageTarget } from '../features/playlistViews/playerMessageSource'

const props = withDefaults(defineProps<{
  currentVideo: Video | null
  currentVideoIndex: number
  resultsCount: number
  hasNext: boolean
  pipActive: boolean
  showAutoSkip?: boolean
}>(), {
  showAutoSkip: true
})

const emit = defineEmits<{
  (e: 'playNext'): void
  (e: 'playPrevious'): void
  (e: 'openPip'): void
  (e: 'closePip'): void
  (e: 'videoWatched', video: Video): void
}>()

const { t } = useI18n()

const iframeRef = ref<HTMLIFrameElement | null>(null)
let lastPlayerMessageSource: PostMessageTarget | null = null
const autoPlay = ref(localStorage.getItem('vocaloidAutoPlay') !== 'false')
const autoSkip = ref(localStorage.getItem('vocaloidAutoSkip') === 'true')
const skipThreshold = ref(parseInt(localStorage.getItem('vocaloidSkipThreshold') || '30', 10))
const playbackSettingsOpen = ref(false)
const syncingPlaybackSettings = ref(false)

const playerController = createEmbeddedPlayerController({
  sendCommand: (command) => sendCommand(command),
  onPlayNext: () => emit('playNext'),
  onMarkWatched: (video) => {
    api.markWatched(video.id, video.title, video.thumbnail_url)
    emit('videoWatched', { ...video, is_watched: true })
  },
  schedule: (callback) => setTimeout(callback, 500),
})

const isPlaying = ref(playerController.state.isPlaying)
const playerReady = ref(playerController.state.playerReady)

const userInfoCache = ref(new Map<string, UserInfo>())
const currentUserInfo = ref<UserInfo | null>(null)
const layoutSections = computed(() => getPlayerColumnLayout())
const playbackSettingsViewModel = computed(() => createMainWindowPlaybackSettingsViewModel(playbackSettingsOpen.value))

// Watch for currentVideo changes to fetch user info and reset states
watch(() => props.currentVideo, async (video, oldVideo) => {
  if (video?.id !== oldVideo?.id) {
    lastPlayerMessageSource = clearPlayerMessageSource(lastPlayerMessageSource)
    playerController.setCurrentVideo(video ?? null)
    playerReady.value = playerController.state.playerReady
    isPlaying.value = playerController.state.isPlaying
  }

  if (video?.uploader_id) {
    if (!userInfoCache.value.has(video.id)) {
      try {
        const userInfo = await api.getUserInfo(video.id)
        if (userInfo) {
          userInfoCache.value.set(video.id, userInfo)
        }
      } catch (e) {
        console.error('Failed to fetch user info:', e)
      }
    }
    currentUserInfo.value = userInfoCache.value.get(video.id) || null
  } else {
    currentUserInfo.value = null
  }
}, { immediate: true })

function getUserNickname(): string {
  if (!props.currentVideo) return ''
  if (currentUserInfo.value?.user_nickname) {
    return currentUserInfo.value.user_nickname
  }
  return props.currentVideo.uploader_name || props.currentVideo.uploader_id || ''
}

function getUserIconUrl(): string | null {
  if (!props.currentVideo) return null
  if (currentUserInfo.value?.user_icon_url) {
    return currentUserInfo.value.user_icon_url
  }
  return getUploaderAvatarUrl(props.currentVideo.uploader_id)
}

function togglePlayPause() {
  if (!playerReady.value) return
  sendCommand(isPlaying.value ? 'pause' : 'play')
}

function sendCommand(command: string) {
  const target = resolvePlayerCommandTarget({
    lastMessageSource: lastPlayerMessageSource,
    iframeWindow: iframeRef.value?.contentWindow ?? null,
  })

  target?.postMessage(
    {
      eventName: command,
      playerId: '1',
      sourceConnectorType: 1,
    },
    'https://embed.nicovideo.jp'
  )
}

function getControllerPlaybackSettings() {
  return {
    autoPlay: autoPlay.value,
    autoSkip: props.showAutoSkip ? autoSkip.value : false,
    skipThreshold: skipThreshold.value,
  }
}

async function persistPlaybackSettings(updates: Pick<PlaybackSettings, 'auto_play' | 'auto_skip'>) {
  if (syncingPlaybackSettings.value) {
    return
  }

  const nextSettings = updatePlaybackSettings(
    {
      autoPlay: autoPlay.value,
      autoSkip: autoSkip.value,
      skipThreshold: skipThreshold.value,
    },
    {
      autoPlay: updates.auto_play,
      autoSkip: updates.auto_skip,
    },
  )

  await api.setPlaybackSettings({
    auto_play: nextSettings.autoPlay,
    auto_skip: nextSettings.autoSkip,
    skip_threshold: nextSettings.skipThreshold,
  })
}

function togglePlaybackSettingsPanel() {
  playbackSettingsOpen.value = !playbackSettingsOpen.value
}

function handleMessage(event: MessageEvent) {
  if (!event.data || event.origin !== 'https://embed.nicovideo.jp') return

  lastPlayerMessageSource = rememberPlayerMessageSource(event.source)

  const data = typeof event.data === 'string' ? JSON.parse(event.data) : event.data
  const controllerPlaybackSettings = getControllerPlaybackSettings()
  playerController.setPlaybackSettings(controllerPlaybackSettings)
  playerController.handlePlayerEvent(data)
  playerReady.value = playerController.state.playerReady
  isPlaying.value = playerController.state.isPlaying
}

// Lifecycle
let unlistenPlaybackSettings: (() => void) | null = null
let unlistenVideoWatched: (() => void) | null = null

onMounted(async () => {
  window.addEventListener('message', handleMessage)
  
  unlistenPlaybackSettings = await listen<PlaybackSettings>('playback-settings-changed', (event) => {
    const settings = event.payload
    syncingPlaybackSettings.value = true
    autoPlay.value = settings.auto_play
    autoSkip.value = settings.auto_skip
    skipThreshold.value = settings.skip_threshold
    syncingPlaybackSettings.value = false
    playerController.setPlaybackSettings(getControllerPlaybackSettings())
  })
  
  unlistenVideoWatched = await listen<{ video_id: string; is_watched: boolean }>('video-watched', (event) => {
    const { video_id } = event.payload
    if (props.currentVideo?.id === video_id) {
      // Update handled by parent
    }
  })
  
  // Get initial playback settings
  const settings = await api.getPlaybackSettings()
  syncingPlaybackSettings.value = true
  autoPlay.value = settings.auto_play
  autoSkip.value = settings.auto_skip
  skipThreshold.value = settings.skip_threshold
  syncingPlaybackSettings.value = false
  playerController.setPlaybackSettings(getControllerPlaybackSettings())
})

onUnmounted(() => {
  window.removeEventListener('message', handleMessage)
  if (unlistenPlaybackSettings) unlistenPlaybackSettings()
  if (unlistenVideoWatched) unlistenVideoWatched()
})

// Save settings to localStorage when changed
watch(autoPlay, async (val, oldVal) => {
  localStorage.setItem('vocaloidAutoPlay', val.toString())
  if (oldVal !== undefined && val !== oldVal) {
    await persistPlaybackSettings({ auto_play: val, auto_skip: autoSkip.value })
  }
})

watch(autoSkip, async (val, oldVal) => {
  localStorage.setItem('vocaloidAutoSkip', val.toString())
  if (oldVal !== undefined && val !== oldVal) {
    await persistPlaybackSettings({ auto_play: autoPlay.value, auto_skip: val })
  }
})

watch(skipThreshold, (val) => {
  localStorage.setItem('vocaloidSkipThreshold', val.toString())
})
</script>

<template>
  <div class="player-column">
    <!-- PiP Placeholder -->
    <div v-if="pipActive" class="pip-placeholder">
      <p>{{ t('player.pipActive') }}</p>
      <button @click="$emit('closePip')">{{ t('player.returnToMain') }}</button>
    </div>
    
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
              ref="iframeRef"
              :src="`https://embed.nicovideo.jp/watch/${currentVideo.id}?jsapi=1&playerId=1`"
              frameborder="0"
              allow="autoplay; encrypted-media"
              allowfullscreen
            ></iframe>
          </div>
        </div>

        <!-- Playback Controls -->
        <div v-else-if="section.section === 'controls'" class="playback-controls">
          <div class="main-bar">
            <div class="media-actions">
              <WatchLaterButton
                :video-id="currentVideo?.id || null"
                :video-title="currentVideo?.title"
                :thumbnail-url="currentVideo?.thumbnail_url"
                :disabled="!currentVideo"
              />
              <button class="icon-btn" :disabled="currentVideoIndex <= 0" @click="$emit('playPrevious')">⏮</button>
              <button class="icon-btn play-pause-btn" @click="togglePlayPause">
                {{ isPlaying ? '⏸' : '▶' }}
              </button>
              <button class="icon-btn" :disabled="currentVideoIndex < 0 || (currentVideoIndex >= resultsCount - 1 && !hasNext)" @click="$emit('playNext')">⏭</button>
            </div>

            <div v-if="showAutoSkip" class="playback-settings-slot">
              <button
                class="icon-btn settings-btn"
                type="button"
                :title="t('player.playbackSettings')"
                :aria-label="t('player.playbackSettings')"
                :aria-expanded="playbackSettingsViewModel.panelOpen"
                @click="togglePlaybackSettingsPanel"
              >⚙</button>
            </div>

            <button class="icon-btn pip-btn" @click="$emit('openPip')" :disabled="!currentVideo" title="PiP">📺</button>
          </div>

          <div
            v-if="showAutoSkip && playbackSettingsViewModel.panelOpen"
            class="playback-settings-panel"
          >
            <label class="toggle-label">
              <input v-model="autoPlay" type="checkbox">
              <span>{{ t('player.autoPlay') }}</span>
            </label>
            <label class="toggle-label">
              <input v-model="autoSkip" type="checkbox">
              <span>{{ t('player.autoSkip') }}</span>
            </label>
          </div>
        </div>
      </template>
    </template>
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

.player-header {
  padding: var(--space-md);
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.header-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-md);
  margin-bottom: var(--space-sm);
}

.video-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  margin: 0;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex: 1;
}

.upload-datetime {
  font-size: 15px;
  color: var(--color-text-secondary-light);
  white-space: nowrap;
  flex-shrink: 0;
}

.meta-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-md);
}

.uploader-info {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--color-bg-hover);
  object-fit: cover;
  display: flex;
  align-items: center;
  justify-content: center;
}

.default-avatar {
  font-size: 14px;
  color: var(--color-text-muted);
}

.uploader-info-placeholder {
  flex: 1;
}

.user-name {
  font-size: var(--font-size-sm);
  color: var(--color-text-primary);
  font-weight: 500;
}

.stats {
  display: flex;
  gap: var(--space-lg);
}

.stat {
  font-size: var(--font-size-sm);
  font-weight: 600;
}

.stat.views { color: #93C5FD; }
.stat.likes { color: #f472b6; }
.stat.mylists { color: #34d399; }
.stat.comments { color: #fbbf24; }

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

.playback-controls {
  background: var(--color-bg-surface);
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.main-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-md);
  height: 56px;
  gap: var(--space-lg);
}

.media-actions {
  display: flex;
  align-items: center;
  gap: var(--space-md);
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

.play-pause-btn {
  width: 48px;
  height: 48px;
  font-size: 24px;
  background: rgba(20, 184, 166, 0.1);
  color: var(--color-accent-primary);
  border-color: rgba(20, 184, 166, 0.2);
}

.play-pause-btn:hover {
  background: var(--color-accent-primary);
  color: var(--color-bg-primary);
}

.playback-settings-slot {
  display: flex;
  align-items: center;
}

.settings-btn {
  font-size: 17px;
}

.playback-settings-panel {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-lg);
  padding: 0 var(--space-md) var(--space-md);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  cursor: pointer;
}

.toggle-label input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--color-accent-primary);
}

.pip-btn {
  /* PiP button at the end */
}

.info-below-player {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-sm) var(--space-md);
  background: var(--color-bg-surface);
}

.tags-section {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: var(--space-sm);
}

.tag {
  font-size: var(--font-size-xs);
  padding: 3px 8px;
  background: var(--color-bg-hover);
  color: var(--color-text-secondary);
  border-radius: 4px;
  white-space: nowrap;
}

.tag.more {
  color: var(--color-accent-primary);
  background: rgba(20, 184, 166, 0.1);
}

.description-section {
  border-top: 1px solid var(--color-border-subtle);
  padding-top: var(--space-sm);
}

.description-content {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  line-height: 1.7;
  word-break: break-word;
}

.description-content.collapsed {
  max-height: 8em;
  overflow: hidden;
}

.description-content :deep(br) {
  display: block;
  content: "";
  margin-bottom: 0.3em;
}

.description-content :deep(a) {
  color: var(--color-accent-primary);
  text-decoration: underline;
}

.expand-btn {
  display: block;
  width: 100%;
  margin-top: var(--space-xs);
  padding: var(--space-xs);
  font-size: var(--font-size-xs);
  color: var(--color-accent-primary);
  background: transparent;
  border: 1px solid var(--color-border-subtle);
  border-radius: 4px;
  cursor: pointer;
}

.expand-btn:hover {
  background: var(--color-bg-hover);
}
</style>
