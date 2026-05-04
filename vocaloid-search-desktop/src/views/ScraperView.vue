<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, inject, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import {
  api,
  type ScraperConfig,
  type ScraperProgress,
  type DatabaseStats,
  type StorageInfo,
  type SyncPreflightEstimate,
  type WatchDataImportPreviewResponse,
  type WatchDataImportCompleted,
} from '../api/tauri-commands'
import { formatDateTime } from '../utils/dateTime'
import { formatStorageSize, formatVideoCount } from '../features/playlistViews/scraperFormatting'

const { t } = useI18n()

const config = ref<ScraperConfig>({
  query: 'VOCALOID',
  max_age_days: 365,
  targets: 'tags',
  category_filter: 'MUSIC',
})

const stats = ref<DatabaseStats>({
  total_videos: 0,
  last_update: null,
})

const progress = ref<ScraperProgress>({
  is_running: false,
  videos_fetched: 0,
  total_expected: null,
  status: 'idle',
})

const showConfirm = ref(false)
const progressPollInterval = ref<number | null>(null)
const freshnessStatus = inject<Ref<{ message: string; isFresh: boolean; localLastUpdate: string | null; apiLastUpdate: string | null }>>('freshnessStatus')
const storageInfo = ref<StorageInfo | null>(null)
const preflightEstimate = ref<SyncPreflightEstimate | null>(null)
const preflightLoading = ref(false)
const showImportConfirm = ref(false)
const selectedImportPath = ref('')
const importPreview = ref<WatchDataImportPreviewResponse | null>(null)
const importSuccess = ref<WatchDataImportCompleted | null>(null)
const importError = ref('')
const importPreviewLoading = ref(false)
const importExecuteLoading = ref(false)

const isStorageInsufficient = computed(() => {
  const estimated = preflightEstimate.value?.estimated_database_size_kb
  const free = preflightEstimate.value?.free_space_kb
  if (estimated === null || estimated === undefined || free === null || free === undefined) {
    return false
  }
  return estimated > free
})

const selectedImportFileName = computed(() => {
  if (!selectedImportPath.value) return ''
  return selectedImportPath.value.split(/[/\\]/).pop() ?? selectedImportPath.value
})

async function loadStorageInfo() {
  try {
    storageInfo.value = await api.getStorageInfo()
  } catch (e) {
    console.error('Failed to get storage info:', e)
  }
}

async function loadConfig() {
  try {
    config.value = await api.getScraperConfig()
  } catch (e) {
    console.error('Failed to load config:', e)
  }
}

async function loadStats() {
  try {
    stats.value = await api.getDatabaseStats()
  } catch (e) {
    console.error('Failed to load stats:', e)
  }
}

async function refreshFreshnessStatus() {
  if (!freshnessStatus) return

  try {
    const freshness = await api.checkDatabaseFreshness()
    freshnessStatus.value = {
      message: freshness.message,
      isFresh: freshness.is_fresh,
      localLastUpdate: freshness.local_last_update ?? null,
      apiLastUpdate: freshness.api_last_update ?? null,
    }
  } catch (e) {
    console.error('Failed to refresh freshness status:', e)
  }
}

async function saveConfig() {
  try {
    await api.saveScraperConfig(config.value)
  } catch (e) {
    console.error('Failed to save config:', e)
  }
}

async function startSync() {
  preflightLoading.value = true
  preflightEstimate.value = null
  try {
    preflightEstimate.value = await api.getSyncPreflightEstimate()
  } catch (e) {
    console.error('Failed to get sync preflight estimate:', e)
  } finally {
    preflightLoading.value = false
    showConfirm.value = true
  }
}

async function runScraper() {
  showConfirm.value = false
  try {
    await api.runScraper()
    startPolling()
  } catch (e) {
    console.error('Failed to start scraper:', e)
  }
}

async function cancelSync() {
  try {
    await api.cancelScraper()
    progress.value.is_running = false
    progress.value.status = 'cancelled'
    stopPolling()
    await loadStats()
  } catch (e) {
    console.error('Failed to cancel scraper:', e)
  }
}

function closeImportModal() {
  selectedImportPath.value = ''
  showImportConfirm.value = false
}

async function selectWatchDataImportFile() {
  importError.value = ''
  importSuccess.value = null
  importPreview.value = null
  importPreviewLoading.value = true

  try {
    const selectedPath = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'SQLite', extensions: ['db', 'sqlite', 'sqlite3'] }],
    })

    if (!selectedPath || Array.isArray(selectedPath)) {
      selectedImportPath.value = ''
      return
    }

    selectedImportPath.value = selectedPath
    importPreview.value = await api.previewWatchDataImport(selectedPath)
    showImportConfirm.value = true
  } catch (error) {
    importError.value = error instanceof Error ? error.message : t('scraper.importPreviewError')
    console.error('Failed to preview watch-data import:', error)
  } finally {
    importPreviewLoading.value = false
  }
}

async function confirmWatchDataImport() {
  if (!selectedImportPath.value || !importPreview.value) return

  importExecuteLoading.value = true
  importError.value = ''

  try {
    importSuccess.value = await api.executeWatchDataImport({
      path: selectedImportPath.value,
      fingerprint: importPreview.value.fingerprint,
      confirmation_token: importPreview.value.confirmation_token,
      confirmed_summary: {
        file_name: importPreview.value.file_name,
        history: importPreview.value.history,
        watch_later: importPreview.value.watch_later,
      },
    })
    showImportConfirm.value = false
    await loadStats()
  } catch (error) {
    importError.value = error instanceof Error ? error.message : t('scraper.importExecuteError')
    console.error('Failed to execute watch-data import:', error)
  } finally {
    importExecuteLoading.value = false
  }
}

async function refreshProgress() {
  try {
    const p = await api.getScraperProgress()
    progress.value = p
    if (!p.is_running) {
      stopPolling()
      await loadStats()
      await refreshFreshnessStatus()
    }
  } catch (e) {
    console.error('Failed to get progress:', e)
  }
}

function startPolling() {
  if (progressPollInterval.value) return

  progressPollInterval.value = window.setInterval(() => {
    void refreshProgress()
  }, 1000)
  void refreshProgress()
}

function stopPolling() {
  if (progressPollInterval.value) {
    clearInterval(progressPollInterval.value)
    progressPollInterval.value = null
  }
}

const categoryOptions = computed(() => [
  { value: null, label: t('scraper.categoryNone') },
  { value: 'MUSIC', label: t('scraper.categoryMusic') },
  { value: 'GAME', label: t('scraper.categoryGame') },
  { value: 'ANIME', label: t('scraper.categoryAnime') },
  { value: 'ENTERTAINMENT', label: t('scraper.categoryEntertainment') },
  { value: 'DANCE', label: t('scraper.categoryDance') },
  { value: 'ANIMAL', label: t('scraper.categoryAnimal') },
  { value: 'NATURE', label: t('scraper.categoryNature') },
  { value: 'COOKING', label: t('scraper.categoryCooking') },
  { value: 'TRAVEL', label: t('scraper.categoryTravel') },
  { value: 'VEHICLE', label: t('scraper.categoryVehicle') },
  { value: 'SPORTS', label: t('scraper.categorySports') },
  { value: 'SOCIAL', label: t('scraper.categorySocial') },
  { value: 'TECHNICAL', label: t('scraper.categoryTechnical') },
  { value: 'LECTURE', label: t('scraper.categoryLecture') },
  { value: 'OTHER', label: t('scraper.categoryOther') },
  { value: 'RADIO', label: t('scraper.categoryRadio') },
])

const targetOptions = computed(() => [
  { value: 'tags', label: t('scraper.targetTags') },
  { value: 'tagsExact', label: t('scraper.targetTagsExact') },
  { value: 'title', label: t('scraper.targetTitle') },
  { value: 'description', label: t('scraper.targetDescription') },
  { value: 'tags,title', label: t('scraper.targetTagsTitle') },
])

onMounted(async () => {
  await loadConfig()
  await loadStats()
  await loadStorageInfo()

  try {
    const currentProgress = await api.getScraperProgress()
    progress.value = currentProgress

    if (currentProgress.is_running) {
      startPolling()
    }
  } catch (e) {
    console.error('Failed to get scraper progress:', e)
  }
})

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="scraper-view">
    <h2>{{ t('scraper.title') }}</h2>

    <div v-if="freshnessStatus?.message && !freshnessStatus.isFresh" class="alert alert-warning status-card">
      <div class="status-title">
        {{ stats.total_videos > 0 ? t('scraper.updateAvailableTitle') : t('scraper.emptyDatabaseTitle') }}
      </div>
      <div class="status-message">{{ freshnessStatus.message }}</div>
    </div>

    <div class="stats-card">
      <div class="stat">
        <span class="label">{{ t('scraper.totalVideos') }}</span>
        <span class="value">{{ stats.total_videos.toLocaleString() }}</span>
      </div>
      <div class="stat">
        <span class="label">{{ t('scraper.lastUpdate') }}</span>
        <span class="value">
          {{ stats.last_update ? formatDateTime(stats.last_update) : t('scraper.neverUpdated') }}
        </span>
      </div>
    </div>

    <div v-if="storageInfo" class="path-info">
      <h3>{{ t('scraper.storageTitle') }}</h3>
      <div class="storage-row">
        <span class="label">{{ t('scraper.dataDirectory') }}</span>
        <code>{{ storageInfo.data_directory }}</code>
      </div>
      <p class="storage-description">{{ t('scraper.storageDescription') }}</p>
    </div>

    <div class="path-info import-card">
      <h3>{{ t('scraper.importTitle') }}</h3>
      <p class="storage-description">
        {{ t('scraper.importDescription') }}
      </p>
      <div v-if="selectedImportFileName" class="storage-row">
        <span class="label">{{ t('scraper.importSelectedFile') }}</span>
        <code>{{ selectedImportFileName }}</code>
      </div>
      <div v-if="importSuccess" class="alert alert-success import-status">
        {{
          t('scraper.importSuccess', {
            fileName: importSuccess.file_name,
            historyAdded: importSuccess.history.add,
            watchLaterAdded: importSuccess.watch_later.add,
          })
        }}
      </div>
      <div v-else-if="importError" class="alert alert-danger import-status">
        {{ importError }}
      </div>
      <div class="import-actions">
        <button @click="selectWatchDataImportFile" class="btn-secondary" :disabled="importPreviewLoading || importExecuteLoading">
          {{ importPreviewLoading ? t('scraper.importPreviewLoading') : t('scraper.importChooseFile') }}
        </button>
      </div>
    </div>

    <div class="config-form">
      <h3>{{ t('scraper.syncSettings') }}</h3>

      <div class="form-group">
        <label>{{ t('scraper.searchKeyword') }}</label>
        <input
          v-model="config.query"
          type="text"
          @change="saveConfig"
          :placeholder="t('scraper.searchKeywordPlaceholder')"
        />
      </div>

      <div class="form-row">
        <div class="form-group">
          <label>{{ t('scraper.maxDays') }}</label>
          <input
            v-model.number="config.max_age_days"
            type="number"
            @change="saveConfig"
            :placeholder="t('scraper.maxDaysPlaceholder')"
          />
          <span class="hint">{{ t('scraper.maxDaysHint') }}</span>
        </div>

        <div class="form-group">
          <label>{{ t('scraper.category') }}</label>
          <select v-model="config.category_filter" @change="saveConfig">
            <option v-for="opt in categoryOptions" :key="opt.value ?? 'none'" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <div class="form-group">
          <label>{{ t('scraper.searchTarget') }}</label>
          <select v-model="config.targets" @change="saveConfig">
            <option v-for="opt in targetOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
      </div>
    </div>

    <div class="actions">
      <button
        v-if="!progress.is_running"
        @click="startSync"
        class="btn-primary"
      >
        {{ t('scraper.startSync') }}
      </button>
      <button
        v-else
        @click="cancelSync"
        class="btn-danger"
      >
        {{ t('scraper.cancelSync') }}
      </button>
    </div>

    <div v-if="progress.is_running || progress.status !== 'idle'" class="progress-card">
      <h3>{{ t('scraper.syncProgress') }}</h3>
      <div class="progress-info">
        <span class="status">{{ progress.status }}</span>
        <span class="count">
          {{ progress.videos_fetched.toLocaleString() }}
          <span v-if="progress.total_expected">
            / {{ progress.total_expected.toLocaleString() }}
          </span>
          {{ t('scraper.videosFetched') }}
        </span>
      </div>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{
            width: progress.total_expected
              ? `${(progress.videos_fetched / progress.total_expected) * 100}%`
              : '0%'
          }"
        ></div>
      </div>
    </div>

    <div v-if="showConfirm" class="modal-backdrop" @click.self="showConfirm = false">
      <div class="modal">
        <h3>{{ t('scraper.startSync') }}</h3>
        <p>{{ t('scraper.confirmClearReplace') }}</p>
        <div v-if="preflightLoading" class="modal-info-row">{{ t('scraper.preflightLoading') }}</div>
        <template v-else>
          <div v-if="isStorageInsufficient" class="alert alert-danger insufficient-storage-alert">
            <div class="status-title">{{ t('scraper.insufficientStorageTitle') }}</div>
            <div class="status-message">{{ t('scraper.insufficientStorageMessage') }}</div>
          </div>
          <div class="modal-info-row">
            <span>{{ t('scraper.estimatedVideos') }}</span>
            <strong>{{ formatVideoCount(preflightEstimate?.estimated_video_count ?? null) }}</strong>
          </div>
          <div class="modal-info-row">
            <span>{{ t('scraper.estimatedDatabaseSize') }}</span>
            <strong>{{ formatStorageSize(preflightEstimate?.estimated_database_size_kb ?? null) }}</strong>
          </div>
          <div class="modal-info-row">
            <span>{{ t('scraper.availableDiskSpace') }}</span>
            <strong>{{ formatStorageSize(preflightEstimate?.free_space_kb ?? null) }}</strong>
          </div>
        </template>
        <div class="modal-actions">
          <button @click="showConfirm = false" class="btn-secondary">{{ t('scraper.cancel') }}</button>
          <button v-if="!isStorageInsufficient" @click="runScraper" class="btn-primary">{{ t('scraper.syncConfirmAction') }}</button>
        </div>
      </div>
    </div>

    <div v-if="showImportConfirm" class="modal-backdrop" @click.self="showImportConfirm = false">
      <div class="modal import-modal">
        <h3>{{ t('scraper.importTitle') }}</h3>
        <p>{{ t('scraper.importReviewDescription') }}</p>
        <div v-if="importPreview" class="import-preview-grid">
          <div class="modal-info-row">
            <span>{{ t('scraper.importSelectedFile') }}</span>
            <strong>{{ importPreview.file_name }}</strong>
          </div>
          <div class="modal-info-row">
            <span>{{ t('scraper.importHistoryRows') }}</span>
            <strong>{{ importPreview.history.imported }}</strong>
          </div>
          <div class="modal-info-row">
            <span>{{ t('scraper.importWatchLaterRows') }}</span>
            <strong>{{ importPreview.watch_later.imported }}</strong>
          </div>
          <div class="modal-info-row">
            <span>{{ t('scraper.importHistoryCounts', { preserve: importPreview.history.preserve, overwrite: importPreview.history.overwrite, add: importPreview.history.add }) }}</span>
          </div>
          <div class="modal-info-row">
            <span>{{ t('scraper.importWatchLaterCounts', { preserve: importPreview.watch_later.preserve, overwrite: importPreview.watch_later.overwrite, add: importPreview.watch_later.add }) }}</span>
          </div>
          <div class="import-disclosure">
            <strong>{{ t('scraper.importPreservedDataTitle') }}</strong>
            <p>{{ t('scraper.importPreservedDataDescription') }}</p>
          </div>
          <div class="import-disclosure">
            <strong>{{ t('scraper.importHistoryOrderingTitle') }}</strong>
            <p>{{ t('scraper.importHistoryOrderingDescription') }}</p>
          </div>
        </div>
        <div v-if="importError" class="alert alert-danger import-status">
          {{ importError }}
        </div>
        <div class="modal-actions">
          <button @click="closeImportModal" class="btn-secondary" :disabled="importExecuteLoading">{{ t('scraper.cancel') }}</button>
          <button @click="confirmWatchDataImport" class="btn-primary" :disabled="!importPreview || importExecuteLoading">
            {{ importExecuteLoading ? t('scraper.importExecuting') : t('scraper.importConfirm') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.scraper-view {
  padding: 2rem;
  max-width: 800px;
  margin: 0 auto;
  height: 100%;
  box-sizing: border-box;
  overflow-y: auto;
}

h2 {
  margin: 0 0 1.5rem 0;
}

.alert {
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 1.5rem;
  font-size: 0.9rem;
}

.alert-warning {
  background: rgba(251, 191, 36, 0.1);
  border: 1px solid rgba(251, 191, 36, 0.3);
  color: #fbbf24;
}

.alert-danger {
  background: rgba(220, 53, 69, 0.12);
  border: 1px solid rgba(220, 53, 69, 0.35);
  color: #ff808f;
}

.alert-success {
  background: rgba(34, 197, 94, 0.12);
  border: 1px solid rgba(34, 197, 94, 0.3);
  color: #86efac;
}

.status-card {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.status-title {
  font-weight: 700;
}

.status-message {
  color: var(--text-primary);
}

.storage-row {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.storage-description {
  margin: 0.75rem 0 0;
  font-size: 0.8rem;
  color: var(--text-secondary);
}

.modal-info-row {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 0.75rem;
  font-size: 0.9rem;
}

.modal-info-row strong {
  text-align: right;
  word-break: break-word;
}

.insufficient-storage-alert {
  margin-bottom: 1rem;
}

.path-info {
  padding: 0.75rem 1rem;
  background: var(--bg-surface);
  border-radius: 8px;
  margin-bottom: 1.5rem;
  font-size: 0.85rem;
  border: 1px solid var(--border-color);
}

.path-info .label {
  color: var(--text-secondary);
}

.path-info code {
  color: var(--accent-primary);
  font-family: monospace;
  word-break: break-all;
}

h3 {
  margin: 0 0 1rem 0;
  font-size: 1rem;
}

.stats-card {
  display: flex;
  gap: 2rem;
  padding: 1.5rem;
  background: var(--bg-surface);
  border-radius: 8px;
  margin-bottom: 1.5rem;
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.stat .label {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.stat .value {
  font-size: 1.25rem;
  font-weight: 600;
}

.config-form {
  padding: 1.5rem;
  background: var(--bg-surface);
  border-radius: 8px;
  margin-bottom: 1.5rem;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  font-size: 0.875rem;
  margin-bottom: 0.25rem;
  color: var(--text-secondary);
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

.form-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.hint {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.actions {
  margin-bottom: 1.5rem;
}

.btn-primary {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 4px;
  background: var(--accent-primary);
  color: white;
  font-size: 1rem;
  cursor: pointer;
}

.btn-secondary {
  padding: 0.75rem 1.5rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: transparent;
  color: var(--text-primary);
  font-size: 1rem;
  cursor: pointer;
}

.btn-danger {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 4px;
  background: #dc3545;
  color: white;
  font-size: 1rem;
  cursor: pointer;
}

.btn-primary:disabled,
.btn-secondary:disabled,
.btn-danger:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.progress-card {
  padding: 1.5rem;
  background: var(--bg-surface);
  border-radius: 8px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
}

.status {
  color: var(--accent-primary);
}

.progress-bar {
  height: 8px;
  background: var(--bg-primary);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-primary);
  transition: width 0.3s;
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
  background: var(--bg-surface);
  padding: 1.5rem;
  border-radius: 8px;
  max-width: 400px;
}

.modal h3 {
  margin-bottom: 0.75rem;
}

.modal p {
  color: var(--text-secondary);
  margin-bottom: 1.5rem;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

.import-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.import-actions {
  display: flex;
  justify-content: flex-start;
}

.import-status {
  margin-bottom: 0;
}

.import-modal {
  max-width: 560px;
}

.import-preview-grid {
  margin-bottom: 1rem;
}

.import-disclosure {
  padding: 0.75rem 1rem;
  border-radius: 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  margin-bottom: 0.75rem;
}

.import-disclosure strong {
  display: block;
  margin-bottom: 0.5rem;
}

.import-disclosure p {
  margin: 0;
}
</style>
