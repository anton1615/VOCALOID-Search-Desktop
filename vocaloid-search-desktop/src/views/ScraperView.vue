<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject } from 'vue'
import { useI18n } from 'vue-i18n'
import { api, type ScraperConfig, type ScraperProgress, type DatabaseStats } from '../api/tauri-commands'

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
const freshnessMessage = inject<string>('freshnessMessage', '')
const databasePath = ref('')

async function loadDatabasePath() {
  try {
    databasePath.value = await api.getDatabasePath()
  } catch (e) {
    console.error('Failed to get database path:', e)
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

async function saveConfig() {
  try {
    await api.saveScraperConfig(config.value)
  } catch (e) {
    console.error('Failed to save config:', e)
  }
}

async function startSync() {
  if (stats.value.total_videos > 0) {
    showConfirm.value = true
  } else {
    await runScraper()
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

function startPolling() {
  progressPollInterval.value = window.setInterval(async () => {
    try {
      const p = await api.getScraperProgress()
      progress.value = p
      if (!p.is_running) {
        stopPolling()
        await loadStats()
      }
    } catch (e) {
      console.error('Failed to get progress:', e)
    }
  }, 1000)
}

function stopPolling() {
  if (progressPollInterval.value) {
    clearInterval(progressPollInterval.value)
    progressPollInterval.value = null
  }
}

const categoryOptions = [
  { value: 'MUSIC', label: '音樂' },
  { value: 'GAME', label: '遊戲' },
  { value: 'ANIME', label: '動畫' },
  { value: 'ENTERTAINMENT', label: '娛樂' },
  { value: 'DANCE', label: '舞蹈' },
  { value: 'OTHER', label: '其他' },
]

const targetOptions = [
  { value: 'tags', label: '標籤' },
  { value: 'tagsExact', label: '標籤 (精確)' },
  { value: 'title', label: '標題' },
  { value: 'description', label: '描述' },
  { value: 'tags,title', label: '標籤 + 標題' },
]

onMounted(async () => {
  await loadConfig()
  await loadStats()
  await loadDatabasePath()
  
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
    
    <div v-if="freshnessMessage" class="alert alert-warning">
      ⚠️ {{ freshnessMessage }}
    </div>
    
    <div class="stats-card">
      <div class="stat">
        <span class="label">{{ t('scraper.totalVideos') }}</span>
        <span class="value">{{ stats.total_videos.toLocaleString() }}</span>
      </div>
      <div class="stat">
        <span class="label">{{ t('scraper.lastUpdate') }}</span>
        <span class="value">
          {{ stats.last_update ? new Date(stats.last_update).toLocaleString() : t('scraper.neverUpdated') }}
        </span>
      </div>
    </div>
    
    <div v-if="databasePath" class="path-info">
      <span class="label">Database:</span>
      <code>{{ databasePath }}</code>
    </div>
    
    <div class="config-form">
      <h3>{{ t('scraper.syncSettings') }}</h3>
      
      <div class="form-group">
        <label>{{ t('scraper.searchKeyword') }}</label>
        <input
          v-model="config.query"
          type="text"
          @change="saveConfig"
          placeholder="VOCALOID, UTAU, CeVIO..."
        />
      </div>
      
      <div class="form-row">
        <div class="form-group">
          <label>{{ t('scraper.maxDays') }}</label>
          <input
            v-model.number="config.max_age_days"
            type="number"
            @change="saveConfig"
            placeholder="0 = unlimited"
          />
          <span class="hint">0 = unlimited</span>
        </div>
        
        <div class="form-group">
          <label>{{ t('scraper.category') }}</label>
          <select v-model="config.category_filter" @change="saveConfig">
            <option v-for="opt in categoryOptions" :key="opt.value" :value="opt.value">
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
        <p>{{ t('scraper.confirmClear') }}</p>
        <div class="modal-actions">
          <button @click="showConfirm = false" class="btn-secondary">{{ t('scraper.cancel') }}</button>
          <button @click="runScraper" class="btn-primary">OK</button>
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
</style>
