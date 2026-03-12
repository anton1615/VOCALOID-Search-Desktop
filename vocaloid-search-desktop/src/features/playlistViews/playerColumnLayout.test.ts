import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'
import { getPlayerColumnLayout } from './playerColumnLayout'

describe('playerColumnLayout', () => {
  test('returns the right column sections in header, player, controls, details order', () => {
    expect(getPlayerColumnLayout()).toEqual([
      {
        section: 'header',
        videoMetaPanelMode: 'header',
      },
      {
        section: 'player',
      },
      {
        section: 'controls',
      },
      {
        section: 'details',
        videoMetaPanelMode: 'details',
      },
    ])
  })

  test('app shell owns the shared main-window player region', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const historyViewPath = resolve(__dirname, '../../views/HistoryView.vue')
    const watchLaterViewPath = resolve(__dirname, '../../views/WatchLaterView.vue')

    const appSource = readFileSync(appPath, 'utf8')
    const searchSource = readFileSync(searchViewPath, 'utf8')
    const historySource = readFileSync(historyViewPath, 'utf8')
    const watchLaterSource = readFileSync(watchLaterViewPath, 'utf8')

    expect(appSource).toContain('PlayerColumn')
    expect(appSource).toContain('api.fetchFullVideoInfo')
    expect(searchSource).not.toContain('<PlayerColumn')
    expect(historySource).not.toContain('<PlayerColumn')
    expect(watchLaterSource).not.toContain('<PlayerColumn')
  })

  test('split layout keeps the list pane scrollable after shell extraction', () => {
    const appPath = resolve(__dirname, '../../App.vue')
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')

    const appSource = readFileSync(appPath, 'utf8')
    const searchSource = readFileSync(searchViewPath, 'utf8')

    expect(appSource).toContain('.list-pane {')
    expect(appSource).toContain('flex: 1;')
    expect(searchSource).toContain('.list-column {')
    expect(searchSource).toContain('height: 100%;')
    expect(searchSource).toContain('overflow: hidden;')
    expect(searchSource).toContain('.video-list {')
    expect(searchSource).toContain('overflow-y: auto;')
  })

  test('search view uses a dedicated list container ref for scrolling instead of document query selection', () => {
    const searchViewPath = resolve(__dirname, '../../views/SearchView.vue')
    const searchSource = readFileSync(searchViewPath, 'utf8')

    expect(searchSource).toContain('const listContainerRef = ref<HTMLElement | null>(null)')
    expect(searchSource).toContain('const listContainer = listContainerRef.value')
    expect(searchSource).not.toContain("document.querySelector('.video-list')")
    expect(searchSource).toContain('ref="listContainerRef"')
  })
})
