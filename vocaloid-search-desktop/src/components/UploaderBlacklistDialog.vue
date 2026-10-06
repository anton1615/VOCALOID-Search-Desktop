<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { getUploaderAvatarUrl, type BlockedUploader, type UploaderCandidate } from '../api/tauri-commands'
import { formatDateTime } from '../utils/dateTime'
import { useUploaderBlacklistStore } from '../stores/uploaderBlacklist'
import UploaderAvatar from './UploaderAvatar.vue'

const { t } = useI18n()
const store = useUploaderBlacklistStore()

const query = ref('')
const highlightIndex = ref(-1)
const searchInputRef = ref<HTMLInputElement | null>(null)

const pendingRemoval = ref<BlockedUploader | null>(null)
const removing = ref(false)

// Already-blocked uploaders are hidden from the candidate list.
const visibleCandidates = computed(() =>
  store.candidates.filter((candidate) => !store.isBlocked(candidate.uploader_id)),
)

function candidateAvatar(candidate: UploaderCandidate): string | null {
  return candidate.icon_url || getUploaderAvatarUrl(candidate.uploader_id)
}

async function selectCandidate(candidate: UploaderCandidate) {
  try {
    await store.add(candidate.uploader_id, candidate.nickname)
    highlightIndex.value = -1
  } catch (error) {
    console.error('[UploaderBlacklistDialog] Failed to add uploader:', error)
  }
}

function onSearchKeydown(event: KeyboardEvent) {
  const list = visibleCandidates.value

  if (event.key === 'ArrowDown') {
    event.preventDefault()
    if (list.length === 0) return
    highlightIndex.value = highlightIndex.value >= list.length - 1 ? 0 : highlightIndex.value + 1
    return
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault()
    if (list.length === 0) return
    highlightIndex.value = highlightIndex.value <= 0 ? list.length - 1 : highlightIndex.value - 1
    return
  }

  if (event.key === 'Enter') {
    event.preventDefault()
    const target = list[highlightIndex.value] ?? list[0]
    if (target) void selectCandidate(target)
  }
}

function promptRemove(item: BlockedUploader) {
  pendingRemoval.value = item
}

function cancelRemove() {
  if (removing.value) return
  pendingRemoval.value = null
}

async function confirmRemove() {
  if (!pendingRemoval.value || removing.value) return

  removing.value = true
  try {
    await store.remove(pendingRemoval.value.uploader_id)
    pendingRemoval.value = null
  } catch (error) {
    console.error('[UploaderBlacklistDialog] Failed to remove uploader:', error)
  } finally {
    removing.value = false
  }
}

function onWindowKeydown(event: KeyboardEvent) {
  if (event.key !== 'Escape') return
  if (pendingRemoval.value) {
    cancelRemove()
  } else {
    store.closeDialog()
  }
}

watch(query, (value) => {
  highlightIndex.value = -1
  store.searchCandidates(value)
})

watch(
  () => store.isOpen,
  (open) => {
    if (!open) return
    query.value = ''
    highlightIndex.value = -1
    pendingRemoval.value = null
    store.searchCandidates('')
    void nextTick(() => searchInputRef.value?.focus())
  },
)

onMounted(() => window.addEventListener('keydown', onWindowKeydown))
onUnmounted(() => window.removeEventListener('keydown', onWindowKeydown))
</script>

<template>
  <div v-if="store.isOpen" class="modal-backdrop" @click.self="store.closeDialog()">
    <div class="modal blacklist-modal" role="dialog" aria-modal="true">
      <div class="modal-header">
        <h3>{{ t('blacklist.title') }}</h3>
        <button class="close-btn" :title="t('blacklist.cancel')" @click="store.closeDialog()">✕</button>
      </div>

      <div class="search-section">
        <input
          ref="searchInputRef"
          v-model="query"
          class="search-input"
          type="text"
          :placeholder="t('blacklist.searchPlaceholder')"
          :title="t('blacklist.searchHint')"
          @keydown="onSearchKeydown"
        />
        <p class="search-hint">{{ t('blacklist.searchHint') }}</p>

        <div class="candidates">
          <div v-if="store.searching" class="state-row">{{ t('blacklist.loading') }}</div>
          <template v-else-if="visibleCandidates.length > 0">
            <button
              v-for="(candidate, idx) in visibleCandidates"
              :key="candidate.uploader_id"
              type="button"
              class="candidate-row"
              :class="{ highlighted: idx === highlightIndex }"
              @mouseenter="highlightIndex = idx"
              @click="selectCandidate(candidate)"
            >
              <span class="avatar-slot">
                <UploaderAvatar :src="candidateAvatar(candidate)" :alt="candidate.nickname" />
              </span>
              <span class="candidate-info">
                <span class="candidate-name">{{ candidate.nickname }}</span>
                <span class="row-sub">id {{ candidate.uploader_id }}</span>
              </span>
              <span v-if="candidate.video_count != null" class="candidate-meta">
                {{ t('blacklist.videoCount', { count: candidate.video_count }) }}
              </span>
            </button>
          </template>
          <div v-else-if="query.trim().length > 0" class="state-row">{{ t('blacklist.noCandidates') }}</div>
        </div>
      </div>

      <div class="list-section">
        <div v-if="store.loading" class="state-row">{{ t('blacklist.loading') }}</div>
        <div v-else-if="store.items.length === 0" class="state-row">{{ t('blacklist.empty') }}</div>
        <ul v-else class="blocked-list">
          <li v-for="item in store.items" :key="item.uploader_id" class="blocked-row">
            <span class="avatar-slot">
              <UploaderAvatar :src="getUploaderAvatarUrl(item.uploader_id)" :alt="item.uploader_id" />
            </span>
            <span class="blocked-info">
              <span class="blocked-name">{{ item.display_name || item.uploader_id }}</span>
              <span class="row-sub">id {{ item.uploader_id }} · {{ formatDateTime(item.added_at) }}</span>
            </span>
            <button class="remove-btn" :title="t('blacklist.remove')" @click="promptRemove(item)">✕</button>
          </li>
        </ul>
      </div>
    </div>

    <div v-if="pendingRemoval" class="modal-backdrop confirm-backdrop" @click.self="cancelRemove">
      <div class="modal confirm-modal">
        <h3>{{ t('blacklist.removeConfirmTitle') }}</h3>
        <p>
          {{ t('blacklist.removeConfirmBody') }}
          <strong>{{ pendingRemoval.display_name || pendingRemoval.uploader_id }}</strong>
        </p>
        <div class="modal-actions">
          <button class="btn-secondary modal-btn" :disabled="removing" @click="cancelRemove">{{ t('blacklist.cancel') }}</button>
          <button class="modal-btn modal-btn-danger" :disabled="removing" @click="confirmRemove">{{ t('blacklist.confirm') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
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
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.28);
  color: var(--color-text-primary);
}

.blacklist-modal {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  width: min(560px, calc(100vw - 2rem));
  max-height: min(80vh, 640px);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-header h3 {
  margin: 0;
  font-size: var(--font-size-lg);
}

.close-btn,
.remove-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border-subtle);
  background: transparent;
  color: var(--color-text-secondary);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.16s ease;
}

.close-btn {
  width: 32px;
  height: 32px;
}

.remove-btn {
  width: 30px;
  height: 30px;
  flex-shrink: 0;
}

.close-btn:hover,
.remove-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border-focus);
}

.search-section {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.search-input {
  width: 100%;
  padding: 0.6rem 0.75rem;
  border-radius: 8px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-border-focus);
}

.search-hint {
  margin: 0;
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.candidates {
  display: flex;
  flex-direction: column;
  max-height: 220px;
  overflow-y: auto;
}

.candidate-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  width: 100%;
  padding: 0.5rem 0.6rem;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--color-text-primary);
  text-align: left;
  cursor: pointer;
}

.candidate-row.highlighted,
.candidate-row:hover {
  background: var(--color-bg-hover);
}

.candidate-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.candidate-name,
.blocked-name {
  font-size: var(--font-size-sm);
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.candidate-meta {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.row-sub {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.list-section {
  border-top: 1px solid var(--color-border-subtle);
  padding-top: 0.75rem;
  overflow-y: auto;
}

.blocked-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.blocked-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.5rem 0.6rem;
  border-radius: 8px;
}

.blocked-row:hover {
  background: var(--color-bg-hover);
}

.blocked-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.state-row {
  padding: 1rem 0.6rem;
  text-align: center;
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.avatar-slot {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border-radius: 50%;
  background: var(--color-bg-hover);
}

.avatar-slot :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-slot :deep(.default-avatar) {
  font-size: 1rem;
}

.confirm-backdrop {
  z-index: 1100;
}

.confirm-modal {
  max-width: 400px;
  width: min(400px, calc(100vw - 2rem));
}

.confirm-modal h3 {
  margin: 0 0 0.75rem;
}

.confirm-modal p {
  color: var(--color-text-secondary);
  margin: 0 0 1.5rem;
  line-height: 1.5;
}

.confirm-modal strong {
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
  cursor: pointer;
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
</style>
