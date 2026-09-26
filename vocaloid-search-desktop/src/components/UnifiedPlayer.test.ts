// @vitest-environment jsdom

import { createApp, nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import { afterEach, describe, expect, test, vi } from 'vitest'
import type { Video } from '../api/tauri-commands'

const video: Video = {
  id: 'sm9',
  title: 'selected title',
  thumbnail_url: null,
  watch_url: null,
  view_count: 0,
  comment_count: 0,
  mylist_count: 0,
  like_count: 0,
  start_time: null,
  tags: [],
  duration: null,
  uploader_id: null,
  uploader_name: null,
  description: null,
  is_watched: false,
}

vi.mock('../composables/usePlayerCore', async () => {
  // Vitest hoists module mocks before static imports, so Vue reactivity must load in this factory.
  const { ref } = await import('vue')
  let iframe: HTMLIFrameElement | null = null
  const player = {
    currentVideo: ref<Video | null>(null),
    currentIndex: ref(0),
    hasNext: ref(true),
    metadataReady: ref(false),
    isPlaying: ref(false),
    playerReady: ref(false),
    playbackSessionKey: ref('Search:1:0:sm9'),
    autoPlay: ref(false),
    autoSkip: ref(false),
    skipThreshold: ref(30),
    currentUserInfo: ref(null),
    setIframeRef(value: HTMLIFrameElement | null) {
      iframe = value
    },
    async handleVideoChange(value: Video | null, index: number, hasNext: boolean) {
      player.currentVideo.value = value
      player.currentIndex.value = index
      player.hasNext.value = hasNext
      player.playerReady.value = false
      player.isPlaying.value = false
    },
    updateIndex() {},
    updateHasNext() {},
    resetState() {},
    togglePlayPause() {
      if (!player.playerReady.value) return
      iframe?.contentWindow?.postMessage(
        { eventName: player.isPlaying.value ? 'pause' : 'play', playerId: '1', sourceConnectorType: 1 },
        'https://embed.nicovideo.jp',
      )
    },
    playNext() {},
    playPrevious() {},
    handlePlayerMessage(event: MessageEvent) {
      const data = event.data as { eventName?: string }
      if (data.eventName === 'loadComplete') player.playerReady.value = true
    },
    setupEventListeners: () => () => {},
    playbackSettingsOpen: ref(false),
    togglePlaybackSettingsPanel() {},
    updatePlaybackSettings() {},
    async loadSettings() {},
    getUserNickname: () => '',
    getUserIconUrl: () => null,
  }

  return { usePlayerCore: () => player }
})


vi.mock('./WatchLaterButton.vue', () => ({
  default: { template: '<button class="watch-later-btn" />' },
}))

import UnifiedPlayer from './UnifiedPlayer.vue'

function mountPlayer(mode: 'full' | 'compact' = 'full') {
  const root = document.createElement('div')
  document.body.append(root)
  const i18n = createI18n({
    legacy: false,
    locale: 'en',
    messages: { en: { player: { previous: 'Previous', next: 'Next' } } },
  })
  const app = createApp(UnifiedPlayer, {
    mode,
    currentVideo: video,
    currentVideoIndex: 0,
    resultsCount: 1,
    hasNext: true,
    playlistType: 'Search',
    playlistVersion: 1,
    showAutoSkip: false,
  })
  app.use(i18n)
  app.mount(root)

  return { app, root }
}

afterEach(() => {
  document.body.replaceChildren()
})

describe('UnifiedPlayer readiness lifecycle', () => {
  test.each(['full', 'compact'] as const)('disables play before the %s embedded player is ready', async (mode) => {
    const { app, root } = mountPlayer(mode)
    await nextTick()

    const playPause = root.querySelector<HTMLButtonElement>('.play-pause-btn')
    expect(playPause?.disabled).toBe(true)

    app.unmount()
  })

  test.each(['full', 'compact'] as const)('captures synchronous initial iframe loadComplete before %s mount and sends its play command', async (mode) => {
    const originalAddEventListener = window.addEventListener
    const originalSrc = Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype, 'src')
    const messageSource = { postMessage: vi.fn() }
    let messageListener: ((event: MessageEvent) => void) | null = null

    window.addEventListener = ((type: string, listener: EventListenerOrEventListenerObject, options?: boolean | AddEventListenerOptions) => {
      if (type === 'message' && typeof listener === 'function') {
        messageListener = listener as (event: MessageEvent) => void
      }
      originalAddEventListener.call(window, type, listener, options)
    }) as typeof window.addEventListener
    Object.defineProperty(HTMLIFrameElement.prototype, 'src', {
      configurable: true,
      get: originalSrc?.get,
      set(value: string) {
        originalSrc?.set?.call(this, value)
        messageListener?.({
          origin: 'https://embed.nicovideo.jp',
          source: messageSource as unknown as MessageEventSource,
          data: { eventName: 'loadComplete', playerId: '1', sourceConnectorType: 0 },
        } as MessageEvent)
      },
    })

    try {
      const { app, root } = mountPlayer(mode)
      await nextTick()

      const playPause = root.querySelector<HTMLButtonElement>('.play-pause-btn')
      const iframe = root.querySelector('iframe')
      const iframeWindow = iframe?.contentWindow
      if (!iframeWindow) throw new Error('mounted player iframe has no content window')
      const postMessage = vi.spyOn(iframeWindow, 'postMessage')

      expect(playPause?.disabled).toBe(false)
      playPause?.click()
      expect(postMessage).toHaveBeenCalledWith(
        { eventName: 'play', playerId: '1', sourceConnectorType: 1 },
        'https://embed.nicovideo.jp',
      )

      app.unmount()
    } finally {
      window.addEventListener = originalAddEventListener
      if (originalSrc) Object.defineProperty(HTMLIFrameElement.prototype, 'src', originalSrc)
    }
  })
})
