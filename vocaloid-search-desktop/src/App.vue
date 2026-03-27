<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, provide, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

import { api, type PlaylistType, type Video } from './api/tauri-commands'
import PlayerColumn from './components/PlayerColumn.vue'
import { useAuthoritativePlaybackSync } from './composables/useAuthoritativePlaybackSync'
import { i18n } from './i18n'
import { useLocaleStore, type Locale } from './stores/locale'
import { useThemeStore } from './stores/theme'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const themeStore = useThemeStore()
const localeStore = useLocaleStore()
const isLoading = ref(true)
const listWidth = ref(40)
const isDragging = ref(false)
const splitLayoutRef = ref<HTMLElement | null>(null)
const currentVideo = ref<Video | null>(null)
const currentVideoIndex = ref(-1)
const resultsCount = ref(0)
const hasNext = ref(false)
const pipActive = ref(false)
const playlistType = ref<PlaylistType>('Search')
const playlistVersion = ref(1)
const freshnessStatus = ref({
  message: '',
  isFresh: true,
  localLastUpdate: null as string | null,
  apiLastUpdate: null as string | null,
})
const shouldRedirectToScraper = ref(false)

provide('freshnessStatus', freshnessStatus)

const localeOptions: { value: Locale; label: string }[] = [
  { value: 'ja', label: '日本語' },
  { value: 'en', label: 'English' },
  { value: 'zh-TW', label: '中文' },
]

watch(() => localeStore.locale, (newLocale) => {
  i18n.global.locale.value = newLocale
})

watch(() => route.name, async (routeName, previousRouteName) => {
  if (routeName === 'scraper' && previousRouteName !== 'scraper') {
    try {
      await api.resetPlaybackForSyncRouteEntry()
      await refreshActivePlayback()
    } catch (error) {
      console.error('Failed to reset playback on sync route entry:', error)
    }
  }
})

const showsSplitLayout = computed(() => route.name !== 'scraper')

async function syncActivePlayback(state: {
  currentVideo: Video | null
  currentVideoIndex: number
  resultsCount: number
  hasNext: boolean
  pipActive?: boolean
  playlistType: PlaylistType
  playlistVersion: number
}) {
  resultsCount.value = state.resultsCount
  hasNext.value = state.hasNext
  pipActive.value = state.pipActive ?? false
  playlistType.value = state.playlistType
  playlistVersion.value = state.playlistVersion

  if (state.currentVideo) {
    currentVideoIndex.value = state.currentVideoIndex
    currentVideo.value = state.currentVideo
  } else {
    currentVideoIndex.value = -1
    currentVideo.value = null
  }
}

const { refreshActivePlayback } = useAuthoritativePlaybackSync(syncActivePlayback)

function handlePlaybackStateChanged() {
  void refreshActivePlayback()
}

async function handlePipOwnershipRegained() {
  await api.reenterActivePlaybackMetadata()
}

async function handlePipClosed() {
  await handlePipOwnershipRegained()
}

function handleFreshnessCheckError(error: unknown) {
  console.error('Failed to check freshness:', error)
}

async function initializeAppState() {
  const freshness = await invoke<{ is_fresh: boolean; message: string; local_last_update?: string | null; api_last_update?: string | null }>('check_database_freshness')

  freshnessStatus.value = {
    message: freshness.message,
    isFresh: freshness.is_fresh,
    localLastUpdate: freshness.local_last_update ?? null,
    apiLastUpdate: freshness.api_last_update ?? null,
  }

  await refreshActivePlayback()

  if (!freshness.is_fresh) {
    shouldRedirectToScraper.value = true
    router.push('/scraper')
  }
}

async function playNext() {
  if (currentVideoIndex.value < 0) return

  await api.playNext()
  await refreshActivePlayback()
}

async function playPrevious() {
  if (currentVideoIndex.value < 0) return

  await api.playPrevious()
  await refreshActivePlayback()
}

async function openPip() {
  if (!currentVideo.value) return
  await api.openPipWindow()
  pipActive.value = true
}

async function closePip() {
  try {
    await api.closePipWindow()
  } catch (e) {
    console.error('Failed to close PiP window:', e)
  }
}

function handleVideoWatched(video: Video) {
  if (currentVideo.value?.id === video.id) {
    currentVideo.value = video
  }
}

function startDrag() {
  isDragging.value = true
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

function onDrag(e: MouseEvent) {
  if (!isDragging.value || !splitLayoutRef.value) return
  const rect = splitLayoutRef.value.getBoundingClientRect()
  const newWidth = ((e.clientX - rect.left) / rect.width) * 100
  listWidth.value = Math.min(60, Math.max(25, newWidth))
}

function stopDrag() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}

let unlistenPipClosed: (() => void) | null = null

onMounted(async () => {
  try {
    await initializeAppState()
    unlistenPipClosed = await listen('pip-closed', handlePipClosed)
  } catch (e) {
    handleFreshnessCheckError(e)
  } finally {
    isLoading.value = false
  }
})

onUnmounted(() => {
  if (unlistenPipClosed) unlistenPipClosed()
  stopDrag()
})
</script>

<template>
  <div class="app-container" :data-theme="themeStore.theme">
    <nav class="sidebar">
      <div class="logo">
        <h2>VOCALOID Search</h2>
      </div>
      <router-link to="/" class="nav-item">
        <span class="icon">🔍</span>
        <span>{{ t('app.search') }}</span>
      </router-link>
      <router-link to="/history" class="nav-item">
        <span class="icon">📜</span>
        <span>{{ t('app.history') }}</span>
      </router-link>
      <router-link to="/watch-later" class="nav-item">
        <span class="icon">❤️</span>
        <span>{{ t('app.watchLater') }}</span>
      </router-link>
      <router-link to="/scraper" class="nav-item">
        <span class="icon">🔄</span>
        <span>{{ t('app.scraper') }}</span>
      </router-link>
      <div class="nav-footer">
        <div class="settings-row">
          <button class="theme-toggle" @click="themeStore.toggle()" :title="themeStore.theme === 'dark' ? t('app.lightMode') : t('app.darkMode')">
            <span v-if="themeStore.theme === 'dark'">☀️</span>
            <span v-else>🌙</span>
          </button>
          <select class="locale-select" v-model="localeStore.locale">
            <option v-for="opt in localeOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </div>
      </div>
    </nav>

    <main class="main-content">
      <div v-if="isLoading" class="loading">
        <div class="spinner"></div>
        <p>載入中...</p>
      </div>
      <template v-else-if="showsSplitLayout">
        <div class="split-layout" ref="splitLayoutRef">
          <div class="list-pane" :style="{ width: `${listWidth}%`, minWidth: '320px', maxWidth: '60%' }">
            <router-view />
          </div>

          <div
            class="resize-divider"
            @mousedown="startDrag"
            :class="{ dragging: isDragging }"
          ></div>

          <PlayerColumn
            :current-video="currentVideo"
            :current-video-index="currentVideoIndex"
            :results-count="resultsCount"
            :has-next="hasNext"
            :playlist-type="playlistType"
            :playlist-version="playlistVersion"
            :pip-active="pipActive"
            :show-auto-skip="true"
            @play-next="playNext"
            @play-previous="playPrevious"
            @open-pip="openPip"
            @close-pip="closePip"
            @video-watched="handleVideoWatched"
            @playback-state-changed="handlePlaybackStateChanged"
          />
        </div>
      </template>
      <router-view v-else />
    </main>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  background: var(--bg-primary);
}

.sidebar {
  width: 200px;
  background: var(--bg-surface);
  border-right: 1px solid var(--border-color);
  padding: 1rem;
  display: flex;
  flex-direction: column;
}

.logo h2 {
  font-size: 1.1rem;
  color: var(--text-primary);
  margin-bottom: 2rem;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: all 0.2s;
  margin-bottom: 0.25rem;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.router-link-active {
  background: var(--accent-primary);
  color: white;
}

.icon {
  font-size: 1.2rem;
}

.main-content {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.split-layout {
  display: flex;
  height: 100%;
  width: 100%;
  min-width: 0;
  background: var(--color-bg-primary);
}

.list-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  height: 100%;
  overflow: hidden;
}

.resize-divider {
  width: 4px;
  flex-shrink: 0;
  cursor: col-resize;
  background: var(--color-border-subtle);
  transition: background 0.2s;
}

.resize-divider:hover,
.resize-divider.dragging {
  background: var(--color-accent-primary);
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.nav-footer {
  margin-top: auto;
  padding-top: 1rem;
  border-top: 1px solid var(--border-color);
}

.settings-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.theme-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: var(--bg-hover);
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 18px;
}

.theme-toggle:hover {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
}

.locale-select {
  padding: 6px 10px;
  border-radius: 8px;
  background: var(--bg-hover);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.locale-select:hover {
  border-color: var(--accent-primary);
}

.locale-select:focus {
  outline: none;
  border-color: var(--accent-primary);
}
</style>
