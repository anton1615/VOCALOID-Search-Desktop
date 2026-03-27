<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, watch, computed, nextTick } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-shell'
import { useI18n } from 'vue-i18n'
import { api, type PlaybackVideoUpdatedPayload, type Video, type VideoSelectedPayload, formatDuration, formatNumber, getUploaderAvatarUrl } from '../api/tauri-commands'
import UploaderAvatar from '../components/UploaderAvatar.vue'
import { buildSearchRequest, createSearchPersistenceState, restoreSearchPersistenceState } from '../features/playlistViews/searchViewState'
import { resolveSearchRestoreState } from '../features/playlistViews/searchRestoreState'
import { applyFormulaSelection, cancelFormulaSelection, selectSortOption, shouldPreloadMore, toggleSortDirection } from '../features/playlistViews/searchViewInteractions'
import {
  applyPlaybackMetadataUpdate,
  scrollVideoIntoView,
  shouldApplyPlaybackMetadataUpdate,
  shouldApplyPlaylistSelection,
  shouldApplyPlaylistSelectionVersion,
} from '../features/playlistViews/playlistViewState'
import { formatDateTime } from '../utils/dateTime'

const { t } = useI18n()

const SEARCH_STATE_KEY = 'vocaloidSearchState'

const query = ref('')
const loading = ref(false)
const results = ref<Video[]>([])
const totalCount = ref(0)
const page = ref(1)
const pageSize = ref(50)
const hasNext = ref(false)
const loadingMore = ref(false)

const currentVideo = ref<Video | null>(null)
const currentVideoIndex = ref(-1)

const sortField = ref('start_time')
const sortOrder = ref('desc')
const excludeWatched = ref(false)
const showAdvancedFilter = ref(false)
const showSortMenu = ref(false)
const showFormulaPanel = ref(false)

const sortWeights = reactive({ view: 5, mylist: 3, comment: 2, like: 1 })
const localWeights = ref({ view: 5, mylist: 3, comment: 2, like: 1 })

const showFormulaFilter = ref(false)
const formulaWeights = reactive({ view: 5, mylist: 3, comment: 2, like: 1 })
const formulaMinScore = ref(0)

const viewGte = ref<number | undefined>(undefined)
const viewLte = ref<number | undefined>(undefined)
const mylistGte = ref<number | undefined>(undefined)
const mylistLte = ref<number | undefined>(undefined)
const commentGte = ref<number | undefined>(undefined)
const commentLte = ref<number | undefined>(undefined)
const likeGte = ref<number | undefined>(undefined)
const likeLte = ref<number | undefined>(undefined)
const startTimeGte = ref('')
const startTimeLte = ref('')

const hasActiveFilters = computed(() => {
  return viewGte.value !== undefined || viewLte.value !== undefined ||
         mylistGte.value !== undefined || mylistLte.value !== undefined ||
         commentGte.value !== undefined || commentLte.value !== undefined ||
         likeGte.value !== undefined || likeLte.value !== undefined ||
         startTimeGte.value !== '' || startTimeLte.value !== '' ||
         (showFormulaFilter.value && formulaMinScore.value > 0)
})

const pipActive = ref(false)
const modalMouseDownOnBackdrop = ref(false)
const sortDropdownRef = ref<HTMLElement | null>(null)
const listContainerRef = ref<HTMLElement | null>(null)

function saveSearchState() {
  const state = createSearchPersistenceState({
    sortField: sortField.value,
    sortOrder: sortOrder.value,
    excludeWatched: excludeWatched.value,
    showFormulaFilter: showFormulaFilter.value,
    formulaWeights: { ...formulaWeights },
    formulaMinScore: formulaMinScore.value,
    sortWeights: { ...sortWeights },
    viewGte: viewGte.value,
    viewLte: viewLte.value,
    mylistGte: mylistGte.value,
    mylistLte: mylistLte.value,
    commentGte: commentGte.value,
    commentLte: commentLte.value,
    likeGte: likeGte.value,
    likeLte: likeLte.value,
    startTimeGte: startTimeGte.value,
    startTimeLte: startTimeLte.value,
  })
  localStorage.setItem(SEARCH_STATE_KEY, JSON.stringify(state))
}

function loadSearchState() {
  try {
    const saved = localStorage.getItem(SEARCH_STATE_KEY)
    if (saved) {
      const state = restoreSearchPersistenceState(JSON.parse(saved))
      sortField.value = state.sortField
      sortOrder.value = state.sortOrder
      excludeWatched.value = state.excludeWatched
      showFormulaFilter.value = state.showFormulaFilter
      formulaMinScore.value = state.formulaMinScore
      viewGte.value = state.viewGte
      viewLte.value = state.viewLte
      mylistGte.value = state.mylistGte
      mylistLte.value = state.mylistLte
      commentGte.value = state.commentGte
      commentLte.value = state.commentLte
      likeGte.value = state.likeGte
      likeLte.value = state.likeLte
      startTimeGte.value = state.startTimeGte
      startTimeLte.value = state.startTimeLte
      Object.assign(formulaWeights, state.formulaWeights)
      Object.assign(sortWeights, state.sortWeights)
    }
  } catch (e) {
    console.error('Failed to load search state', e)
  }
}

async function search() {
  loading.value = true
  await api.setSearchLoading(true)
  page.value = 1
  results.value = []
  currentVideoIndex.value = -1
  currentVideo.value = null

  try {
    const response = await api.search(buildSearchRequest({
      query: query.value,
      page: page.value,
      pageSize: pageSize.value,
      state: createSearchPersistenceState({
        sortField: sortField.value,
        sortOrder: sortOrder.value,
        excludeWatched: excludeWatched.value,
        showFormulaFilter: showFormulaFilter.value,
        formulaWeights: { ...formulaWeights },
        formulaMinScore: formulaMinScore.value,
        sortWeights: { ...sortWeights },
        viewGte: viewGte.value,
        viewLte: viewLte.value,
        mylistGte: mylistGte.value,
        mylistLte: mylistLte.value,
        commentGte: commentGte.value,
        commentLte: commentLte.value,
        likeGte: likeGte.value,
        likeLte: likeLte.value,
        startTimeGte: startTimeGte.value,
        startTimeLte: startTimeLte.value,
      }),
    }))
    results.value = response.results
    totalCount.value = response.total
    hasNext.value = response.has_next
  } catch (e) {
    console.error('Search error:', e)
  } finally {
    loading.value = false
    await api.setSearchLoading(false)
  }
}

async function loadMore() {
  // Prevent loadMore during active search
  if (loading.value) {
    return
  }
  if (loadingMore.value || !hasNext.value) return
  
  loadingMore.value = true
  
  try {
    const searchState = await api.getSearchState()
    await api.loadMore('Search', searchState.version)
    // Sync results from backend instead of appending
    const updatedState = await api.getSearchState()
    results.value = updatedState.results ?? []
    page.value = updatedState.page
    hasNext.value = updatedState.has_next
  } catch (e) {
    console.error('Load more error:', e)
  } finally {
    loadingMore.value = false
  }
}

async function playVideo(video: Video) {
  const index = results.value.findIndex(v => v.id === video.id)
  if (index >= 0) {
    await api.setPlaylistType('Search')
    await api.setPlaylistIndex(index)
  }
}

async function openNicoPage(event: Event, video: Video) {
  event.stopPropagation()
  if (video.watch_url) {
    await open(video.watch_url)
  }
}

function toggleSortOrder() {
  sortOrder.value = toggleSortDirection(sortOrder.value)
  search()
}

function resetFilters() {
  viewGte.value = undefined
  viewLte.value = undefined
  mylistGte.value = undefined
  mylistLte.value = undefined
  commentGte.value = undefined
  commentLte.value = undefined
  likeGte.value = undefined
  likeLte.value = undefined
  startTimeGte.value = ''
  startTimeLte.value = ''
  showFormulaFilter.value = false
  formulaMinScore.value = 0
  Object.assign(formulaWeights, { view: 5, mylist: 3, comment: 2, like: 1 })
}

function applyFilters() {
  showAdvancedFilter.value = false
  search()
}

function setDatePreset(days: number) {
  const today = new Date()
  const startDate = new Date(today)
  startDate.setDate(today.getDate() - (days - 1))
  
  const formatDate = (d: Date): string => d.toISOString().split('T')[0] || ''
  startTimeGte.value = formatDate(startDate)
  startTimeLte.value = formatDate(today)
}

const sortOptions = computed(() => [
  { value: 'start_time', label: t('filter.uploadDate') },
  { value: 'view', label: t('filter.viewCount') },
  { value: 'mylist', label: t('filter.mylistCount') },
  { value: 'comment', label: t('filter.commentCount') },
  { value: 'like', label: t('filter.likeCount') },
  { value: 'custom', label: t('filter.customFormula') },
])

const sortLabel = computed(() => {
  if (sortField.value === 'custom') {
    return `${t('filter.customFormula')} ${sortOrder.value === 'desc' ? '↓' : '↑'}`
  }
  const opt = sortOptions.value.find(o => o.value === sortField.value)
  return `${opt?.label || t('filter.uploadDate')} ${sortOrder.value === 'desc' ? '↓' : '↑'}`
})

function selectSort(field: string) {
  const next = selectSortOption(field, sortWeights)
  showSortMenu.value = next.showSortMenu
  showFormulaPanel.value = next.showFormulaPanel

  if (next.localWeights) {
    localWeights.value = next.localWeights
  }

  if (next.sortField) {
    sortField.value = next.sortField
  }

  if (next.shouldRunSearch) {
    search()
  }
}

function applyFormula() {
  const next = applyFormulaSelection(localWeights.value)
  Object.assign(sortWeights, next.sortWeights)
  sortField.value = next.sortField
  showFormulaPanel.value = next.showFormulaPanel
  if (next.shouldRunSearch) {
    search()
  }
}

function cancelFormula() {
  const next = cancelFormulaSelection()
  showFormulaPanel.value = next.showFormulaPanel
  localWeights.value = next.localWeights
}

function handleSortClickOutside(event: MouseEvent) {
  if (sortDropdownRef.value && !sortDropdownRef.value.contains(event.target as Node)) {
    showSortMenu.value = false
  }
}

function handleModalMouseDown(event: MouseEvent) {
  modalMouseDownOnBackdrop.value = (event.target as HTMLElement).classList.contains('modal-backdrop')
}

function handleModalMouseUp(event: MouseEvent, closeAction: () => void) {
  if (modalMouseDownOnBackdrop.value && (event.target as HTMLElement).classList.contains('modal-backdrop')) {
    closeAction()
  }
  modalMouseDownOnBackdrop.value = false
}

const observerTarget = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

function setupObserver() {
  if (observer) observer.disconnect()
  
  observer = new IntersectionObserver((entries) => {
    if (entries[0] && entries[0].isIntersecting && !loadingMore.value && hasNext.value) {
      loadMore()
    }
  }, {
    rootMargin: '100px 0px',
    threshold: 0.1
  })
  
  if (observerTarget.value) {
    observer.observe(observerTarget.value)
  }
}

let unlistenPip: (() => void) | null = null
let unlistenVideoSelected: (() => void) | null = null
let unlistenPlaybackVideoUpdated: (() => void) | null = null
let unlistenVideoWatched: (() => void) | null = null

onMounted(async () => {
  loadSearchState()
  document.addEventListener('click', handleSortClickOutside)
  setupObserver()

  // Check if Rust AppState has existing state before searching
  try {
    const playlistState = await api.getPlaylistState()
    const searchState = await api.getSearchState()
    
    const restored = resolveSearchRestoreState(playlistState, searchState)

    if (!restored.shouldRunInitialSearch) {
      results.value = restored.results
      currentVideoIndex.value = restored.currentVideoIndex
      currentVideo.value = restored.currentVideo
      hasNext.value = restored.hasNext
      totalCount.value = restored.totalCount
      page.value = restored.page
      pipActive.value = restored.pipActive
    } else {
      await search()
    }
  } catch (e) {
    console.error('[SearchView] Failed to restore state:', e)
    // Fallback to initial search on error
    await search()
  }

  // Scroll to the currently playing video after state restoration
  await nextTick()
  const listContainer = listContainerRef.value
  scrollVideoIntoView(currentVideoIndex.value, listContainer)

  unlistenPip = await listen('pip-closed', () => {
    pipActive.value = false
  })

  unlistenVideoSelected = await listen<VideoSelectedPayload>('video-selected', async (event) => {
    const payload = event.payload
    const latestPlaylistState = await api.getPlaylistState()
    if (
      !shouldApplyPlaylistSelection('Search', payload) ||
      !shouldApplyPlaylistSelectionVersion(latestPlaylistState.playlist_version, payload)
    ) {
      currentVideo.value = null
      currentVideoIndex.value = -1
      return
    }
    currentVideo.value = payload.video
    currentVideoIndex.value = payload.index

    // Scroll logic: keep videos visible above and below
    const videoElement = document.getElementById('video-' + payload.index)
    const prevVideoElement = document.getElementById('video-' + (payload.index - 1))
    const nextNextVideoElement = document.getElementById('video-' + (payload.index + 2))
    const listContainer = listContainerRef.value
    
    if (listContainer) {
      const containerRect = listContainer.getBoundingClientRect()
      
      // Check if we need to scroll up (previous video not visible)
      if (prevVideoElement && payload.index > 0) {
        const prevRect = prevVideoElement.getBoundingClientRect()
        if (prevRect.top < containerRect.top) {
          // Previous video is above visible area, scroll to show it
          prevVideoElement.scrollIntoView({ behavior: 'smooth', block: 'start' })
        }
      }
      
      // Check if we need to scroll down (video 2 positions below not visible)
      if (nextNextVideoElement) {
        const nextNextRect = nextNextVideoElement.getBoundingClientRect()
        if (nextNextRect.bottom > containerRect.bottom) {
          nextNextVideoElement.scrollIntoView({ behavior: 'smooth', block: 'end' })
        }
      } else if (videoElement) {
        // Less than 2 videos below, just scroll current into view
        const videoRect = videoElement.getBoundingClientRect()
        if (videoRect.bottom > containerRect.bottom || videoRect.top < containerRect.top) {
          videoElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
        }
      }
    }
    
    // Preload more results when approaching end of list
    if (shouldPreloadMore({
      resultsLength: results.value.length,
      index: payload.index,
      hasNext: hasNext.value,
      loadingMore: loadingMore.value,
    })) {
      loadMore()
    }

  })

  unlistenPlaybackVideoUpdated = await listen<PlaybackVideoUpdatedPayload>('playback-video-updated', async (event) => {
    const payload = event.payload
    const latestPlaylistState = await api.getPlaylistState()

    if (
      latestPlaylistState.playlist_type !== 'Search' ||
      latestPlaylistState.index !== currentVideoIndex.value ||
      !shouldApplyPlaybackMetadataUpdate({
        expectedPlaylistType: 'Search',
        expectedPlaylistVersion: latestPlaylistState.playlist_version,
        currentVideoIndex: currentVideoIndex.value,
        currentVideoId: latestPlaylistState.current_video_id,
        payload,
      }) ||
      !currentVideo.value
    ) {
      return
    }

    currentVideo.value = applyPlaybackMetadataUpdate(currentVideo.value, payload)
  })

  unlistenVideoWatched = await listen<{ video_id: string; is_watched: boolean }>('video-watched', (event) => {
    const { video_id, is_watched } = event.payload
    // Update results array
    const video = results.value.find(v => v.id === video_id)
    if (video) {
      video.is_watched = is_watched
    }
    // Update currentVideo if it matches
    if (currentVideo.value?.id === video_id) {
      currentVideo.value.is_watched = is_watched
    }
  })
})

onUnmounted(() => {
  document.removeEventListener('click', handleSortClickOutside)
  if (observer) observer.disconnect()
  if (unlistenPip) unlistenPip()
  if (unlistenVideoSelected) unlistenVideoSelected()
  if (unlistenPlaybackVideoUpdated) unlistenPlaybackVideoUpdated()
  if (unlistenVideoWatched) unlistenVideoWatched()
})

watch([
  sortField, sortOrder, excludeWatched, showFormulaFilter, formulaMinScore,
  viewGte, viewLte, mylistGte, mylistLte, commentGte, commentLte,
  likeGte, likeLte, startTimeGte, startTimeLte
], saveSearchState, { deep: true })

watch(formulaWeights, () => saveSearchState(), { deep: true })
watch(sortWeights, () => saveSearchState(), { deep: true })
</script>

<template>
  <div class="search-view">
    <div class="list-column">
      <div class="search-container">
        <div class="search-input-wrapper">
          <span class="search-icon">🔍</span>
          <input
            v-model="query"
            type="text"
            :placeholder="t('search.placeholder')"
            class="search-input"
            @keyup.enter="search"
          />
          <button v-if="query" class="clear-btn" @click="query = ''">✕</button>
        </div>
        <button class="search-btn" :disabled="loading" @click="search">
          <span v-if="loading" class="spinner"></span>
          <span v-else>{{ t('app.search') }}</span>
        </button>
      </div>
      
      <div class="filter-bar">
        <div class="filter-scroll">
          <div class="sort-dropdown" ref="sortDropdownRef">
            <button class="filter-pill active" @click.stop="showSortMenu = !showSortMenu">
              <span class="icon">⇅</span> {{ sortLabel }}
            </button>
          </div>

          <button class="filter-pill" @click="toggleSortOrder">
            {{ sortOrder === 'desc' ? `↓ ${t('filter.desc')}` : `↑ ${t('filter.asc')}` }}
          </button>
          <button 
            class="filter-pill" 
            :class="{ active: excludeWatched }"
            @click="excludeWatched = !excludeWatched; search()"
          >
            <span v-if="excludeWatched">👁️</span> {{ t('filter.hideWatched') }}
          </button>
          <button 
            class="filter-more" 
            :class="{ 'has-filters': hasActiveFilters }"
            @click="showAdvancedFilter = true"
          >
            ⚙️ {{ t('filter.advancedFilter') }} <span v-if="hasActiveFilters" class="badge"></span>
          </button>
        </div>
      </div>
      
      <Teleport to="body">
        <div 
          v-if="showSortMenu" 
          class="sort-menu-portal"
          :style="{ 
            position: 'fixed',
            top: sortDropdownRef ? `${sortDropdownRef.getBoundingClientRect().bottom + 4}px` : '0',
            left: sortDropdownRef ? `${sortDropdownRef.getBoundingClientRect().left}px` : '0',
          }"
        >
          <button 
            v-for="opt in sortOptions" 
            :key="opt.value"
            class="sort-option"
            :class="{ selected: sortField === opt.value }"
            @click="selectSort(opt.value)"
          >
            {{ opt.label }}
          </button>
        </div>
      </Teleport>
      
      <div v-if="totalCount > 0" class="result-count">
        {{ t('search.results', { count: totalCount.toLocaleString() }) }}
      </div>
      
      <div class="video-list" ref="listContainerRef">
        <div
          v-for="(video, idx) in results"
          :key="video.id"
          :id="'video-' + idx"
          class="video-item"
          :class="{ playing: currentVideo?.id === video.id, watched: video.is_watched }"
        >
          <div class="left-column">
            <div class="rank">{{ idx + 1 }}</div>
            <UploaderAvatar
              :src="getUploaderAvatarUrl(video.uploader_id)"
              :alt="video.uploader_name || video.uploader_id || 'Uploader avatar'"
              class="uploader-avatar"
            />
          </div>
          
          <div class="thumbnail-wrapper" @click="playVideo(video)">
            <img :src="video.thumbnail_url || ''" class="thumbnail" loading="lazy" />
            <span v-if="video.duration" class="duration">{{ formatDuration(video.duration) }}</span>
            <span v-if="video.is_watched" class="watched-badge">已看</span>
          </div>
          
          <div class="video-info" @click="playVideo(video)">
            <div class="title-row">
              <h3 class="title">{{ video.title }}</h3>
              <span class="upload-datetime">{{ formatDateTime(video.start_time) }}</span>
            </div>
            
            <div class="stats-row">
              <span class="stat views">▶ {{ formatNumber(video.view_count) }}</span>
              <span class="stat likes">❤️ {{ formatNumber(video.like_count) }}</span>
              <span class="stat mylists">📝 {{ formatNumber(video.mylist_count) }}</span>
              <span class="stat comments">💬 {{ formatNumber(video.comment_count) }}</span>
            </div>
            
            <div class="tags-row" v-if="video.tags && video.tags.length > 0">
              <span v-for="tag in video.tags.slice(0, 4)" :key="tag" class="tag">{{ tag }}</span>
              <span v-if="video.tags.length > 4" class="tag more">+{{ video.tags.length - 4 }}</span>
            </div>
          </div>
          
          <div class="actions">
            <button class="nico-btn" @click="openNicoPage($event, video)" title="在 niconico 開啟">
              🔗
            </button>
            <div class="playing-bars" v-if="currentVideo?.id === video.id">
              <span class="bar"></span>
              <span class="bar"></span>
              <span class="bar"></span>
            </div>
          </div>
        </div>
        
        <div ref="observerTarget" class="scroll-trigger">
          <div v-if="loadingMore" class="spinner"></div>
          <span v-else-if="!hasNext && results.length > 0" class="end-message">{{ t('search.noResults') }}</span>
        </div>
      </div>
    </div>

    <!-- Advanced Filter Modal -->
    <div v-if="showAdvancedFilter" class="modal-backdrop" 
         @mousedown="handleModalMouseDown"
         @mouseup="handleModalMouseUp($event, () => showAdvancedFilter = false)">
      <div class="modal-content modal-large">
        <h2>{{ t('filter.advancedFilter') }}</h2>
        <button class="close-btn" @click="showAdvancedFilter = false">✕</button>
        
        <div class="filter-form">
          <div class="form-group">
            <label>{{ t('filter.views') }}</label>
            <div class="range-inputs">
              <input type="number" v-model.number="viewGte" :placeholder="t('filter.min')"> - 
              <input type="number" v-model.number="viewLte" :placeholder="t('filter.max')">
            </div>
          </div>
          <div class="form-group">
            <label>{{ t('filter.mylists') }}</label>
            <div class="range-inputs">
              <input type="number" v-model.number="mylistGte" :placeholder="t('filter.min')"> - 
              <input type="number" v-model.number="mylistLte" :placeholder="t('filter.max')">
            </div>
          </div>
          <div class="form-group">
            <label>{{ t('filter.comments') }}</label>
            <div class="range-inputs">
              <input type="number" v-model.number="commentGte" :placeholder="t('filter.min')"> - 
              <input type="number" v-model.number="commentLte" :placeholder="t('filter.max')">
            </div>
          </div>
          <div class="form-group">
            <label>{{ t('filter.likes') }}</label>
            <div class="range-inputs">
              <input type="number" v-model.number="likeGte" :placeholder="t('filter.min')"> - 
              <input type="number" v-model.number="likeLte" :placeholder="t('filter.max')">
            </div>
          </div>
          <div class="form-group">
            <label>{{ t('filter.uploadDateRange') }}</label>
            <div class="range-inputs">
              <input type="date" v-model="startTimeGte"> - 
              <input type="date" v-model="startTimeLte">
            </div>
            <div class="date-presets">
              <button class="preset-btn" @click="setDatePreset(2)">{{ t('filter.recent2days') }}</button>
              <button class="preset-btn" @click="setDatePreset(7)">{{ t('filter.recent7days') }}</button>
              <button class="preset-btn" @click="setDatePreset(30)">{{ t('filter.recent30days') }}</button>
              <button class="preset-btn" @click="setDatePreset(365)">{{ t('filter.recent1year') }}</button>
            </div>
          </div>

          <div class="divider"></div>

          <div class="form-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="showFormulaFilter">
              {{ t('filter.formulaFilter') }}
            </label>
          </div>

          <div v-if="showFormulaFilter" class="formula-section">
            <div class="formula-weights">
              <div class="form-group inline">
                <label>👁 {{ t('filter.views') }}</label>
                <input type="number" v-model.number="formulaWeights.view" min="0" max="10" step="0.5">
              </div>
              <div class="form-group inline">
                <label>📝 {{ t('filter.mylists') }}</label>
                <input type="number" v-model.number="formulaWeights.mylist" min="0" max="10" step="0.5">
              </div>
              <div class="form-group inline">
                <label>💬 {{ t('filter.comments') }}</label>
                <input type="number" v-model.number="formulaWeights.comment" min="0" max="10" step="0.5">
              </div>
              <div class="form-group inline">
                <label>❤️ {{ t('filter.likes') }}</label>
                <input type="number" v-model.number="formulaWeights.like" min="0" max="10" step="0.5">
              </div>
            </div>
            <div class="form-group">
              <label>{{ t('filter.minScore') }}</label>
              <input type="number" v-model.number="formulaMinScore" min="0">
            </div>
            <div class="formula-preview">
              {{ t('filter.formulaPreview', { view: formulaWeights.view, mylist: formulaWeights.mylist, comment: formulaWeights.comment, like: formulaWeights.like }) }}
            </div>
          </div>
        </div>
        
        <div class="modal-actions">
          <button class="btn-secondary" @click="resetFilters">{{ t('filter.reset') }}</button>
          <button class="btn-primary" @click="applyFilters">{{ t('filter.applyAndSearch') }}</button>
        </div>
      </div>
    </div>
    
    <!-- Custom Formula Panel -->
    <div v-if="showFormulaPanel" class="modal-backdrop"
         @mousedown="handleModalMouseDown"
         @mouseup="handleModalMouseUp($event, cancelFormula)">
      <div class="formula-panel">
        <div class="formula-header">
          <h3>{{ t('filter.customFormula') }}</h3>
          <button class="close-btn" @click="cancelFormula">✕</button>
        </div>
        
        <div class="formula-content">
          <p class="formula-hint">{{ t('filter.formulaWeights') }}</p>
          
          <div class="weight-inputs">
            <div class="weight-row">
              <label>👁 {{ t('filter.views') }}</label>
              <input type="number" v-model.number="localWeights.view" min="0" max="10" step="0.5">
            </div>
            <div class="weight-row">
              <label>📝 {{ t('filter.mylists') }}</label>
              <input type="number" v-model.number="localWeights.mylist" min="0" max="10" step="0.5">
            </div>
            <div class="weight-row">
              <label>💬 {{ t('filter.comments') }}</label>
              <input type="number" v-model.number="localWeights.comment" min="0" max="10" step="0.5">
            </div>
            <div class="weight-row">
              <label>❤️ {{ t('filter.likes') }}</label>
              <input type="number" v-model.number="localWeights.like" min="0" max="10" step="0.5">
            </div>
          </div>

          <div class="formula-preview">
            {{ t('filter.formulaPreview', { view: localWeights.view, mylist: localWeights.mylist, comment: localWeights.comment, like: localWeights.like }) }}
          </div>
        </div>

        <div class="formula-actions">
          <button class="btn-secondary" @click="cancelFormula">{{ t('filter.reset') }}</button>
          <button class="btn-primary" @click="applyFormula">{{ t('filter.apply') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.search-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 0;
  background: var(--color-bg-primary);
}

.list-column {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  background: var(--color-bg-primary);
  overflow: hidden;
}

.search-container {
  display: flex;
  gap: var(--space-sm);
  padding: var(--space-md);
  background: var(--color-bg-primary);
  position: sticky;
  top: 0;
  z-index: 5;
  border-bottom: 1px solid var(--color-border-subtle);
}

.search-input-wrapper {
  flex: 1;
  display: flex;
  align-items: center;
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: 20px;
  padding: 0 var(--space-md);
  height: 40px;
  transition: border-color 0.2s;
}

.search-input-wrapper:focus-within {
  border-color: var(--color-border-focus);
}

.search-icon {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
  margin-right: var(--space-sm);
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--color-text-primary);
  font-size: var(--font-size-base);
  outline: none;
  min-width: 0;
}

.search-input::placeholder {
  color: var(--color-text-muted);
}

.clear-btn {
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
  padding: var(--space-xs);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  cursor: pointer;
  background: transparent;
  border: none;
}

.clear-btn:hover {
  color: var(--color-text-primary);
  background: var(--color-bg-hover);
}

.search-btn {
  height: 40px;
  padding: 0 var(--space-lg);
  border-radius: 20px;
  background: var(--color-accent-primary);
  color: var(--color-bg-primary);
  font-weight: 600;
  font-size: var(--font-size-sm);
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 72px;
  border: none;
  cursor: pointer;
}

.search-btn:hover:not(:disabled) {
  background: var(--color-accent-secondary);
}

.search-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(0,0,0,0.2);
  border-top-color: var(--color-bg-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.filter-bar {
  display: flex;
  background: var(--color-bg-surface);
  border-bottom: 1px solid var(--color-border-subtle);
  overflow: visible;
  position: relative;
}

.filter-scroll {
  display: flex;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  min-width: min-content;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.filter-scroll::-webkit-scrollbar {
  display: none;
}

.filter-pill, .filter-more {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: 6px var(--space-md);
  border-radius: 16px;
  border: 1px solid var(--color-border-subtle);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 500;
  white-space: nowrap;
  transition: all 0.2s ease;
  height: 32px;
  cursor: pointer;
  flex-shrink: 0;
}

.filter-pill:hover, .filter-more:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.filter-pill.active {
  background: var(--color-accent-primary);
  border-color: var(--color-accent-primary);
  color: var(--color-bg-primary);
}

.filter-more.has-filters {
  border-color: var(--color-accent-secondary);
  color: var(--color-accent-secondary);
}

.badge {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-accent-secondary);
  display: inline-block;
}

.filter-select {
  padding: 6px var(--space-md);
  border-radius: 16px;
  border: 1px solid var(--color-border-subtle);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 500;
  height: 32px;
  cursor: pointer;
  flex-shrink: 0;
}

.result-count {
  padding: var(--space-xs) var(--space-md);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  background: var(--color-bg-surface);
  border-bottom: 1px solid var(--color-border-subtle);
}

.filters {
  display: flex;
  gap: var(--space-sm);
  margin-top: var(--space-sm);
  flex-wrap: wrap;
}

.filters select {
  padding: var(--space-xs) var(--space-sm);
  border-radius: 4px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
}

.checkbox {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.result-count {
  padding: var(--space-xs) var(--space-md);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  background: var(--color-bg-surface);
  border-bottom: 1px solid var(--color-border-subtle);
}

.video-list {
  flex: 1;
  overflow-y: auto;
}

.video-item {
  display: flex;
  align-items: flex-start;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border-subtle);
  gap: 12px;
  transition: background-color 0.2s;
  min-height: 88px;
}

.video-item:hover {
  background: var(--color-bg-hover);
}

.video-item.playing {
  background: rgba(20, 184, 166, 0.1);
  border-left: 3px solid var(--color-accent-primary);
  padding-left: 13px;
}

.video-item.watched {
  opacity: 0.7;
}

.left-column {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  width: 36px;
}

.rank {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.playing .rank {
  color: var(--color-accent-primary);
}

.uploader-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  object-fit: cover;
  background: var(--color-bg-surface);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  border: 2px solid var(--color-border-subtle);
}

.default-avatar {
  color: var(--color-text-muted);
}

.thumbnail-wrapper {
  position: relative;
  flex-shrink: 0;
  cursor: pointer;
}

.thumbnail {
  width: 120px;
  height: 68px;
  object-fit: cover;
  border-radius: 6px;
  background: var(--color-bg-surface);
}

.duration {
  position: absolute;
  bottom: 4px;
  right: 4px;
  background: rgba(0, 0, 0, 0.85);
  color: #fff;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 500;
}

.watched-badge {
  position: absolute;
  top: 4px;
  left: 4px;
  background: var(--color-accent-primary);
  color: #fff;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 600;
}

.video-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
  cursor: pointer;
}

.title-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
}

.title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: 1.35;
  flex: 1;
}

.upload-datetime {
  font-size: 15px;
  color: var(--color-text-secondary-light);
  white-space: nowrap;
  flex-shrink: 0;
}

.playing .title {
  color: var(--color-accent-primary);
}

.stats-row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.stat {
  font-size: 13px;
  font-weight: 500;
}

.stat.views { color: var(--color-stat-views); }
.stat.mylists { color: var(--color-stat-mylists); }
.stat.likes { color: var(--color-stat-likes); }
.stat.comments { color: var(--color-stat-comments); }

.tags-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag {
  font-size: 11px;
  padding: 2px 8px;
  background: var(--color-bg-surface);
  color: var(--color-text-secondary);
  border-radius: 4px;
  border: 1px solid var(--color-border-subtle);
}

.tag.more {
  background: transparent;
  border-style: dashed;
  color: var(--color-text-muted);
}

.actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  justify-content: center;
}

.nico-btn {
  width: 36px;
  height: 36px;
  color: var(--color-text-secondary);
  font-size: 18px;
  opacity: 0.6;
  transition: opacity 0.2s;
  padding: 8px;
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.video-item:hover .nico-btn {
  opacity: 1;
}

.nico-btn:hover {
  color: var(--color-accent-primary);
  border-color: var(--color-accent-primary);
}

.playing-bars {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  height: 16px;
}

.bar {
  width: 4px;
  background-color: var(--color-accent-primary);
  animation: bounce 1.2s infinite ease-in-out;
}

.bar:nth-child(1) { height: 60%; animation-delay: -0.2s; }
.bar:nth-child(2) { height: 100%; animation-delay: -0.1s; }
.bar:nth-child(3) { height: 80%; animation-delay: 0s; }

@keyframes bounce {
  0%, 100% { transform: scaleY(0.5); }
  50% { transform: scaleY(1); }
}

.scroll-trigger {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
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

.end-message {
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.player-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-surface);
  overflow: hidden;
}

.player-header {
  padding: var(--space-md);
  border-bottom: 1px solid var(--color-border-subtle);
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

.empty-player {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
  background: var(--color-bg-primary);
}

.playback-controls {
  background: var(--color-bg-surface);
  border-bottom: 1px solid var(--color-border-subtle);
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

.pip-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  gap: 1rem;
}

.pip-placeholder button {
  padding: var(--space-sm) var(--space-md);
  border-radius: 4px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  cursor: pointer;
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

.tags-section .tag {
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  background: var(--color-bg-hover);
  padding: 3px 8px;
  border-radius: 4px;
  white-space: nowrap;
}

.tags-section .tag.more {
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

/* Modal styles */
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-content {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: 8px;
  padding: var(--space-xl);
  width: 90%;
  max-width: 400px;
  position: relative;
  box-shadow: var(--shadow-md);
  max-height: 90vh;
  overflow-y: auto;
}

.modal-content h2 {
  font-size: var(--font-size-lg);
  color: var(--color-text-primary);
  margin-bottom: var(--space-xl);
}

.close-btn {
  position: absolute;
  top: var(--space-md);
  right: var(--space-md);
  font-size: var(--font-size-lg);
  color: var(--color-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
}

.filter-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  margin-bottom: var(--space-xl);
}

.form-group label {
  display: block;
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: var(--space-xs);
}

.range-inputs {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  color: var(--color-text-muted);
}

.range-inputs input {
  flex: 1;
  width: 0;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary);
  padding: 8px;
  border-radius: 4px;
}

.range-inputs input:focus {
  border-color: var(--color-border-focus);
  outline: none;
}

.date-presets {
  display: flex;
  gap: var(--space-xs);
  margin-top: var(--space-sm);
  flex-wrap: wrap;
}

.preset-btn {
  padding: 4px 10px;
  border-radius: 12px;
  border: 1px solid var(--color-border-subtle);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-xs);
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.preset-btn:hover {
  background: var(--color-accent-primary);
  border-color: var(--color-accent-primary);
  color: var(--color-bg-primary);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-md);
}

.btn-primary, .btn-secondary {
  padding: 8px 16px;
  border-radius: 4px;
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
}

.btn-primary {
  background: var(--color-accent-primary);
  color: var(--color-bg-primary);
  border: none;
}

.btn-primary:hover {
  background: var(--color-accent-secondary);
}

.btn-secondary {
  background: transparent;
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-secondary);
}

.btn-secondary:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.modal-content.modal-large {
  max-width: 500px;
}

.divider {
  height: 1px;
  background: var(--color-border-subtle);
  margin: var(--space-md) 0;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  cursor: pointer;
  font-size: var(--font-size-base);
  color: var(--color-text-primary);
}

.checkbox-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: var(--color-accent-primary);
}

.formula-section {
  background: var(--color-bg-primary);
  padding: var(--space-md);
  border-radius: 6px;
  border: 1px solid var(--color-border-subtle);
}

.formula-weights {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--space-sm);
  margin-bottom: var(--space-md);
}

.form-group.inline {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.form-group.inline label {
  margin-bottom: 0;
  min-width: 60px;
}

.form-group.inline input {
  width: 60px;
  padding: 4px 8px;
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary);
  border-radius: 4px;
}

.sort-dropdown {
  position: relative;
  flex-shrink: 0;
}

.sort-menu-portal {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: 8px;
  padding: 4px;
  z-index: 9999;
  min-width: 120px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
}

.sort-option {
  display: block;
  width: 100%;
  padding: 8px 12px;
  text-align: left;
  background: transparent;
  color: var(--color-text-secondary);
  border: none;
  border-radius: 4px;
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all 0.15s;
}

.sort-option:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.sort-option.selected {
  background: rgba(20, 184, 166, 0.15);
  color: var(--color-accent-primary);
  font-weight: 500;
}

.resize-divider {
  width: 6px;
  background: var(--color-border-subtle);
  cursor: col-resize;
  flex-shrink: 0;
  transition: background 0.2s;
}

.resize-divider:hover,
.resize-divider.dragging {
  background: var(--color-accent-primary);
}

.formula-panel {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: 12px;
  width: 90%;
  max-width: 420px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.formula-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border-subtle);
}

.formula-header h3 {
  margin: 0;
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--color-text-primary);
}

.formula-content {
  padding: 20px;
}

.formula-hint {
  margin: 0 0 16px 0;
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.weight-inputs {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.weight-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.weight-row label {
  font-size: var(--font-size-base);
  color: var(--color-text-primary);
}

.weight-row input {
  width: 80px;
  padding: 8px 12px;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border-subtle);
  border-radius: 6px;
  color: var(--color-text-primary);
  font-size: var(--font-size-base);
  text-align: center;
}

.weight-row input:focus {
  outline: none;
  border-color: var(--color-accent-primary);
}

.formula-preview {
  margin-top: 16px;
  padding: 12px;
  background: var(--color-bg-primary);
  border-radius: 6px;
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  text-align: center;
  font-family: monospace;
  word-break: break-all;
}

.formula-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--color-border-subtle);
}
</style>
