<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import WatchLaterButton from './WatchLaterButton.vue'

const props = withDefaults(defineProps<{
  /** Layout direction */
  layout: 'horizontal' | 'vertical'
  /** Current video ID */
  videoId: string | null
  /** Current video title */
  videoTitle?: string
  /** Current video thumbnail URL */
  thumbnailUrl?: string
  /** Whether player is ready */
  playerReady: boolean
  /** Whether currently playing */
  isPlaying: boolean
  /** Current video index */
  currentIndex: number
  /** Total results count */
  resultsCount: number
  /** Whether there is a next video */
  hasNext: boolean
  /** Whether to show auto-skip settings */
  showAutoSkip?: boolean
  /** Auto-play setting */
  autoPlay?: boolean
  /** Auto-skip setting */
  autoSkip?: boolean
  /** Whether settings panel is open */
  settingsPanelOpen?: boolean
}>(), {
  showAutoSkip: true,
  autoPlay: true,
  autoSkip: false,
  settingsPanelOpen: false,
})

defineEmits<{
  (e: 'togglePlayPause'): void
  (e: 'playNext'): void
  (e: 'playPrevious'): void
  (e: 'toggleSettings'): void
  (e: 'update:autoPlay', value: boolean): void
  (e: 'update:autoSkip', value: boolean): void
}>()

const { t } = useI18n()

const isVertical = computed(() => props.layout === 'vertical')

const canPlayPrevious = computed(() => props.currentIndex > 0)
const canPlayNext = computed(() => props.currentIndex >= 0 && (props.currentIndex < props.resultsCount - 1 || props.hasNext))
</script>

<template>
  <div class="player-controls" :class="{ vertical: isVertical }">
    <!-- Vertical layout (compact mode) -->
    <template v-if="isVertical">
      <div class="controls-vertical">
        <WatchLaterButton
          :video-id="videoId"
          :video-title="videoTitle"
          :thumbnail-url="thumbnailUrl"
          :disabled="!videoId"
        />
        <button
          class="icon-btn"
          :disabled="!canPlayPrevious"
          :title="t('player.previous')"
          @click="$emit('playPrevious')"
        >
          ⏮
        </button>
        <button
          class="icon-btn play-pause-btn"
          :disabled="!videoId"
          @click="$emit('togglePlayPause')"
        >
          {{ isPlaying ? '⏸' : '▶' }}
        </button>
        <button
          class="icon-btn"
          :disabled="!canPlayNext"
          :title="t('player.next')"
          @click="$emit('playNext')"
        >
          ⏭
        </button>
      </div>
    </template>

    <!-- Horizontal layout (full mode) -->
    <template v-else>
      <WatchLaterButton
        :video-id="videoId"
        :video-title="videoTitle"
        :thumbnail-url="thumbnailUrl"
        :disabled="!videoId"
      />
      
      <div class="media-actions">
        <button
          class="icon-btn"
          :disabled="!canPlayPrevious"
          @click="$emit('playPrevious')"
        >
          ⏮
        </button>
        <button
          class="icon-btn play-pause-btn"
          @click="$emit('togglePlayPause')"
        >
          {{ isPlaying ? '⏸' : '▶' }}
        </button>
        <button
          class="icon-btn"
          :disabled="!canPlayNext"
          @click="$emit('playNext')"
        >
          ⏭
        </button>
      </div>

      <div v-if="showAutoSkip" class="playback-settings-slot">
        <button
          class="icon-btn settings-btn"
          type="button"
          :title="t('player.playbackSettings')"
          :aria-label="t('player.playbackSettings')"
          :aria-expanded="settingsPanelOpen"
          @click="$emit('toggleSettings')"
        >
          ⚙
        </button>
      </div>

      <!-- Settings panel (only in horizontal mode) -->
      <div v-if="showAutoSkip && settingsPanelOpen" class="playback-settings-panel">
        <label class="toggle-label">
          <input
            type="checkbox"
            :checked="autoPlay"
            @change="$emit('update:autoPlay', ($event.target as HTMLInputElement).checked)"
          >
          <span>{{ t('player.autoPlay') }}</span>
        </label>
        <label class="toggle-label">
          <input
            type="checkbox"
            :checked="autoSkip"
            @change="$emit('update:autoSkip', ($event.target as HTMLInputElement).checked)"
          >
          <span>{{ t('player.autoSkip') }}</span>
        </label>
      </div>
    </template>
  </div>
</template>

<style scoped>
/* Base styles - only apply to vertical mode */
.player-controls {
  flex-shrink: 0;
}

.player-controls.vertical {
  display: flex;
  flex-direction: column;
  background: var(--color-bg-surface);
  border-right: 1px solid var(--color-border-subtle);
  height: 100%;
}

.controls-vertical {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-md) var(--space-xs);
  gap: var(--space-sm);
  flex: 1;
}

/* Horizontal mode - parent (.controls-section) handles layout */
.player-controls:not(.vertical) {
  display: contents;
}

.media-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
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

.player-controls.vertical .icon-btn {
  width: 36px;
  height: 36px;
  font-size: 16px;
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

.player-controls.vertical .play-pause-btn {
  width: 42px;
  height: 42px;
  font-size: 20px;
}

.play-pause-btn:hover {
  background: var(--color-accent-primary);
  color: var(--color-bg-primary);
}

.playback-settings-slot {
  display: flex;
  align-items: center;
}

.settings-btn {
  font-size: 17px;
}

.playback-settings-panel {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-lg);
  padding: var(--space-sm) var(--space-md);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  border-top: 1px solid var(--color-border-subtle);
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
</style>
