<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { api, type Video, type VideoSelectedPayload, formatDuration } from '../api/tauri-commands'
import { formatDateTime } from '../utils/dateTime'
import { mapWatchLaterEntryToVideo } from '../utils/playlistPlaceholders'
import { getInitialPlaylistViewState, scrollVideoIntoView, shouldApplyPlaylistSelection, shouldApplyPlaylistSelectionVersion } from '../features/playlistViews/playlistViewState'
import { createPagedPlaylistController } from '../features/playlistViews/pagedPlaylistController'

const { t } = useI18n()

const results = ref<Video[]>([])
const totalCount = ref(0)
const page = ref(1)
const pageSize = ref(50)
const hasNext = ref(false)
const loading = ref(false)
const loadingMore = ref(false)

const currentVideo = ref<Video | null>(null)
const currentVideoIndex = ref(-1)
const showRemoveConfirm = ref(false)
const pendingRemovalVideo = ref<Video | null>(null)
const removing = ref(false)

// Filtering
const searchQuery = ref('')
const sortOrder = ref<'desc' | 'asc'>('desc')

const pipActive = ref(false)

const watchLaterController = createPagedPlaylistController({
  initialPage: page.value,
  initialPageSize: pageSize.value,
  initialSortOrder: sortOrder.value,
  fetchPage: async (nextPage, nextPageSize, nextSortOrder) => {
    const response = await api.getWatchLater(nextPage, nextPageSize, nextSortOrder)
    return {
      total: response.total,
      has_next: response.has_next,
      results: response.results.map(mapWatchLaterEntryToVideo),
    }
  },
})

function syncWatchLaterSnapshot(snapshot: {
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

function formatAddedAt(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)
  
  if (diffMins < 1) return t('watchLater.justNow')
  if (diffMins < 60) return t('watchLater.minutesAgo', { count: diffMins })
  if (diffHours < 24) return t('watchLater.hoursAgo', { count: diffHours })
  if (diffDays < 7) return t('watchLater.daysAgo', { count: diffDays })
  return formatDateTime(dateStr)
}

async function loadWatchLater() {
  loading.value = true
  try {
    watchLaterController.setSortOrder(sortOrder.value)
    const snapshot = await watchLaterController.loadFirstPage()
    syncWatchLaterSnapshot(snapshot)
  } catch (e) {
    console.error('Failed to load watch later:', e)
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  if (!hasNext.value || loadingMore.value) return
  loadingMore.value = true
  try {
    const snapshot = await watchLaterController.loadNextPage()
    if (snapshot) {
      syncWatchLaterSnapshot(snapshot)
    }
  } catch (e) {
    console.error('Failed to load more:', e)
  } finally {
    loadingMore.value = false
  }
}

async function playVideo(_video: Video, index: number) {
  await api.setPlaylistType('WatchLater')
  await api.setPlaylistIndex(index)
}

async function removeFromList(videoId: string) {
  try {
    await api.removeFromWatchLater(videoId)
    results.value = results.value.filter(v => v.id !== videoId)
    totalCount.value--
    if (currentVideo.value?.id === videoId) {
      currentVideo.value = null
      currentVideoIndex.value = -1
    }
  } catch (e) {
    console.error('Failed to remove from watch later:', e)
  }
}

function promptRemove(video: Video) {
  pendingRemovalVideo.value = video
  showRemoveConfirm.value = true
}

function cancelRemove() {
  if (removing.value) return

  showRemoveConfirm.value = false
  pendingRemovalVideo.value = null
}

async function confirmRemove() {
  if (!pendingRemovalVideo.value || removing.value) return

  removing.value = true

  try {
    await removeFromList(pendingRemovalVideo.value.id)
    showRemoveConfirm.value = false
    pendingRemovalVideo.value = null
  } finally {
    removing.value = false
  }
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
let unlistenWatchLaterChanged: (() => void) | null = null

onMounted(async () => {
  // Restore view state, but always reload fresh list from DB instead of trusting stale Rust playlist entries
  try {
    const playlistState = await api.getPlaylistState()
    const watchLaterState = await api.getWatchLaterState()
    
    page.value = watchLaterState.page || 1
    pageSize.value = watchLaterState.page_size || 50
    sortOrder.value = (watchLaterState.sort_direction as 'desc' | 'asc') || 'desc'
    searchQuery.value = watchLaterState.search_query || ''
    pipActive.value = playlistState.pip_active
    
    await loadWatchLater()
    
    const initialViewState = getInitialPlaylistViewState({
      expectedPlaylistType: 'WatchLater',
      expectedPlaylistVersion: watchLaterState.version,
      playlistType: playlistState.playlist_type,
      playlistVersion: playlistState.playlist_version,
      playlistIndex: playlistState.index ?? -1,
      results: results.value,
    })

    if (initialViewState.selectedVideo && initialViewState.selectedIndex >= 0) {
      currentVideo.value = initialViewState.selectedVideo
      currentVideoIndex.value = initialViewState.selectedIndex
    }

    // Scroll to the currently playing video after state restoration
    await nextTick()
    const listContainer = document.querySelector('.video-list')
    scrollVideoIntoView(initialViewState.selectedIndex, listContainer as HTMLElement | null)
  } catch (e) {
    console.error('[WatchLaterView] Failed to restore state:', e)
    await loadWatchLater()
  }
  
  setupObserver()
  
  unlistenVideoSelected = await listen<VideoSelectedPayload>('video-selected', async (event) => {
    const payload = event.payload
    const latestPlaylistState = await api.getPlaylistState()
    if (
      !shouldApplyPlaylistSelection('WatchLater', payload) ||
      !shouldApplyPlaylistSelectionVersion(latestPlaylistState.playlist_version, payload)
    ) {
      currentVideo.value = null
      currentVideoIndex.value = -1
      return
    }
    currentVideoIndex.value = payload.index
    currentVideo.value = payload.video
    
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
  
  unlistenWatchLaterChanged = await listen('watch-later-changed', () => {
    loadWatchLater()
  })
})

onUnmounted(() => {
  if (observer) observer.disconnect()
  if (unlistenVideoSelected) unlistenVideoSelected()
  if (unlistenWatchLaterChanged) unlistenWatchLaterChanged()
  // Save state to Rust
  api.setWatchLaterState({
    page: page.value,
    page_size: pageSize.value,
    total_count: totalCount.value,
    has_next: hasNext.value,
    sort_direction: sortOrder.value,
    search_query: searchQuery.value,
    version: 0,
  }).catch(e => console.error('[WatchLaterView] Failed to save state:', e))
})

// Apply filters
function applyFilters() {
  page.value = 1
  loadWatchLater()
}

// Toggle sort order
function toggleSortOrder() {
  sortOrder.value = sortOrder.value === 'desc' ? 'asc' : 'desc'
  applyFilters()
}
</script>

<template>
  <div class="watch-later-view">
    <div class="list-column">
      <div class="header">
        <h2>❤️ {{ t('watchLater.title') }}</h2>
        <span class="count">{{ totalCount.toLocaleString() }} {{ t('watchLater.videos') }}</span>
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
      
      <div class="video-list">
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
            <div class="added-at">{{ formatAddedAt(video.start_time || '') }}</div>
          </div>
          
          <div class="actions">
            <button class="remove-btn" @click="promptRemove(video)" :title="t('watchLater.remove')">
              ✕
            </button>
          </div>
        </div>
        
        <div ref="observerTarget" class="scroll-trigger">
          <div v-if="loadingMore" class="spinner"></div>
          <span v-else-if="!hasNext && results.length > 0" class="end-message">{{ t('search.noMore') }}</span>
        </div>
      </div>
    </div>

    <div v-if="showRemoveConfirm" class="modal-backdrop" @click.self="cancelRemove">
      <div class="modal">
        <h3>{{ t('watchLater.confirmRemoveTitle') }}</h3>
        <p>
          {{ t('watchLater.confirmRemoveMessage') }}
          <strong v-if="pendingRemovalVideo">{{ pendingRemovalVideo.title }}</strong>
        </p>
        <div class="modal-actions">
          <button class="btn-secondary modal-btn" @click="cancelRemove" :disabled="removing">{{ t('watchLater.cancel') }}</button>
          <button class="btn-danger modal-btn modal-btn-danger" @click="confirmRemove" :disabled="removing">{{ t('watchLater.confirm') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.watch-later-view {
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

.added-at {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.actions {
  display: flex;
  align-items: center;
}

.remove-btn {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: transparent;
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
}

.remove-btn:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: rgb(239, 68, 68);
}

.scroll-trigger {
  padding: var(--space-md);
  text-align: center;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--color-border-subtle);
  border-top-color: var(--color-accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.end-message {
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--color-bg-surface);
  padding: 1.5rem;
  border-radius: 8px;
  max-width: 400px;
  width: min(400px, calc(100vw - 2rem));
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.28);
}

.modal h3 {
  margin: 0 0 0.75rem;
}

.modal p {
  color: var(--color-text-secondary);
  margin: 0 0 1.5rem;
  line-height: 1.5;
}

.modal strong {
  display: block;
  margin-top: 0.5rem;
  color: var(--color-text-primary);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

.modal-btn {
  min-width: 110px;
  padding: 0.72rem 1.05rem;
  border-radius: 10px;
  font-size: var(--font-size-sm);
  font-weight: 600;
  letter-spacing: 0.01em;
  border: 1px solid var(--color-border-subtle);
  transition:
    transform 0.16s ease,
    background-color 0.16s ease,
    border-color 0.16s ease,
    color 0.16s ease,
    box-shadow 0.16s ease,
    opacity 0.16s ease;
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.16);
}

.modal-btn:hover:not(:disabled) {
  transform: translateY(-1px);
}

.modal-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.btn-secondary.modal-btn {
  background: color-mix(in srgb, var(--color-bg-hover) 72%, var(--color-bg-surface) 28%);
  color: var(--color-text-primary);
}

.btn-secondary.modal-btn:hover:not(:disabled) {
  background: var(--color-bg-hover);
  border-color: var(--color-border-focus);
}

.modal-btn-danger {
  border-color: rgba(220, 53, 69, 0.42);
  background: linear-gradient(180deg, #ef5b70 0%, #dc3545 100%);
  color: white;
}

.modal-btn-danger:hover:not(:disabled) {
  background: linear-gradient(180deg, #f46b7f 0%, #e04252 100%);
  border-color: rgba(244, 107, 127, 0.48);
  box-shadow: 0 14px 30px rgba(220, 53, 69, 0.26);
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
