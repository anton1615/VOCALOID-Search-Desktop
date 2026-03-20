<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { Video } from '../api/tauri-commands'
import {
  getCompactTitleContentHeight,
  getCompactTitleLineState,
  getVideoMetaPanelLayout,
  getVideoMetaPresentationContract,
  observeCompactTitleResize,
  observeDescriptionToggleResize,
  shouldShowDescriptionToggle,
  type CompactHeaderLineState,
  type VideoMetaPanelDisplayMode,
  type VideoMetaPanelPresentationMode,
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
  presentationMode?: VideoMetaPanelPresentationMode
}>(), {
  uploaderName: '',
  uploaderIconUrl: null,
  uploadDateTime: '',
  collapseLabel: '收起',
  expandLabel: '展開',
  showUploaderPlaceholder: false,
  displayMode: 'full',
  presentationMode: 'full',
})

const descriptionExpanded = ref(false)
const showDescriptionToggle = ref(false)
const descriptionContentRef = ref<HTMLElement | null>(null)
const titleRef = ref<HTMLElement | null>(null)
const compactHeaderLineState = ref<CompactHeaderLineState>('two-line')
const copied = ref(false)
let stopObservingDescriptionResize: (() => void) | null = null
let stopObservingCompactTitleResize: (() => void) | null = null

const visibleTags = computed(() => props.video.tags?.slice(0, 12) ?? [])
const remainingTagCount = computed(() => Math.max((props.video.tags?.length ?? 0) - visibleTags.value.length, 0))
const hasUploader = computed(() => Boolean(props.video.uploader_id || props.video.uploader_name || props.uploaderName))
const layout = computed(() => getVideoMetaPanelLayout(props.displayMode))
const presentationContract = computed(() => getVideoMetaPresentationContract(props.presentationMode, compactHeaderLineState.value))
const sanitizedDescription = computed(() => sanitizeDescriptionHtml(props.video.description ?? ''))

function stopDescriptionResizeObserver() {
  stopObservingDescriptionResize?.()
  stopObservingDescriptionResize = null
}

function stopCompactTitleResizeObserver() {
  stopObservingCompactTitleResize?.()
  stopObservingCompactTitleResize = null
}

async function updateCompactHeaderLineState() {
  if (props.presentationMode !== 'compact' || !layout.value.showHeader) {
    compactHeaderLineState.value = 'two-line'
    return
  }

  await nextTick()

  const titleElement = titleRef.value
  if (!titleElement) {
    compactHeaderLineState.value = 'two-line'
    return
  }

  const computedStyle = window.getComputedStyle(titleElement)
  const rawLineHeight = Number.parseFloat(computedStyle.lineHeight)
  const fallbackFontSize = Number.parseFloat(computedStyle.fontSize)
  const lineHeight = Number.isFinite(rawLineHeight)
    ? rawLineHeight
    : fallbackFontSize * 1.4
  const minHeight = Number.parseFloat(computedStyle.minHeight)
  const contentHeight = getCompactTitleContentHeight({
    elementHeight: titleElement.getBoundingClientRect().height,
    minHeight,
  })

  compactHeaderLineState.value = getCompactTitleLineState({
    titleHeight: contentHeight,
    lineHeight,
  })
}

async function syncCompactTitleObservation() {
  stopCompactTitleResizeObserver()

  if (props.presentationMode !== 'compact' || !layout.value.showHeader) {
    return
  }

  await nextTick()

  const titleElement = titleRef.value
  if (!titleElement) {
    return
  }

  stopObservingCompactTitleResize = observeCompactTitleResize(titleElement, () => {
    void updateCompactHeaderLineState()
  })
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
  await updateCompactHeaderLineState()
  await syncCompactTitleObservation()
})

watch(() => [props.video.title, props.presentationMode, props.displayMode], async () => {
  await updateCompactHeaderLineState()
  await syncCompactTitleObservation()
})

onMounted(async () => {
  await updateDescriptionToggle()
  await syncDescriptionToggleObservation()
  await updateCompactHeaderLineState()
  await syncCompactTitleObservation()
})

onBeforeUnmount(() => {
  stopDescriptionResizeObserver()
  stopCompactTitleResizeObserver()
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
  <div
    class="video-meta-panel"
    :data-presentation-mode="props.presentationMode"
    :data-title-clamp="presentationContract.titleClampLines"
    :data-uploader-clamp="presentationContract.uploaderClampLines"
    :data-compact-header-line-state="props.presentationMode === 'compact' ? compactHeaderLineState : undefined"
  >
    <div v-if="layout.showHeader" class="player-header">
      <div class="header-row">
        <h1
          ref="titleRef"
          class="video-title"
          :class="[
            `title-clamp-${presentationContract.titleClampLines}`,
            `title-frame-${presentationContract.titleClampLines}`,
            `compact-header-line-state-${presentationContract.compactHeaderLineState ?? 'two-line'}`,
          ]"
        >{{ video.title }}</h1>
        <span class="upload-datetime">{{ uploadDateTime }}</span>
      </div>
      <div :class="['meta-row', { 'meta-row-emphasized': presentationContract.emphasizedMeta, 'meta-row-fixed-height': presentationContract.fixedMetaRowHeight }]">
        <div v-if="hasUploader" class="uploader-info">
          <UploaderAvatar
            :src="uploaderIconUrl"
            :alt="uploaderName || video.uploader_name || video.uploader_id || 'Uploader avatar'"
            :class="['avatar', `avatar-${presentationContract.avatarSize}`]"
          />
          <span class="user-name" :class="[`uploader-clamp-${presentationContract.uploaderClampLines}`]">{{ uploaderName || video.uploader_name || video.uploader_id }}</span>
        </div>
        <div v-else-if="showUploaderPlaceholder" class="uploader-info-placeholder"></div>
        <div :class="['stats', `stats-gap-${presentationContract.statsGap}`, { 'stats-inline-spacing': presentationContract.statsInlineSpacing }]">
          <span class="stat views">▶ {{ video.view_count?.toLocaleString() ?? 0 }}</span>
          <span class="stat likes">❤️ {{ video.like_count?.toLocaleString() ?? 0 }}</span>
          <span class="stat mylists">📝 {{ video.mylist_count?.toLocaleString() ?? 0 }}</span>
          <span class="stat comments">💬 {{ video.comment_count?.toLocaleString() ?? 0 }}</span>
        </div>
      </div>
    </div>

    <div v-if="layout.showDetails" class="info-below-player">
      <div v-if="video.watch_url" :class="['url-section', `url-treatment-${presentationContract.urlTreatment}`]">
        <span class="url-text">{{ video.watch_url }}</span>
        <button class="copy-btn" @click="copyToClipboard">
          {{ copied ? '已複製 ✓' : '📋' }}
        </button>
      </div>

      <div v-if="visibleTags.length" class="tags-section">
        <span v-for="tag in visibleTags" :key="tag" class="tag">{{ tag }}</span>
        <span v-if="remainingTagCount > 0" class="tag more">+{{ remainingTagCount }}</span>
      </div>
      <div v-if="presentationContract.showTagDescriptionDivider && visibleTags.length && video.description" class="tag-description-divider"></div>
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

.video-meta-panel[data-presentation-mode='compact'] {
  --compact-player-header-height: 82px;
}

.video-meta-panel[data-presentation-mode='compact'] .player-header {
  height: var(--compact-player-header-height);
  padding: 0 var(--space-md);
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  overflow: hidden;
}

.video-meta-panel[data-presentation-mode='compact'] .meta-row {
  min-height: 24px;
}

.header-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-md);
  margin-bottom: var(--space-sm);
}

.video-meta-panel[data-presentation-mode='compact'] .header-row {
  margin-bottom: 0;
}

.video-meta-panel[data-presentation-mode='compact'] .player-header {
  justify-content: space-evenly;
}

.video-meta-panel[data-presentation-mode='compact'][data-compact-header-line-state='single-line'] .title-frame-2 {
  min-height: calc(1.4em * 1);
}

.video-meta-panel[data-presentation-mode='compact'][data-compact-header-line-state='two-line'] .title-frame-2 {
  min-height: calc(1.4em * 2);
}

.video-meta-panel[data-presentation-mode='compact'][data-compact-header-line-state='two-line'] .header-row {
  margin-bottom: 0;
}

.video-meta-panel[data-presentation-mode='compact'].video-meta-panel :deep(.meta-row-fixed-height),
.video-meta-panel[data-presentation-mode='compact'] .meta-row-fixed-height {
  min-height: 24px;
}

.video-meta-panel[data-presentation-mode='compact'].video-meta-panel :deep(.uploader-info),
.video-meta-panel[data-presentation-mode='compact'] .uploader-info {
  gap: var(--space-xs);
}

.video-meta-panel[data-presentation-mode='compact'].video-meta-panel :deep(.avatar-sm),
.video-meta-panel[data-presentation-mode='compact'] .avatar-sm {
  width: 20px;
  height: 20px;
}

.video-meta-panel[data-presentation-mode='compact'].video-meta-panel :deep(.user-name),
.video-meta-panel[data-presentation-mode='compact'] .user-name {
  line-height: 1.2;
}

.video-meta-panel[data-presentation-mode='compact'].video-meta-panel :deep(.stat),
.video-meta-panel[data-presentation-mode='compact'] .stat {
  line-height: 1.2;
}

.video-meta-panel[data-presentation-mode='compact'].video-meta-panel :deep(.upload-datetime),
.video-meta-panel[data-presentation-mode='compact'] .upload-datetime {
  line-height: 1.2;
}

.upload-datetime {
  font-size: 15px;
  color: var(--color-text-secondary-light);
  white-space: nowrap;
  flex-shrink: 0;
}

.video-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex: 1;
}

.title-clamp-1 {
  -webkit-line-clamp: 1;
}

.title-clamp-2 {
  -webkit-line-clamp: 2;
}

.title-frame-1 {
  min-height: calc(1.4em * 1);
}

.title-frame-2 {
  min-height: calc(1.4em * 2);
}

.meta-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-md);
}

.meta-row-emphasized :deep(.upload-datetime),
.meta-row-emphasized :deep(.stat) {
  font-weight: 700;
}

.meta-row-fixed-height {
  min-height: 32px;
}

.uploader-info {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  min-width: 0;
}

.avatar {
  border-radius: 50%;
  background: var(--color-bg-hover);
  object-fit: cover;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.avatar-sm {
  width: 24px;
  height: 24px;
}

.avatar-md {
  width: 32px;
  height: 32px;
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

.uploader-clamp-1 {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stats {
  display: flex;
}

.stats-gap-normal {
  gap: var(--space-lg);
}

.stats-gap-spacious {
  gap: calc(var(--space-lg) + var(--space-xs));
}

.stats-inline-spacing {
  letter-spacing: 0.02em;
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

.tag-description-divider {
  height: 1px;
  margin-bottom: var(--space-sm);
  background: var(--color-border-subtle);
}

.url-section {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
  padding: var(--space-xs) var(--space-sm);
  border-radius: 6px;
}

.url-treatment-surface {
  background: var(--color-bg-hover);
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
