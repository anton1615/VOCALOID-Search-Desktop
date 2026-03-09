<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { api, type Video, type VideoSelectedPayload, type PlaybackSettings, type UserInfo, getUploaderAvatarUrl } from './api/tauri-commands'
import WatchLaterButton from './components/WatchLaterButton.vue'
import VideoMetaPanel from './components/VideoMetaPanel.vue'
import { formatDateTime } from './utils/dateTime'
import { createEmbeddedPlayerController } from './features/playlistViews/embeddedPlayerController'
import { getPipLayout } from './features/playlistViews/pipLayout'
import { resolvePlayerCommandTarget } from './features/playlistViews/playerCommandTarget'
import { rememberPlayerMessageSource, type PostMessageTarget } from './features/playlistViews/playerMessageSource'
const currentVideo = ref<Video | null>(null)
const currentIndex = ref(-1)
const hasNext = ref(false)
const iframeRef = ref<HTMLIFrameElement | null>(null)
let lastPlayerMessageSource: PostMessageTarget | null = null
const autoPlay = ref(true)
const autoSkip = ref(false)
const skipThreshold = ref(30)

const playerController = createEmbeddedPlayerController({
  sendCommand: (command) => sendCommand(command),
  onPlayNext: () => {
    void playNext()
  },
  onMarkWatched: (video) => {
    void api.markWatched(video.id, video.title, video.thumbnail_url)
    if (currentVideo.value?.id === video.id) {
      currentVideo.value.is_watched = true
    }
  },
  schedule: (callback) => setTimeout(callback, 500),
})

const isPlaying = ref(playerController.state.isPlaying)
const playerReady = ref(playerController.state.playerReady)

const userInfoCache = reactive(new Map<string, UserInfo>())
const currentUserInfo = ref<UserInfo | null>(null)
const pipLayout = getPipLayout()

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

function togglePlayPause() {
  if (!playerReady.value) return
  sendCommand(isPlaying.value ? 'pause' : 'play')
}

async function playNext() {
  if (currentIndex.value >= 0) {
    // Check if we need to preload more results
    // Load more when within 5 videos of the end
    const playlistState = await api.getPlaylistState()
    const remaining = playlistState.results.length - currentIndex.value - 1
    
    if (remaining <= 5 && hasNext.value) {
      // Close to end, try to preload more
      try {
        const searchState = await api.getSearchState()
        if (searchState.has_next) {
          console.log('[PiP] Preloading more results... (remaining:', remaining, ')')
          await api.loadMore()
        }
      } catch (e) {
        console.error('[PiP] Preload failed:', e)
      }
    } else if (!hasNext.value) {
      // At the end, try to load more as last resort
      try {
        const searchState = await api.getSearchState()
        if (searchState.has_next) {
          console.log('[PiP] At end, loading more...')
          await api.loadMore()
          const newPlaylistState = await api.getPlaylistState()
          if (newPlaylistState.index + 1 < newPlaylistState.results.length) {
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
  }
}

async function playPrevious() {
  if (currentIndex.value > 0) {
    await api.setPlaylistIndex(currentIndex.value - 1)
  }
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
  currentUserInfo.value = null
  playerController.setCurrentVideo(video)
  playerReady.value = playerController.state.playerReady
  isPlaying.value = playerController.state.isPlaying

  await fetchUserInfo(video)
  console.log('[PiP] Calling loadPlayer with:', video.id)
  loadPlayer(video.id)
}

function handleMessage(event: MessageEvent) {
  if (!event.data || event.origin !== 'https://embed.nicovideo.jp') return

  lastPlayerMessageSource = rememberPlayerMessageSource(event.source)

  const data = typeof event.data === 'string' ? JSON.parse(event.data) : event.data
  playerController.setPlaybackSettings({
    autoPlay: autoPlay.value,
    autoSkip: autoSkip.value,
    skipThreshold: skipThreshold.value,
  })
  playerController.handlePlayerEvent(data)
  playerReady.value = playerController.state.playerReady
  isPlaying.value = playerController.state.isPlaying
}

let unlistenVideoSelected: (() => void) | null = null
let unlistenPlaybackSettings: (() => void) | null = null
let unlistenVideoWatched: (() => void) | null = null
let unlistenSearchResultsUpdated: (() => void) | null = null
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
  playerController.setPlaybackSettings({
    autoPlay: autoPlay.value,
    autoSkip: autoSkip.value,
    skipThreshold: skipThreshold.value,
  })

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

  unlistenVideoWatched = await listen<{ video_id: string; is_watched: boolean }>('video-watched', (event) => {
    const { video_id, is_watched } = event.payload
    console.log('[PiP] Received video-watched event:', video_id, is_watched)
    if (currentVideo.value?.id === video_id) {
      currentVideo.value.is_watched = is_watched
    }
  })

  // Listen for search results updates from main window
  unlistenSearchResultsUpdated = await listen('search-results-updated', async () => {
    console.log('[PiP] Received search-results-updated event')
    // Get fresh playlist state after results are updated
    const playlistState = await api.getPlaylistState()
    // If we have a current video, update hasNext based on new results
    if (currentIndex.value >= 0 && currentIndex.value < playlistState.results.length) {
      hasNext.value = currentIndex.value + 1 < playlistState.results.length
    }
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
  if (unlistenVideoWatched) unlistenVideoWatched()
  if (unlistenSearchResultsUpdated) unlistenSearchResultsUpdated()
  if (unlistenResize) unlistenResize()
  if (unlistenMove) unlistenMove()
  if (unlistenClose) unlistenClose()
})
</script>

<template>
  <div class="pip-container">
    <div class="sidebar">
      <WatchLaterButton 
        :video-id="currentVideo?.id || null"
        :video-title="currentVideo?.title"
        :thumbnail-url="currentVideo?.thumbnail_url"
        :disabled="!currentVideo"
      />
      <button @click="playPrevious" class="icon-btn" :disabled="currentIndex <= 0" title="上一首">⏮</button>
      <button @click="togglePlayPause" class="icon-btn play-pause-btn" :disabled="!currentVideo">
        {{ isPlaying ? '⏸' : '▶' }}
      </button>
      <button @click="playNext" class="icon-btn" :disabled="!hasNext" title="下一首">⏭</button>
    </div>

    <div class="player-column">
      <div v-if="!currentVideo" class="empty-state">
        <span>從主視窗選擇影片</span>
      </div>

      <template v-else>
        <template v-for="contentSection in pipLayout.content" :key="contentSection.section">
          <VideoMetaPanel
            v-if="contentSection.section === 'header' || contentSection.section === 'details'"
            :video="currentVideo"
            :uploader-name="getCachedUserNickname(currentVideo)"
            :uploader-icon-url="getCachedUserIconUrl(currentVideo)"
            :upload-date-time="formatDateTime(currentVideo.start_time)"
            :display-mode="contentSection.videoMetaPanelMode"
          />

          <div v-else class="video-container">
            <div class="aspect-ratio-box">
              <iframe
                ref="iframeRef"
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
  width: 50px;
  min-width: 50px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-md) var(--space-xs);
  gap: var(--space-sm);
  background: var(--color-bg-surface);
  border-right: 1px solid var(--color-border-subtle);
}

.icon-btn {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
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
  width: 42px;
  height: 42px;
  font-size: 20px;
  background: rgba(20, 184, 166, 0.1);
  color: var(--color-accent-primary);
  border-color: rgba(20, 184, 166, 0.2);
}

.play-pause-btn:hover {
  background: var(--color-accent-primary);
  color: var(--color-bg-primary);
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
  font-size: 15px;
  font-weight: 500;
}

.stat.views { color: var(--color-stat-views); }
.stat.likes { color: var(--color-stat-likes); }
.stat.mylists { color: var(--color-stat-mylists); }
.stat.comments { color: var(--color-stat-comments); }

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
