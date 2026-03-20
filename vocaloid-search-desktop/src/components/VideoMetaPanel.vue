<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { Video } from '../api/tauri-commands'
import {
  getVideoMetaPanelLayout,
  observeDescriptionToggleResize,
  shouldShowDescriptionToggle,
  type VideoMetaPanelDisplayMode,
} from '../features/playlistViews/videoMetaPanelLayout'
import { sanitizeDescriptionHtml } from '../features/playlistViews/videoMetaPanelSanitization'
import UploaderAvatar from './UploaderAvatar.vue'

const props = withDefaults(defineProps<{
  video: Video
  uploaderName?: string
  uploaderIconUrl?: string | null
  uploadDateTime?: string
  collapseLabel?: string
  expandLabel?: string
  showUploaderPlaceholder?: boolean
  displayMode?: VideoMetaPanelDisplayMode
}>(), {
  uploaderName: '',
  uploaderIconUrl: null,
  uploadDateTime: '',
  collapseLabel: '收起',
  expandLabel: '展開',
  showUploaderPlaceholder: false,
  displayMode: 'full',
})

const descriptionExpanded = ref(false)
const showDescriptionToggle = ref(false)
const descriptionContentRef = ref<HTMLElement | null>(null)
const copied = ref(false)
let stopObservingDescriptionResize: (() => void) | null = null

const visibleTags = computed(() => props.video.tags?.slice(0, 12) ?? [])
const remainingTagCount = computed(() => Math.max((props.video.tags?.length ?? 0) - visibleTags.value.length, 0))
const hasUploader = computed(() => Boolean(props.video.uploader_id || props.video.uploader_name || props.uploaderName))
const layout = computed(() => getVideoMetaPanelLayout(props.displayMode))
const sanitizedDescription = computed(() => sanitizeDescriptionHtml(props.video.description ?? ''))

function stopDescriptionResizeObserver() {
  stopObservingDescriptionResize?.()
  stopObservingDescriptionResize = null
}

async function updateDescriptionToggle() {
  if (!layout.value.showDetails) {
    showDescriptionToggle.value = false
    return
  }

  if (descriptionExpanded.value && showDescriptionToggle.value) {
    return
  }

  await nextTick()

  const descriptionContent = descriptionContentRef.value
  if (!descriptionContent) {
    showDescriptionToggle.value = false
    return
  }

  showDescriptionToggle.value = shouldShowDescriptionToggle({
    scrollHeight: descriptionContent.scrollHeight,
    clientHeight: descriptionContent.clientHeight,
  })
}

async function syncDescriptionToggleObservation() {
  stopDescriptionResizeObserver()

  if (!layout.value.showDetails) {
    return
  }

  await nextTick()

  const descriptionContent = descriptionContentRef.value
  if (!descriptionContent) {
    return
  }

  stopObservingDescriptionResize = observeDescriptionToggleResize(descriptionContent, () => {
    void updateDescriptionToggle()
  })
}

watch(() => [props.video.id, sanitizedDescription.value, props.displayMode], async () => {
  descriptionExpanded.value = false
  await updateDescriptionToggle()
  await syncDescriptionToggleObservation()
})

onMounted(async () => {
  await updateDescriptionToggle()
  await syncDescriptionToggleObservation()
})

onBeforeUnmount(() => {
  stopDescriptionResizeObserver()
})

async function copyToClipboard() {
  if (!props.video.watch_url || copied.value) return
  
  try {
    await navigator.clipboard.writeText(props.video.watch_url)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 1500)
  } catch (err) {
    console.error('Failed to copy URL:', err)
  }
}

</script>

<template>
  <div class="video-meta-panel">
    <div v-if="layout.showHeader" class="player-header">
      <div class="header-row">
        <h1 class="video-title">{{ video.title }}</h1>
        <span class="upload-datetime">{{ uploadDateTime }}</span>
      </div>
      <div class="meta-row">
        <div v-if="hasUploader" class="uploader-info">
          <UploaderAvatar
            :src="uploaderIconUrl"
            :alt="uploaderName || video.uploader_name || video.uploader_id || 'Uploader avatar'"
            class="avatar"
          />
          <span class="user-name">{{ uploaderName || video.uploader_name || video.uploader_id }}</span>
        </div>
        <div v-else-if="showUploaderPlaceholder" class="uploader-info-placeholder"></div>
        <div class="stats">
          <span class="stat views">▶ {{ video.view_count?.toLocaleString() ?? 0 }}</span>
          <span class="stat likes">❤️ {{ video.like_count?.toLocaleString() ?? 0 }}</span>
          <span class="stat mylists">📝 {{ video.mylist_count?.toLocaleString() ?? 0 }}</span>
          <span class="stat comments">💬 {{ video.comment_count?.toLocaleString() ?? 0 }}</span>
        </div>
      </div>
    </div>

    <div v-if="layout.showDetails" class="info-below-player">
      <div v-if="video.watch_url" class="url-section">
        <span class="url-text">{{ video.watch_url }}</span>
        <button class="copy-btn" @click="copyToClipboard">
          {{ copied ? '已複製 ✓' : '📋' }}
        </button>
      </div>

      <div v-if="visibleTags.length" class="tags-section">
        <span v-for="tag in visibleTags" :key="tag" class="tag">{{ tag }}</span>
        <span v-if="remainingTagCount > 0" class="tag more">+{{ remainingTagCount }}</span>
      </div>
      <div v-if="video.description" class="description-section">
        <div
          ref="descriptionContentRef"
          class="description-content"
          :class="{ collapsed: !descriptionExpanded }"
          v-html="sanitizedDescription"
        ></div>
        <button
          v-if="showDescriptionToggle"
          class="expand-btn"
          @click="descriptionExpanded = !descriptionExpanded"
        >
          {{ descriptionExpanded ? collapseLabel : expandLabel }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
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

.stat.views { color: var(--color-stat-views, #93C5FD); }
.stat.likes { color: var(--color-stat-likes, #f472b6); }
.stat.mylists { color: var(--color-stat-mylists, #34d399); }
.stat.comments { color: var(--color-stat-comments, #fbbf24); }

.info-below-player {
  flex-shrink: 0;
  padding: var(--space-sm) var(--space-md);
}

.tags-section {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-xs);
  margin-bottom: var(--space-sm);
}

.tag {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--color-bg-hover);
  color: var(--color-text-secondary);
  font-size: var(--font-size-xs);
}

.tag.more {
  opacity: 0.8;
}

.url-section {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
  padding: var(--space-xs) var(--space-sm);
  background: var(--color-bg-hover);
  border-radius: 6px;
}

.url-text {
  flex: 1;
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.copy-btn {
  flex-shrink: 0;
  padding: 4px 8px;
  font-size: var(--font-size-xs);
  font-weight: 500;
  color: var(--color-text-secondary);
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.copy-btn:hover {
  background: var(--color-accent-primary);
  color: var(--color-bg-primary);
  border-color: var(--color-accent-primary);
}

.description-section {
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  line-height: 1.6;
}

.description-content.collapsed {
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.expand-btn {
  display: block;
  width: 100%;
  margin-top: var(--space-sm);
  padding: var(--space-xs);
  font-size: var(--font-size-sm);
  font-weight: 600;
  text-align: center;
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
