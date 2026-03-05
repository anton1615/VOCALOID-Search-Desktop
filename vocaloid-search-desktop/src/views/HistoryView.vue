<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { api, type HistoryEntry, type Video, type UserInfo } from '../api/tauri-commands'

const { t } = useI18n()

const results = ref<HistoryEntry[]>([])
const totalCount = ref(0)
const page = ref(1)
const pageSize = ref(50)
const hasNext = ref(false)
const loading = ref(false)

const currentVideo = ref<Video | null>(null)
const loadingVideo = ref(false)
const videoError = ref<string | null>(null)

const userInfoCache = new Map<string, UserInfo>()

async function loadHistory() {
  loading.value = true
  try {
    const response = await api.getHistory(page.value, pageSize.value)
    results.value = response.results
    totalCount.value = response.total
    hasNext.value = response.has_next
  } catch (e) {
    console.error('Failed to load history:', e)
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  if (!hasNext.value) return
  page.value++
  try {
    const response = await api.getHistory(page.value, pageSize.value)
    results.value = [...results.value, ...response.results]
    hasNext.value = response.has_next
  } catch (e) {
    console.error('Failed to load more:', e)
    page.value--
  }
}

async function playFromHistory(entry: HistoryEntry) {
  loadingVideo.value = true
  videoError.value = null
  currentVideo.value = null
  
  try {
    const metadata = await api.fetchVideoMetadata(entry.video_id)
    
    if (!metadata) {
      videoError.value = t('history.videoNotFound')
      loadingVideo.value = false
      return
    }
    
    const userInfo = await api.getUserInfo(entry.video_id)
    if (userInfo) {
      userInfoCache.set(entry.video_id, userInfo)
    }
    
    currentVideo.value = {
      ...metadata,
      is_watched: true,
      uploader_name: userInfo?.user_nickname || metadata.uploader_name
    }
    
    const watchUrl = `https://www.nicovideo.jp/watch/${entry.video_id}`
    window.open(watchUrl, '_blank')
  } catch (e) {
    console.error('Failed to play from history:', e)
    videoError.value = t('history.fetchError')
  } finally {
    loadingVideo.value = false
  }
}

onMounted(loadHistory)
</script>

<template>
  <div class="history-view">
    <div class="header">
      <h2>{{ t('history.title') }}</h2>
      <span class="count">{{ totalCount.toLocaleString() }} {{ t('history.videos') }}</span>
    </div>
    
    <div v-if="loading" class="loading">{{ t('history.loading') }}</div>
    
    <div v-else class="video-list">
      <div
        v-for="video in results"
        :key="video.video_id"
        class="video-item"
        @click="playFromHistory(video)"
      >
        <img :src="video.thumbnail_url || ''" class="thumbnail" loading="lazy" />
        <div class="info">
          <div class="title">{{ video.title }}</div>
          <div class="watched-at">
            {{ t('history.watchedAt') }} {{ new Date(video.watched_at).toLocaleString('zh-TW') }}
          </div>
        </div>
      </div>
      
      <div v-if="hasNext" class="load-more" @click="loadMore">
        {{ t('history.loadMore') }}
      </div>
    </div>
    
    <div v-if="!loading && results.length === 0" class="empty">
      {{ t('history.empty') }}
    </div>
    
    <div v-if="videoError" class="error-toast">
      {{ videoError }}
    </div>
  </div>
</template>

<style scoped>
.history-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 1rem;
}

.header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.header h2 {
  margin: 0;
  font-size: 1.25rem;
}

.count {
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.video-list {
  flex: 1;
  overflow-y: auto;
}

.video-item {
  display: flex;
  gap: 1rem;
  padding: 1rem;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
}

.video-item:hover {
  background: var(--color-bg-hover);
}

.thumbnail {
  width: 160px;
  height: 90px;
  object-fit: cover;
  border-radius: 4px;
  flex-shrink: 0;
}

.info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.title {
  font-size: 1rem;
  font-weight: 500;
  margin-bottom: 0.5rem;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.watched-at {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.load-more {
  padding: 1rem;
  text-align: center;
  color: var(--color-accent-primary);
  cursor: pointer;
}

.loading, .empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--color-text-muted);
}

.error-toast {
  position: fixed;
  bottom: 2rem;
  left: 50%;
  transform: translateX(-50%);
  background: var(--color-bg-surface);
  border: 1px solid var(--color-accent-primary);
  color: var(--color-text-primary);
  padding: 0.75rem 1.5rem;
  border-radius: 8px;
  box-shadow: var(--shadow-md);
}
</style>
