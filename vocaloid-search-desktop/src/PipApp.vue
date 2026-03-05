<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { api, type Video, type VideoSelectedPayload, type PlaybackSettings, type UserInfo, getUploaderAvatarUrl } from './api/tauri-commands'

const currentVideo = ref<Video | null>(null)
const currentIndex = ref(-1)
const hasNext = ref(false)
const iframeRef = ref<HTMLIFrameElement | null>(null)
const isPlaying = ref(false)
const playerReady = ref(false)
const autoPlay = ref(true)
const autoSkip = ref(false)
const skipThreshold = ref(30)
const descriptionExpanded = ref(false)

const userInfoCache = reactive(new Map<string, UserInfo>())
const currentUserInfo = ref<UserInfo | null>(null)

const pipWindow = getCurrentWindow()
let isClosing = false

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
    console.error('Failed to save pip window state:', e)
  }
}

function formatDateTime(dateStr: string | null): string {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  return `${date.getFullYear()}/${(date.getMonth() + 1).toString().padStart(2, '0')}/${date.getDate().toString().padStart(2, '0')} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`
}

function getCachedUserNickname(video: Video | null): string {
  if (!video) return ''
  if (video.id === currentVideo.value?.id && currentUserInfo.value?.user_nickname) {
    return currentUserInfo.value.user_nickname
  }
  const cached = userInfoCache.get(video.id)
  if (cached?.user_nickname) return cached.user_nickname
  return video.uploader_name || video.uploader_id || ''
}

function getCachedUserIconUrl(video: Video | null): string | null {
  if (!video) return null
  if (video.id === currentVideo.value?.id && currentUserInfo.value?.user_icon_url) {
    return currentUserInfo.value.user_icon_url
  }
  const cached = userInfoCache.get(video.id)
  if (cached?.user_icon_url) return cached.user_icon_url
  return getUploaderAvatarUrl(video.uploader_id)
}

function loadPlayer(videoId: string) {
  console.log('[PiP] loadPlayer called with:', videoId, 'iframeRef:', !!iframeRef.value)
  if (iframeRef.value) {
    playerReady.value = false
    iframeRef.value.src = `https://embed.nicovideo.jp/watch/${videoId}?jsapi=1&playerId=1`
    console.log('[PiP] iframe src set')
  } else {
    console.error('[PiP] iframeRef is null!')
  }
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

function togglePlayPause() {
  if (!playerReady.value) return
  sendCommand(isPlaying.value ? 'pause' : 'play')
}

async function playNext() {
  if (currentIndex.value >= 0 && hasNext.value) {
    await api.setPlaylistIndex(currentIndex.value + 1)
  }
}

async function playPrevious() {
  if (currentIndex.value > 0) {
    await api.setPlaylistIndex(currentIndex.value - 1)
  }
}

async function toggleAutoSkip() {
  autoSkip.value = !autoSkip.value
  await api.setPlaybackSettings({
    auto_play: autoPlay.value,
    auto_skip: autoSkip.value,
    skip_threshold: skipThreshold.value
  })
}

async function updateSkipThreshold(value: number) {
  skipThreshold.value = value
  await api.setPlaybackSettings({
    auto_play: autoPlay.value,
    auto_skip: autoSkip.value,
    skip_threshold: value
  })
}

async function fetchUserInfo(video: Video) {
  if (!userInfoCache.has(video.id)) {
    try {
      const userInfo = await api.getUserInfo(video.id)
      if (userInfo) {
        userInfoCache.set(video.id, userInfo)
      }
    } catch (e) {
      console.error('Failed to fetch user info:', e)
    }
  }
  currentUserInfo.value = userInfoCache.get(video.id) || null
}

async function handleVideoChange(video: Video, index: number, hasNextVideo: boolean) {
  console.log('[PiP] handleVideoChange called:', video.id, 'index:', index)
  currentVideo.value = video
  currentIndex.value = index
  hasNext.value = hasNextVideo
  descriptionExpanded.value = false
  currentUserInfo.value = null

  await fetchUserInfo(video)
  console.log('[PiP] Calling loadPlayer with:', video.id)
  loadPlayer(video.id)
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
    } else if (statusNum === 3) {
      isPlaying.value = false
    } else if (statusNum === 4) {
      isPlaying.value = false
      if (autoPlay.value) {
        playNext()
      }
    }
  }

  if (data.eventName === 'playerMetadataChange') {
    const currentTime = data.data?.currentTime
    const duration = data.data?.duration
    if (currentTime && duration && autoSkip.value) {
      const remaining = duration - currentTime
      if (remaining <= skipThreshold.value && currentTime > 10) {
        playNext()
      }
    }
  }
}

let unlistenVideoSelected: (() => void) | null = null
let unlistenPlaybackSettings: (() => void) | null = null
let unlistenResize: (() => void) | null = null
let unlistenMove: (() => void) | null = null
let unlistenClose: (() => void) | null = null

onMounted(async () => {
  console.log('[PiP] onMounted start')
  window.addEventListener('message', handleMessage)

  const state = await api.getPlaylistState()
  console.log('[PiP] getPlaylistState result:', state.results.length, 'videos, index:', state.index)
  if (state.results.length > 0 && state.index >= 0 && state.index < state.results.length) {
    await handleVideoChange(state.results[state.index], state.index, state.has_next)
  }

  const settings = await api.getPlaybackSettings()
  autoPlay.value = settings.auto_play
  autoSkip.value = settings.auto_skip
  skipThreshold.value = settings.skip_threshold

  console.log('[PiP] Setting up video-selected listener')
  unlistenVideoSelected = await listen<VideoSelectedPayload>('video-selected', async (event) => {
    const payload = event.payload
    console.log('[PiP] Received video-selected event:', payload.video.id, 'index:', payload.index)
    await handleVideoChange(payload.video, payload.index, payload.has_next)
  })

  unlistenPlaybackSettings = await listen<PlaybackSettings>('playback-settings-changed', (event) => {
    const settings = event.payload
    autoPlay.value = settings.auto_play
    autoSkip.value = settings.auto_skip
    skipThreshold.value = settings.skip_threshold
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
  
  console.log('[PiP] onMounted complete')
})

onUnmounted(() => {
  window.removeEventListener('message', handleMessage)
  if (unlistenVideoSelected) unlistenVideoSelected()
  if (unlistenPlaybackSettings) unlistenPlaybackSettings()
  if (unlistenResize) unlistenResize()
  if (unlistenMove) unlistenMove()
  if (unlistenClose) unlistenClose()
})
</script>

<template>
  <div class="pip-container">
    <div class="sidebar">
      <button @click="playPrevious" class="icon-btn" :disabled="currentIndex <= 0" title="上一首">⏮</button>
      <button @click="togglePlayPause" class="icon-btn play-pause-btn" :disabled="!currentVideo">
        {{ isPlaying ? '⏸' : '▶' }}
      </button>
      <button @click="playNext" class="icon-btn" :disabled="!hasNext" title="下一首">⏭</button>
      <div class="sidebar-divider"></div>
      <div class="auto-skip-section">
        <label class="toggle-label">
          <input type="checkbox" :checked="autoSkip" @change="toggleAutoSkip">
          <span>跳過</span>
        </label>
        <div v-if="autoSkip" class="threshold-input">
          <input type="number" :value="skipThreshold" @change="updateSkipThreshold(parseInt(($event.target as HTMLInputElement).value, 10))" min="5" max="120" step="5">
          <span>s</span>
        </div>
      </div>
    </div>

    <div class="player-column">
      <div v-if="!currentVideo" class="empty-state">
        <span>從主視窗選擇影片</span>
      </div>

      <template v-else>
        <div class="player-header">
          <div class="header-row">
            <h1 class="video-title">{{ currentVideo.title }}</h1>
            <span class="upload-datetime">{{ formatDateTime(currentVideo.start_time) }}</span>
          </div>
          <div class="meta-row">
            <div class="uploader-info" v-if="currentVideo.uploader_id">
              <img v-if="getCachedUserIconUrl(currentVideo)" :src="getCachedUserIconUrl(currentVideo)!" class="avatar" />
              <div v-else class="avatar default-avatar">👤</div>
              <span class="user-name">{{ getCachedUserNickname(currentVideo) }}</span>
            </div>
            <div class="stats">
              <span class="stat">▶ {{ currentVideo.view_count?.toLocaleString() ?? 0 }}</span>
              <span class="stat">❤️ {{ currentVideo.like_count?.toLocaleString() ?? 0 }}</span>
              <span class="stat">📝 {{ currentVideo.mylist_count?.toLocaleString() ?? 0 }}</span>
              <span class="stat">💬 {{ currentVideo.comment_count?.toLocaleString() ?? 0 }}</span>
            </div>
          </div>
        </div>

        <div class="video-container">
          <div class="aspect-ratio-box">
            <iframe
              ref="iframeRef"
              frameborder="0"
              allow="autoplay; encrypted-media"
              allowfullscreen
            ></iframe>
          </div>
        </div>

        <div class="info-below-player">
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
              {{ descriptionExpanded ? '收起' : '展開' }}
            </button>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.pip-container {
  display: flex;
  height: 100vh;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
}

.sidebar {
  width: 60px;
  min-width: 60px;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--space-md) var(--space-xs);
  gap: var(--space-sm);
  background: var(--color-bg-surface);
  border-right: 1px solid var(--color-border-subtle);
}

.sidebar-divider {
  width: 100%;
  height: 1px;
  background: var(--color-border-subtle);
  margin: var(--space-xs) 0;
}

.auto-skip-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-xs);
  font-size: var(--font-size-xs);
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

.toggle-label {
  display: flex;
  align-items: center;
  gap: 2px;
  cursor: pointer;
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
}

.toggle-label input[type="checkbox"] {
  width: 14px;
  height: 14px;
  accent-color: var(--color-accent-primary);
}

.threshold-input {
  display: flex;
  align-items: center;
  gap: 2px;
}

.threshold-input input {
  width: 32px;
  padding: 2px 4px;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary);
  border-radius: 3px;
  font-size: var(--font-size-xs);
  text-align: center;
}

.player-column {
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
  color: var(--color-text-primary);
  margin: 0;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex: 1;
}

.upload-datetime {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
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

.user-name {
  font-size: var(--font-size-sm);
  color: var(--color-text-primary);
  font-weight: 500;
}

.stats {
  display: flex;
  gap: var(--space-md);
}

.stat {
  font-size: var(--font-size-sm);
  font-weight: 600;
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

.aspect-ratio-box iframe {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  border: none;
}

.info-below-player {
  flex-shrink: 0;
  padding: var(--space-sm) var(--space-md);
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
