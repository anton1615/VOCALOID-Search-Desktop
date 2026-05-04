import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'

describe('scraper sync flow', () => {
  test('provides freshness status as reactive state instead of a raw injected string value', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const scraperViewPath = resolve(__dirname, '../../views/ScraperView.vue')
    const appSource = readFileSync(appPath, 'utf8')
    const scraperViewSource = readFileSync(scraperViewPath, 'utf8')

    expect(appSource).toContain("provide('freshnessStatus'")
    expect(appSource).not.toContain("provide('freshnessMessage', freshnessMessage.value)")
    expect(scraperViewSource).toContain("inject<Ref")
    expect(scraperViewSource).toContain("'freshnessStatus'")
  })

  test('always opens the sync confirmation dialog through a preflight estimate step', () => {
    const scraperViewPath = resolve(__dirname, '../../views/ScraperView.vue')
    const source = readFileSync(scraperViewPath, 'utf8')

    expect(source).toContain('await api.getSyncPreflightEstimate()')
    expect(source).not.toContain('if (stats.value.total_videos > 0)')
    expect(source).toContain('showConfirm.value = true')
  })

  test('refreshes freshness status from backend when scraper polling observes completion', () => {
    const scraperViewPath = resolve(__dirname, '../../views/ScraperView.vue')
    const source = readFileSync(scraperViewPath, 'utf8')

    expect(source).toContain('async function refreshFreshnessStatus()')
    expect(source).toContain('await api.checkDatabaseFreshness()')
    expect(source).toContain('await refreshFreshnessStatus()')
  })

  test('makes scraper route content scrollable so sync progress is not clipped', () => {
    const scraperViewPath = resolve(__dirname, '../../views/ScraperView.vue')
    const source = readFileSync(scraperViewPath, 'utf8')

    expect(source).toContain('height: 100%;')
    expect(source).toContain('overflow-y: auto;')
  })

  test('renders structured storage information instead of the raw database path block', () => {
    const scraperViewPath = resolve(__dirname, '../../views/ScraperView.vue')
    const source = readFileSync(scraperViewPath, 'utf8')

    expect(source).toContain("t('scraper.storageTitle')")
    expect(source).toContain("t('scraper.dataDirectory')")
    expect(source).not.toContain('<span class="label">Database:</span>')
  })

  test('exposes a complete category list plus a no-filter option', () => {
    const scraperViewPath = resolve(__dirname, '../../views/ScraperView.vue')
    const source = readFileSync(scraperViewPath, 'utf8')

    expect(source).toContain("value: null")
    expect(source).toContain("t('scraper.categoryNone')")
    expect(source).toContain("value: 'ANIMAL'")
    expect(source).toContain("value: 'NATURE'")
    expect(source).toContain("value: 'COOKING'")
    expect(source).toContain("value: 'TRAVEL'")
    expect(source).toContain("value: 'VEHICLE'")
    expect(source).toContain("value: 'SPORTS'")
    expect(source).toContain("value: 'SOCIAL'")
    expect(source).toContain("value: 'TECHNICAL'")
    expect(source).toContain("value: 'LECTURE'")
    expect(source).toContain("value: 'RADIO'")
  })

  test('formats estimated video count and blocks confirmation when estimated size exceeds free space', () => {
    const scraperViewPath = resolve(__dirname, '../../views/ScraperView.vue')
    const source = readFileSync(scraperViewPath, 'utf8')

    expect(source).toContain('formatVideoCount(preflightEstimate?.estimated_video_count ?? null)')
    expect(source).toContain('const isStorageInsufficient = computed(() =>')
    expect(source).toContain("t('scraper.insufficientStorageTitle')")
    expect(source).toContain('v-if="!isStorageInsufficient"')
  })

  test('treats entering /scraper as a route-entry playback reset boundary', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const apiPath = resolve(__dirname, '../../api/tauri-commands.ts')
    const source = readFileSync(appPath, 'utf8')
    const apiSource = readFileSync(apiPath, 'utf8')

    expect(source).toContain("watch(() => route.name")
    expect(source).toContain("routeName === 'scraper'")
    expect(source).toContain('await api.resetPlaybackForSyncRouteEntry()')
    expect(source).toContain('await refreshActivePlayback()')
    expect(apiSource).toContain('resetPlaybackForSyncRouteEntry: async (): Promise<void> =>')
    expect(apiSource).toContain("return invoke('reset_playback_for_sync_route_entry')")
  })

  test('guards sync-route playback reset watcher with error handling', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const source = readFileSync(appPath, 'utf8')

    expect(source).toContain("watch(() => route.name, async (routeName, previousRouteName) => {")
    expect(source).toContain('try {')
    expect(source).toContain('await api.resetPlaybackForSyncRouteEntry()')
    expect(source).toContain('await refreshActivePlayback()')
    expect(source).toContain("console.error('Failed to reset playback on sync route entry:'")
  })

  test('adds watch-data import api contracts and command wrappers', () => {
    const apiPath = resolve(__dirname, '../../api/tauri-commands.ts')
    const source = readFileSync(apiPath, 'utf8')

    expect(source).toContain('export interface WatchDataImportCounts')
    expect(source).toContain('add: number')
    expect(source).toContain('export interface WatchDataImportPreviewResponse')
    expect(source).toContain('confirmation_token: string')
    expect(source).toContain('export interface WatchDataImportConfirmedSummary')
    expect(source).toContain('export interface WatchDataImportCompleted')
    expect(source).toContain('previewWatchDataImport: async (path: string): Promise<WatchDataImportPreviewResponse> =>')
    expect(source).toContain("return invoke('preview_watch_data_import', { request: { path } })")
    expect(source).toContain('executeWatchDataImport: async (request: WatchDataImportExecuteRequest): Promise<WatchDataImportCompleted> =>')
    expect(source).toContain("return invoke('execute_watch_data_import', { request })")
  })

  test('keeps watch-data import as a page-local file-preview-confirm flow with its own modal', () => {
    const scraperViewPath = resolve(__dirname, '../../views/ScraperView.vue')
    const source = readFileSync(scraperViewPath, 'utf8')

    expect(source).toContain("import { open } from '@tauri-apps/plugin-dialog'")
    expect(source).toContain('const showImportConfirm = ref(false)')
    expect(source).toContain('const importPreview = ref<WatchDataImportPreviewResponse | null>(null)')
    expect(source).toContain('const importSuccess = ref<WatchDataImportCompleted | null>(null)')
    expect(source).toContain('const importError = ref(\'\')')
    expect(source).toContain('async function selectWatchDataImportFile()')
    expect(source).toContain("filters: [{ name: 'SQLite', extensions: ['db', 'sqlite', 'sqlite3'] }]")
    expect(source).toContain('if (!selectedPath || Array.isArray(selectedPath)) {')
    expect(source).toContain("selectedImportPath.value = ''")
    expect(source).toContain('importPreview.value = await api.previewWatchDataImport(selectedPath)')
    expect(source).toContain('showImportConfirm.value = true')
    expect(source).toContain('async function confirmWatchDataImport()')
    expect(source).toContain('if (!selectedImportPath.value || !importPreview.value) return')
    expect(source).toContain('confirmed_summary: {')
    expect(source).toContain('file_name: importPreview.value.file_name')
    expect(source).toContain('history: importPreview.value.history')
    expect(source).toContain('watch_later: importPreview.value.watch_later')
    expect(source).toContain('importSuccess.value = await api.executeWatchDataImport({')
    expect(source).toContain('showImportConfirm.value = false')
    expect(source).toContain('@click.self="showImportConfirm = false"')
    expect(source).toContain('<div v-if="showImportConfirm" class="modal-backdrop"')
    expect(source).toContain("t('scraper.importTitle')")
    expect(source).toContain("t('scraper.importDescription')")
    expect(source).toContain("t('scraper.importPreservedDataDescription')")
    expect(source).toContain("t('scraper.importHistoryOrderingDescription')")
  })


  test('refreshes mounted views from backend truth when watch-data import completes and localizes startup loading copy', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const pipPath = resolve(__dirname, '../../PipApp.vue')
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const historyViewPath = resolve(__dirname, '../../views/HistoryView.vue')
    const watchLaterViewPath = resolve(__dirname, '../../views/WatchLaterView.vue')
    const appSource = readFileSync(appPath, 'utf8')
    const pipSource = readFileSync(pipPath, 'utf8')
    const searchSource = readFileSync(searchViewPath, 'utf8')
    const historySource = readFileSync(historyViewPath, 'utf8')
    const watchLaterSource = readFileSync(watchLaterViewPath, 'utf8')

    expect(appSource).toContain("listen('watch-data-import-complete', async () => {")
    expect(appSource).toContain("console.log('[App] Received watch-data-import-complete event')")
    expect(appSource).toContain('await refreshActivePlayback()')
    expect(appSource).toContain("<p>{{ t('history.loading') }}</p>")
    expect(appSource).toContain('if (unlistenWatchDataImportComplete) unlistenWatchDataImportComplete()')

    expect(pipSource).toContain("listen('watch-data-import-complete', async () => {")
    expect(pipSource).toContain("console.log('[PiP] Received watch-data-import-complete event')")
    expect(pipSource).toContain('await refreshActivePlayback()')
    expect(pipSource).toContain('if (unlistenWatchDataImportComplete) unlistenWatchDataImportComplete()')

    expect(historySource).toContain("listen('watch-data-import-complete', async () => {")
    expect(historySource).toContain('await loadHistory()')
    expect(historySource).toContain('if (unlistenWatchDataImportComplete) unlistenWatchDataImportComplete()')

    expect(watchLaterSource).toContain("listen('watch-data-import-complete', async () => {")
    expect(watchLaterSource).toContain('await loadWatchLater()')
    expect(watchLaterSource).toContain('if (unlistenWatchDataImportComplete) unlistenWatchDataImportComplete()')

    expect(searchSource).toContain("listen('watch-data-import-complete', async () => {")
    expect(searchSource).toContain('const searchState = await api.getSearchState()')
    expect(searchSource).toContain('const restored = resolveSearchRestoreState(playlistState, searchState)')
    expect(searchSource).toContain('results.value = restored.results')
    expect(searchSource).toContain('currentVideo.value = restored.currentVideo')
    expect(searchSource).toContain('if (unlistenWatchDataImportComplete) unlistenWatchDataImportComplete()')
  })
})
