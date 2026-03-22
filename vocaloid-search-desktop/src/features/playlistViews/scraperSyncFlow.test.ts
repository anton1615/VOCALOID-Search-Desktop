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
})
