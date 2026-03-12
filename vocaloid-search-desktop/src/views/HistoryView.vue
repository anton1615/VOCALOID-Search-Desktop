<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, nextTick } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { api, type Video, type UserInfo, type VideoSelectedPayload, formatDuration } from '../api/tauri-commands'
import { formatDateTime } from '../utils/dateTime'
import { mapHistoryEntryToVideo } from '../utils/playlistPlaceholders'
import { createHydratedCurrentVideo, getInitialPlaylistViewState, scrollVideoIntoView, shouldApplyPlaylistSelection, shouldApplyPlaylistSelectionVersion } from '../features/playlistViews/playlistViewState'
import { createPagedPlaylistController } from '../features/playlistViews/pagedPlaylistController'

const { t } = useI18n()

const userInfoCache = reactive(new Map<string, UserInfo>())

const results = ref<Video[]>([])
const totalCount = ref(0)
const page = ref(1)
const pageSize = ref(50)
const hasNext = ref(false)
const loading = ref(false)
const loadingMore = ref(false)

const currentVideo = ref<Video | null>(null)
const currentVideoIndex = ref(-1)

// Filtering
const searchQuery = ref('')
const sortOrder = ref<'desc' | 'asc'>('desc')

const pipActive = ref(false)

const historyController = createPagedPlaylistController({
  initialPage: page.value,
  initialPageSize: pageSize.value,
  initialSortOrder: sortOrder.value,
  fetchPage: async (nextPage, nextPageSize, nextSortOrder) => {
    const response = await api.getHistory(nextPage, nextPageSize, nextSortOrder)
    return {
      total: response.total,
      has_next: response.has_next,
      results: response.results.map(mapHistoryEntryToVideo),
    }
  },
})

function syncHistorySnapshot(snapshot: {
  results: Video[]
  totalCount: number
  page: number
  pageSize: number
  hasNext: boolean
  sortOrder: 'desc' | 'asc'
}) {
  results.value = snapshot.results
  totalCount.value = snapshot.totalCount
  page.value = snapshot.page
  pageSize.value = snapshot.pageSize
  hasNext.value = snapshot.hasNext
  sortOrder.value = snapshot.sortOrder
}

function formatWatchedAt(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)
  
  if (diffMins < 1) return t('history.justNow')
  if (diffMins < 60) return t('history.minutesAgo', { count: diffMins })
  if (diffHours < 24) return t('history.hoursAgo', { count: diffHours })
  if (diffDays < 7) return t('history.daysAgo', { count: diffDays })
  return formatDateTime(dateStr)
}

async function loadHistory() {
  loading.value = true
  try {
    historyController.setSortOrder(sortOrder.value)
    const snapshot = await historyController.loadFirstPage()
    syncHistorySnapshot(snapshot)
  } catch (e) {
    console.error('Failed to load history:', e)
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  if (!hasNext.value || loadingMore.value) return
  loadingMore.value = true
  try {
    const snapshot = await historyController.loadNextPage()
    if (snapshot) {
      syncHistorySnapshot(snapshot)
    }
  } catch (e) {
    console.error('Failed to load more:', e)
  } finally {
    loadingMore.value = false
  }
}

async function hydrateCurrentVideo(video: Video, index: number) {
  currentVideoIndex.value = index
  console.log('[HistoryView] hydrateCurrentVideo start:', {
    id: video.id,
    index,
    baseTitle: video.title,
    baseStartTime: video.start_time,
    baseUploaderId: video.uploader_id,
    baseUploaderName: video.uploader_name,
    baseViews: video.view_count,
    baseTags: video.tags?.length ?? 0,
    hasBaseDescription: !!video.description,
  })
  
  try {
    const fullInfo = await api.fetchFullVideoInfo(video.id)
    console.log('[HistoryView] hydrateCurrentVideo fullInfo:', {
      id: fullInfo.id,
      title: fullInfo.title,
      startTime: fullInfo.start_time,
      uploaderId: fullInfo.uploader_id,
      uploaderName: fullInfo.uploader_name,
      views: fullInfo.view_count,
      tags: fullInfo.tags?.length ?? 0,
      hasDescription: !!fullInfo.description,
    })
    
    currentVideo.value = fullInfo
    results.value[index] = fullInfo
    await api.updatePlaylistVideo(index, fullInfo)
    if (fullInfo.uploader_id) {
      const userInfo = await api.getUserInfo(fullInfo.id)
      if (userInfo) {
        userInfoCache.set(fullInfo.id, userInfo)
      }
    }
    return
  } catch (e) {
    console.error('Failed to fetch video info:', e)
  }
  
  currentVideo.value = createHydratedCurrentVideo(video, null)
}

async function playVideo(video: Video, index: number) {
  currentVideoIndex.value = index
  await api.setPlaylistType('History')
  await api.setPlaylistIndex(index)
  await hydrateCurrentVideo(video, index)
}

// Infinite scroll
const observerTarget = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

function setupObserver() {
  if (observer) observer.disconnect()
  
  observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && hasNext.value && !loadingMore.value) {
      loadMore()
    }
  }, { threshold: 0.1 })
  
  if (observerTarget.value) {
    observer.observe(observerTarget.value)
  }
}

// Event listeners
let unlistenVideoSelected: (() => void) | null = null
let unlistenVideoWatched: (() => void) | null = null

onMounted(async () => {
  // Restore view state, but always reload fresh list from DB instead of trusting stale Rust playlist entries
  try {
    const playlistState = await api.getPlaylistState()
    const historyState = await api.getHistoryState()
    
    page.value = historyState.page || 1
    pageSize.value = historyState.page_size || 50
    sortOrder.value = (historyState.sort_direction as 'desc' | 'asc') || 'desc'
    searchQuery.value = historyState.search_query || ''
    pipActive.value = playlistState.pip_active
    
    await loadHistory()
    
    const initialViewState = getInitialPlaylistViewState({
      expectedPlaylistType: 'History',
      expectedPlaylistVersion: historyState.version,
      playlistType: playlistState.playlist_type,
      playlistVersion: playlistState.playlist_version,
      playlistIndex: playlistState.index ?? -1,
      results: results.value,
    })

    if (initialViewState.selectedVideo && initialViewState.selectedIndex >= 0) {
      await hydrateCurrentVideo(initialViewState.selectedVideo, initialViewState.selectedIndex)
    }

    // Scroll to the currently playing video after state restoration
    await nextTick()
    const listContainer = document.querySelector('.video-list')
    scrollVideoIntoView(initialViewState.selectedIndex, listContainer as HTMLElement | null)
  } catch (e) {
    console.error('[HistoryView] Failed to restore state:', e)
    await loadHistory()
  }
  
  setupObserver()
  
  unlistenVideoSelected = await listen<VideoSelectedPayload>('video-selected', async (event) => {
    const payload = event.payload
    const latestPlaylistState = await api.getPlaylistState()
    if (
      !shouldApplyPlaylistSelection('History', payload) ||
      !shouldApplyPlaylistSelectionVersion(latestPlaylistState.playlist_version, payload)
    ) {
      currentVideo.value = null
      currentVideoIndex.value = -1
      return
    }
    currentVideoIndex.value = payload.index
    
    const baseVideo = results.value[payload.index] ?? payload.video
    await hydrateCurrentVideo(baseVideo, payload.index)
    
    // Scroll logic: keep videos visible above and below
    const videoElement = document.getElementById('video-' + payload.index)
    const prevVideoElement = document.getElementById('video-' + (payload.index - 1))
    const nextNextVideoElement = document.getElementById('video-' + (payload.index + 2))
    const listContainer = document.querySelector('.video-list')
    
    if (listContainer) {
      const containerRect = listContainer.getBoundingClientRect()
      
      if (prevVideoElement && payload.index > 0) {
        const prevRect = prevVideoElement.getBoundingClientRect()
        if (prevRect.top < containerRect.top) {
          prevVideoElement.scrollIntoView({ behavior: 'smooth', block: 'start' })
        }
      }
      
      if (nextNextVideoElement) {
        const nextNextRect = nextNextVideoElement.getBoundingClientRect()
        if (nextNextRect.bottom > containerRect.bottom) {
          nextNextVideoElement.scrollIntoView({ behavior: 'smooth', block: 'end' })
        }
      } else if (videoElement) {
        const videoRect = videoElement.getBoundingClientRect()
        if (videoRect.bottom > containerRect.bottom || videoRect.top < containerRect.top) {
          videoElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
        }
      }
    }
  })
  
  unlistenVideoWatched = await listen<{ video_id: string; is_watched: boolean }>('video-watched', async (event) => {
    const { video_id, is_watched } = event.payload
    if (currentVideo.value?.id === video_id) {
      currentVideo.value.is_watched = is_watched
    }
  })
})

onUnmounted(() => {
  if (observer) observer.disconnect()
  if (unlistenVideoSelected) unlistenVideoSelected()
  if (unlistenVideoWatched) unlistenVideoWatched()
  
  // Save state to Rust
  api.setHistoryState({
    page: page.value,
    page_size: pageSize.value,
    total_count: totalCount.value,
    has_next: hasNext.value,
    sort_direction: sortOrder.value,
    search_query: searchQuery.value,
    version: 0,
  }).catch(e => console.error('[HistoryView] Failed to save state:', e))
})

// Apply filters
function applyFilters() {
  page.value = 1
  loadHistory()
}

// Toggle sort order
function toggleSortOrder() {
  sortOrder.value = sortOrder.value === 'desc' ? 'asc' : 'desc'
  applyFilters()
}
</script>

<template>
  <div class="history-view">
    <div class="list-column">
      <div class="header">
        <h2>📜 {{ t('history.title') }}</h2>
        <span class="count">{{ totalCount.toLocaleString() }} {{ t('history.videos') }}</span>
      </div>
      
      <div class="filter-bar">
        <div class="filter-scroll">
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('filter.search')"
            class="search-filter"
            @keyup.enter="applyFilters"
          />
          <button class="filter-pill" @click="toggleSortOrder">
            {{ sortOrder === 'desc' ? `↓ ${t('filter.desc')}` : `↑ ${t('filter.asc')}` }}
          </button>
        </div>
      </div>
      
      <div v-if="loading && results.length === 0" class="loading-state">
        <div class="spinner"></div>
        <span>{{ t('history.loading') }}</span>
      </div>
      
      <div v-else-if="results.length === 0" class="empty-state">
        {{ t('history.empty') }}
      </div>
      
      <div v-else class="video-list">
        <div
          v-for="(video, idx) in results"
          :key="video.id"
          :id="'video-' + idx"
          class="video-item"
          :class="{ playing: currentVideo?.id === video.id }"
        >
          <div class="thumbnail-wrapper" @click="playVideo(video, idx)">
            <img :src="video.thumbnail_url || ''" class="thumbnail" loading="lazy" />
            <span v-if="video.duration" class="duration">{{ formatDuration(video.duration) }}</span>
          </div>
          
          <div class="video-info" @click="playVideo(video, idx)">
            <div class="title-row">
              <h3 class="title">{{ video.title }}</h3>
            </div>
            <div class="watched-at">{{ t('history.watchedAt') }} {{ formatWatchedAt(video.start_time || '') }}</div>
          </div>
        </div>
        
        <div ref="observerTarget" class="scroll-trigger">
          <div v-if="loadingMore" class="spinner"></div>
          <span v-else-if="!hasNext && results.length > 0" class="end-message">{{ t('search.noMore') }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 0;
  background: var(--color-bg-primary);
}

.list-column {
  display: flex;
  flex-direction: column;
  background: var(--color-bg-surface);
  border-right: 1px solid var(--color-border-subtle);
  overflow: hidden;
}

.header {
  padding: var(--space-md);
  border-bottom: 1px solid var(--color-border-subtle);
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.header h2 {
  margin: 0;
  font-size: var(--font-size-lg);
  font-weight: 600;
}

.count {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

.filter-bar {
  padding: var(--space-sm) var(--space-md);
  border-bottom: 1px solid var(--color-border-subtle);
}

.filter-scroll {
  display: flex;
  gap: var(--space-sm);
  align-items: center;
}

.search-filter {
  flex: 1;
  padding: var(--space-xs) var(--space-sm);
  border-radius: 4px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
}

.filter-pill {
  padding: var(--space-xs) var(--space-sm);
  border-radius: 4px;
  background: var(--color-bg-hover);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  white-space: nowrap;
}

.filter-pill:hover {
  background: var(--color-accent-primary);
  color: white;
}

.loading-state,
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  gap: var(--space-sm);
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--color-border-subtle);
  border-top-color: var(--color-accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.video-list {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-sm);
}

.video-item {
  display: flex;
  gap: var(--space-sm);
  padding: var(--space-sm);
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
}

.video-item:hover {
  background: var(--color-bg-hover);
}

.video-item.playing {
  background: rgba(20, 184, 166, 0.1);
  border-left: 3px solid var(--color-accent-primary);
}

.thumbnail-wrapper {
  position: relative;
  flex-shrink: 0;
}

.thumbnail {
  width: 120px;
  height: 68px;
  object-fit: cover;
  border-radius: 4px;
}

.duration {
  position: absolute;
  bottom: 4px;
  right: 4px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 2px 4px;
  border-radius: 2px;
  font-size: 11px;
}

.video-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.title {
  font-size: var(--font-size-sm);
  font-weight: 500;
  margin: 0 0 var(--space-xs) 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.watched-at {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.scroll-trigger {
  padding: var(--space-md);
  text-align: center;
}

.end-message {
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.resize-divider {
  width: 4px;
  cursor: col-resize;
  background: var(--color-border-subtle);
  transition: background 0.2s;
}

.resize-divider:hover,
.resize-divider.dragging {
  background: var(--color-accent-primary);
}

.player-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--color-bg-surface);
  overflow: hidden;
  position: relative;
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
  padding: var(--space-sm) var(--space-md);
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.main-bar {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.media-actions {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
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

.pip-btn {
  margin-left: auto;
}

.info-below-player {
  flex-shrink: 0;
  padding: var(--space-sm) var(--space-md);
  overflow-y: auto;
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
  line-height: 1.6;
  color: var(--color-text-secondary);
}

.description-content.collapsed {
  max-height: 100px;
  overflow: hidden;
}

.expand-btn {
  margin-top: var(--space-sm);
  padding: var(--space-xs) var(--space-sm);
  border-radius: 4px;
  background: var(--color-bg-hover);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary);
  font-size: var(--font-size-xs);
  cursor: pointer;
}
</style>
