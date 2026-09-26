import { nextTick, ref } from 'vue'
import { describe, expect, test, vi } from 'vitest'
import { usePlayerCore } from './usePlayerCore'
import { usePlayerPropSync } from './usePlayerPropSync'
import type { PlaylistType, Video } from '../api/tauri-commands'

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

vi.mock('./usePlayerEvents', () => ({
  usePlayerEvents: () => ({ setupEventListeners: () => () => {} }),
}))

vi.mock('./usePlayerSettings', () => ({
  usePlayerSettings: () => ({
    autoPlay: { value: false },
    autoSkip: { value: false },
    skipThreshold: { value: 30 },
    syncFromBackend: vi.fn(),
    loadSettings: vi.fn(),
  }),
}))

vi.mock('./usePlayerInfo', () => ({
  usePlayerInfo: () => ({
    currentUserInfo: { value: null },
    fetchUserInfo: vi.fn(),
    getUserNickname: () => '',
    getUserIconUrl: () => null,
    clearCurrentUserInfo: vi.fn(),
  }),
}))

describe('player prop synchronization', () => {
  test('changing hasNext preserves playing state and pause command behavior', async () => {
    const currentVideo = ref<Video | null>(video)
    const currentVideoIndex = ref(0)
    const hasNext = ref(false)
    const playlistType = ref<PlaylistType>('Search')
    const playlistVersion = ref(1)
    const postMessage = vi.fn()
    const player = usePlayerCore({
      onPlayNext: vi.fn(),
      onMarkWatched: vi.fn(),
      setupEvents: false,
    })

    player.setIframeRef({
      contentWindow: { postMessage },
    } as unknown as HTMLIFrameElement)
    const stopSyncing = usePlayerPropSync({
      currentVideo,
      currentVideoIndex,
      hasNext,
      playlistType,
      playlistVersion,
      player,
    })
    await nextTick()

    const receive = (data: unknown) => player.handlePlayerMessage({
      data,
      origin: 'https://embed.nicovideo.jp',
      source: null,
    } as MessageEvent)
    receive({ eventName: 'loadComplete' })
    receive({ eventName: 'playerStatusChange', data: { playerStatus: 2 } })
    expect(player.playerReady.value).toBe(true)
    expect(player.isPlaying.value).toBe(true)

    hasNext.value = true
    await nextTick()

    expect(player.playerReady.value).toBe(true)
    expect(player.isPlaying.value).toBe(true)
    player.togglePlayPause()
    expect(postMessage).toHaveBeenLastCalledWith(
      { eventName: 'pause', playerId: '1', sourceConnectorType: 1 },
      'https://embed.nicovideo.jp',
    )
    receive({ eventName: 'playerStatusChange', data: { playerStatus: 3 } })
    expect(player.isPlaying.value).toBe(false)
    player.togglePlayPause()
    expect(postMessage).toHaveBeenLastCalledWith(
      { eventName: 'play', playerId: '1', sourceConnectorType: 1 },
      'https://embed.nicovideo.jp',
    )

    stopSyncing()
  })
})
