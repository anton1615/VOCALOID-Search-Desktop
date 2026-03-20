import { beforeEach, describe, expect, test, vi } from 'vitest'
import { usePlayerCore } from './usePlayerCore'

const usePlayerEventsMock = vi.fn()
const fetchUserInfoMock = vi.fn()
let capturedEventOptions: any = null

vi.mock('./usePlayerEvents', () => ({
  usePlayerEvents: (options: unknown) => {
    capturedEventOptions = options
    usePlayerEventsMock(options)
    return {
      setupEventListeners: () => () => {},
    }
  },
}))

vi.mock('./usePlayerSettings', () => ({
  usePlayerSettings: () => ({
    autoPlay: { value: true },
    autoSkip: { value: false },
    skipThreshold: { value: 30 },
    syncFromBackend: vi.fn(),
    loadSettings: vi.fn(),
  }),
}))

vi.mock('./usePlayerInfo', () => ({
  usePlayerInfo: () => ({
    currentUserInfo: { value: null },
    fetchUserInfo: fetchUserInfoMock,
    getUserNickname: (video: { uploader_name?: string | null; uploader_id?: string | null }) => video.uploader_name || video.uploader_id || '',
    getUserIconUrl: () => null,
    clearCurrentUserInfo: vi.fn(),
  }),
}))

describe('usePlayerCore playback metadata updates', () => {
  beforeEach(() => {
    capturedEventOptions = null
    usePlayerEventsMock.mockClear()
    fetchUserInfoMock.mockClear()
  })

  test('selection keeps metadata hidden until a matching playback metadata update arrives', async () => {
    const player = usePlayerCore({
      onPlayNext: vi.fn(),
      onMarkWatched: vi.fn(),
      onPlaybackStateChanged: vi.fn(),
      setupEvents: true,
      getPlaybackIdentity: () => ({
        playlistType: 'History',
        playlistVersion: 4,
        currentIndex: 1,
        videoId: 'sm9',
      }),
    })

    const selectedPayload = {
      playlist_type: 'History' as const,
      playlist_version: 4,
      index: 1,
      has_next: true,
      video: { id: 'sm9', title: 'selected title' },
    }

    await capturedEventOptions.onVideoSelected(selectedPayload)

    expect(player.metadataReady.value).toBe(false)

    await capturedEventOptions.onPlaybackMetadataUpdated({
      playlist_type: 'History',
      playlist_version: 4,
      index: 1,
      list_id: 'History',
      video: { id: 'sm9', title: 'updated title' },
    })

    expect(player.metadataReady.value).toBe(true)
  })

  test('same playback identity can re-enter pending and only matching enrichment makes metadata ready again', async () => {
    const player = usePlayerCore({
      onPlayNext: vi.fn(),
      onMarkWatched: vi.fn(),
      onPlaybackStateChanged: vi.fn(),
      setupEvents: true,
      getPlaybackIdentity: () => ({
        playlistType: 'History',
        playlistVersion: 4,
        currentIndex: 1,
        videoId: 'sm9',
      }),
    })

    const selectedPayload = {
      playlist_type: 'History' as const,
      playlist_version: 4,
      index: 1,
      has_next: true,
      video: { id: 'sm9', title: 'selected title' },
    }

    await capturedEventOptions.onVideoSelected(selectedPayload)
    expect(player.metadataReady.value).toBe(false)

    await capturedEventOptions.onPlaybackMetadataUpdated({
      playlist_type: 'History',
      playlist_version: 4,
      index: 1,
      list_id: 'History',
      video: { id: 'sm9', title: 'updated title' },
    })
    expect(player.metadataReady.value).toBe(true)

    await capturedEventOptions.onVideoSelected(selectedPayload)
    expect(player.metadataReady.value).toBe(false)

    await capturedEventOptions.onPlaybackMetadataUpdated({
      playlist_type: 'History',
      playlist_version: 5,
      index: 1,
      list_id: 'History',
      video: { id: 'sm9', title: 'stale title' },
    })
    expect(player.metadataReady.value).toBe(false)

    await capturedEventOptions.onPlaybackMetadataUpdated({
      playlist_type: 'History',
      playlist_version: 4,
      index: 1,
      list_id: 'History',
      video: { id: 'sm9', title: 'updated again' },
    })
    expect(player.metadataReady.value).toBe(true)
  })

  test('matching playback metadata updates do not rerun selection reset side effects', async () => {
    const player = usePlayerCore({
      onPlayNext: vi.fn(),
      onMarkWatched: vi.fn(),
      onPlaybackStateChanged: vi.fn(),
      setupEvents: true,
      getPlaybackIdentity: () => ({
        playlistType: 'History',
        playlistVersion: 4,
        currentIndex: 1,
        videoId: 'sm9',
      }),
    })

    const selectedPayload = {
      playlist_type: 'History' as const,
      playlist_version: 4,
      index: 1,
      has_next: true,
      video: { id: 'sm9', title: 'selected title' },
    }

    await capturedEventOptions.onVideoSelected(selectedPayload)

    player.playerReady.value = true
    player.isPlaying.value = true

    await capturedEventOptions.onPlaybackMetadataUpdated({
      playlist_type: 'History',
      playlist_version: 4,
      index: 1,
      list_id: 'History',
      video: { id: 'sm9', title: 'updated title' },
    })

    expect(player.currentVideo.value).toEqual({ id: 'sm9', title: 'updated title' })
    expect(player.currentIndex.value).toBe(1)
    expect(player.hasNext.value).toBe(true)
    expect(player.playerReady.value).toBe(true)
    expect(player.isPlaying.value).toBe(true)
  })

  test('matching playback metadata updates update local video immediately before parent refresh lands', async () => {
    const player = usePlayerCore({
      onPlayNext: vi.fn(),
      onMarkWatched: vi.fn(),
      onPlaybackStateChanged: vi.fn(),
      setupEvents: true,
      getPlaybackIdentity: () => ({
        playlistType: 'History',
        playlistVersion: 4,
        currentIndex: 0,
        videoId: 'sm1',
      }),
    })

    const selectedPayload = {
      playlist_type: 'History' as const,
      playlist_version: 4,
      index: 1,
      has_next: true,
      video: { id: 'sm9', title: 'selected title' },
    }

    await capturedEventOptions.onVideoSelected(selectedPayload)

    await capturedEventOptions.onPlaybackMetadataUpdated({
      playlist_type: 'History',
      playlist_version: 4,
      index: 1,
      list_id: 'History',
      video: { id: 'sm9', title: 'updated title', uploader_name: 'Tom' },
    })

    expect(player.currentVideo.value).toEqual({ id: 'sm9', title: 'updated title', uploader_name: 'Tom' })
  })

  test('refreshes authoritative playback when local selection matches but parent playback identity still lags', async () => {
    const onPlaybackStateChanged = vi.fn()

    usePlayerCore({
      onPlayNext: vi.fn(),
      onMarkWatched: vi.fn(),
      onPlaybackStateChanged,
      setupEvents: true,
      getPlaybackIdentity: () => ({
        playlistType: 'History',
        playlistVersion: 4,
        currentIndex: 0,
        videoId: 'sm1',
      }),
    })

    await capturedEventOptions.onVideoSelected({
      playlist_type: 'History',
      playlist_version: 4,
      index: 1,
      has_next: true,
      video: { id: 'sm9', title: 'selected title' },
    })
    onPlaybackStateChanged.mockClear()

    await capturedEventOptions.onPlaybackMetadataUpdated({
      playlist_type: 'History',
      playlist_version: 4,
      index: 1,
      list_id: 'History',
      video: { id: 'sm9', title: 'updated title' },
    })

    expect(onPlaybackStateChanged).toHaveBeenCalledTimes(1)
  })

  test('refreshes authoritative playback only for matching playback metadata updates', async () => {
    const onPlaybackStateChanged = vi.fn()

    usePlayerCore({
      onPlayNext: vi.fn(),
      onMarkWatched: vi.fn(),
      onPlaybackStateChanged,
      setupEvents: true,
      getPlaybackIdentity: () => ({
        playlistType: 'History',
        playlistVersion: 4,
        currentIndex: 1,
        videoId: 'sm9',
      }),
    })

    await capturedEventOptions.onPlaybackMetadataUpdated({
      playlist_type: 'History',
      playlist_version: 4,
      index: 1,
      list_id: 'History',
      video: { id: 'sm9', title: 'updated title' },
    })

    expect(onPlaybackStateChanged).toHaveBeenCalledTimes(1)

    await capturedEventOptions.onPlaybackMetadataUpdated({
      playlist_type: 'History',
      playlist_version: 5,
      index: 1,
      list_id: 'History',
      video: { id: 'sm9', title: 'stale title' },
    })

    expect(onPlaybackStateChanged).toHaveBeenCalledTimes(1)
  })
})
