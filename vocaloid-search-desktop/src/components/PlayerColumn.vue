<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { api, type Video, type UserInfo, type PlaybackSettings, getUploaderAvatarUrl } from '../api/tauri-commands'
import WatchLaterButton from './WatchLaterButton.vue'

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
const isPlaying = ref(false)
const playerReady = ref(false)
const hasMarkedCurrent = ref(false)
const autoPlay = ref(localStorage.getItem('vocaloidAutoPlay') !== 'false')
const autoSkip = ref(localStorage.getItem('vocaloidAutoSkip') === 'true')
const skipThreshold = ref(parseInt(localStorage.getItem('vocaloidSkipThreshold') || '30', 10))
const descriptionExpanded = ref(false)

const userInfoCache = ref(new Map<string, UserInfo>())
const currentUserInfo = ref<UserInfo | null>(null)

// Watch for currentVideo changes to fetch user info and reset states
watch(() => props.currentVideo, async (video, oldVideo) => {
  // Reset states when video changes
  if (video?.id !== oldVideo?.id) {
    playerReady.value = false
    isPlaying.value = false
    hasMarkedCurrent.value = false
    descriptionExpanded.value = false
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

function formatDateTime(dateStr: string | null): string {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  const year = date.getFullYear()
  const month = (date.getMonth() + 1).toString().padStart(2, '0')
  const day = date.getDate().toString().padStart(2, '0')
  const hour = date.getHours().toString().padStart(2, '0')
  const minute = date.getMinutes().toString().padStart(2, '0')
  return `${year}/${month}/${day} ${hour}:${minute}`
}

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
  iframeRef.value?.contentWindow?.postMessage(
    {
      eventName: command,
      playerId: '1',
      sourceConnectorType: 1,
    },
    'https://embed.nicovideo.jp'
  )
}

function handleMessage(event: MessageEvent) {
  if (!event.data || event.origin !== 'https://embed.nicovideo.jp') return

  const data = typeof event.data === 'string' ? JSON.parse(event.data) : event.data

  if (data.eventName === 'loadComplete') {
    playerReady.value = true
    if (autoPlay.value) {
      setTimeout(() => sendCommand('play'), 500)
    }
  }

  if (data.eventName === 'playerStatusChange' || data.eventName === 'statusChange') {
    const status = data.data?.playerStatus
    const statusNum = typeof status === 'string' ? parseInt(status, 10) : status

    if (statusNum === 2) {
      isPlaying.value = true
      if (props.currentVideo && !hasMarkedCurrent.value) {
        api.markWatched(
          props.currentVideo.id,
          props.currentVideo.title,
          props.currentVideo.thumbnail_url
        )
        emit('videoWatched', { ...props.currentVideo, is_watched: true })
        hasMarkedCurrent.value = true
      }
    } else if (statusNum === 3) {
      isPlaying.value = false
    } else if (statusNum === 4) {
      isPlaying.value = false
      if (autoPlay.value) {
        emit('playNext')
      }
    }
  }

  if (data.eventName === 'playerMetadataChange') {
    const currentTime = data.data?.currentTime
    const duration = data.data?.duration
    if (currentTime && duration && autoSkip.value && props.showAutoSkip) {
      const remaining = duration - currentTime
      if (remaining <= skipThreshold.value && currentTime > 10) {
        emit('playNext')
      }
    }
  }
}

// Lifecycle
let unlistenPlaybackSettings: (() => void) | null = null
let unlistenVideoWatched: (() => void) | null = null

onMounted(async () => {
  window.addEventListener('message', handleMessage)
  
  unlistenPlaybackSettings = await listen<PlaybackSettings>('playback-settings-changed', (event) => {
    const settings = event.payload
    autoPlay.value = settings.auto_play
    autoSkip.value = settings.auto_skip
    skipThreshold.value = settings.skip_threshold
  })
  
  unlistenVideoWatched = await listen<{ video_id: string; is_watched: boolean }>('video-watched', (event) => {
    const { video_id } = event.payload
    if (props.currentVideo?.id === video_id) {
      // Update handled by parent
    }
  })
  
  // Get initial playback settings
  const settings = await api.getPlaybackSettings()
  autoPlay.value = settings.auto_play
  autoSkip.value = settings.auto_skip
  skipThreshold.value = settings.skip_threshold
})

onUnmounted(() => {
  window.removeEventListener('message', handleMessage)
  if (unlistenPlaybackSettings) unlistenPlaybackSettings()
  if (unlistenVideoWatched) unlistenVideoWatched()
})

// Save settings to localStorage when changed
watch(autoSkip, (val) => {
  localStorage.setItem('vocaloidAutoSkip', val.toString())
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
      <!-- Player Header -->
      <div v-if="currentVideo" class="player-header">
        <div class="header-row">
          <h1 class="video-title">{{ currentVideo.title }}</h1>
          <span class="upload-datetime">{{ formatDateTime(currentVideo.start_time) }}</span>
        </div>
        <div class="meta-row">
          <div class="uploader-info" v-if="currentVideo.uploader_id || currentVideo.uploader_name">
            <img v-if="getUserIconUrl()" :src="getUserIconUrl()!" class="avatar" />
            <div v-else class="avatar default-avatar">👤</div>
            <span class="user-name">{{ getUserNickname() }}</span>
          </div>
          <div v-else class="uploader-info-placeholder"></div>
          <div class="stats">
            <span class="stat views">▶ {{ currentVideo.view_count?.toLocaleString() ?? 0 }}</span>
            <span class="stat likes">❤️ {{ currentVideo.like_count?.toLocaleString() ?? 0 }}</span>
            <span class="stat mylists">📝 {{ currentVideo.mylist_count?.toLocaleString() ?? 0 }}</span>
            <span class="stat comments">💬 {{ currentVideo.comment_count?.toLocaleString() ?? 0 }}</span>
          </div>
        </div>
      </div>
      
      <!-- Video Container -->
      <div class="video-container">
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
      <div class="playback-controls">
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
          
          <!-- Auto-skip controls -->
          <div v-if="showAutoSkip" class="auto-skip-controls">
            <label class="toggle-label">
              <input type="checkbox" v-model="autoSkip">
              <span>{{ t('player.autoSkip') }}</span>
            </label>
            <div v-if="autoSkip" class="threshold-input">
              <input type="number" v-model.number="skipThreshold" min="5" max="120" step="5">
              <span>{{ t('player.seconds') }}</span>
            </div>
          </div>
          
          <button class="icon-btn pip-btn" @click="$emit('openPip')" :disabled="!currentVideo" title="PiP">📺</button>
        </div>
      </div>
      
      <!-- Info Below Player -->
      <div v-if="currentVideo" class="info-below-player">
        <div v-if="currentVideo.tags?.length" class="tags-section">
          <span class="tag" v-for="tag in currentVideo.tags.slice(0, 12)" :key="tag">{{ tag }}</span>
          <span class="tag more" v-if="currentVideo.tags.length > 12">+{{ currentVideo.tags.length - 12 }}</span>
        </div>
        <div v-if="currentVideo.description" class="description-section">
          <div class="description-content" :class="{ collapsed: !descriptionExpanded }" v-html="currentVideo.description"></div>
          <button 
            v-if="currentVideo.description.length > 200" 
            class="expand-btn" 
            @click="descriptionExpanded = !descriptionExpanded"
          >
            {{ descriptionExpanded ? t('player.collapse') : t('player.expand') }}
          </button>
        </div>
      </div>
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

.auto-skip-controls {
  display: flex;
  align-items: center;
  gap: var(--space-md);
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

.threshold-input {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.threshold-input input {
  width: 50px;
  padding: 4px 8px;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary);
  border-radius: 4px;
  font-size: var(--font-size-sm);
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
