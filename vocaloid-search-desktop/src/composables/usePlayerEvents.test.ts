import { beforeEach, describe, expect, test, vi } from 'vitest'
import type { PlaybackVideoUpdatedPayload, Video } from '../api/tauri-commands'
import { usePlayerEvents } from './usePlayerEvents'

const listeners = new Map<string, (event: { payload: unknown }) => unknown>()
const playbackVideo: Video = {
  id: 'sm9',
  title: 'Updated title',
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


vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (eventName: string, handler: (event: { payload: unknown }) => unknown) => {
    listeners.set(eventName, handler)
    return () => {
      listeners.delete(eventName)
    }
  }),
}))

describe('usePlayerEvents', () => {
  beforeEach(() => {
    listeners.clear()
    vi.clearAllMocks()
  })

  test('forwards video-selected events to selection handler without triggering metadata update handler', async () => {
    const onVideoSelected = vi.fn()
    const onPlaybackMetadataUpdated = vi.fn()

    const events = usePlayerEvents({
      onVideoSelected,
      onPlaybackMetadataUpdated,
      onPlaybackSettingsChanged: vi.fn(),
      onVideoWatched: vi.fn(),
      onActivePlaybackCleared: vi.fn(),
    })

    events.setupEventListeners()

    const payload = {
      playlist_type: 'History',
      playlist_version: 7,
      index: 2,
      has_next: true,
      video: {
        id: 'sm9',
        title: 'Selected title',
      },
    }

    await listeners.get('video-selected')?.({ payload })

    expect(onVideoSelected).toHaveBeenCalledWith(payload)
    expect(onPlaybackMetadataUpdated).not.toHaveBeenCalled()
  })

  test('forwards playback-video-updated events to metadata update handler without triggering selection handler', async () => {
    const onVideoSelected = vi.fn()
    const onPlaybackMetadataUpdated = vi.fn()

    const events = usePlayerEvents({
      onVideoSelected,
      onPlaybackMetadataUpdated,
      onPlaybackSettingsChanged: vi.fn(),
      onVideoWatched: vi.fn(),
      onActivePlaybackCleared: vi.fn(),
    })

    events.setupEventListeners()

    const payload: PlaybackVideoUpdatedPayload = {
      list_id: 'History',
      playlist_type: 'History',
      playlist_version: 7,
      index: 2,
      video: playbackVideo,
    }

    await listeners.get('playback-video-updated')?.({ payload })

    expect(onPlaybackMetadataUpdated).toHaveBeenCalledWith(payload)
    expect(onVideoSelected).not.toHaveBeenCalled()
  })
})
