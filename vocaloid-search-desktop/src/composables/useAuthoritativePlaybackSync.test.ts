import { describe, expect, test, vi } from 'vitest'
import { useAuthoritativePlaybackSync } from './useAuthoritativePlaybackSync'

vi.mock('../api/tauri-commands', () => ({
  api: {
    getPlaylistState: vi.fn().mockResolvedValue({
      playlist_type: 'History',
      playlist_version: 12,
      results: [
        { id: 'sm1', title: 'Video 1' },
        { id: 'sm2', title: 'Video 2' },
      ],
      index: 1,
      has_next: false,
      pip_active: true,
    }),
  },
}))

describe('useAuthoritativePlaybackSync', () => {
  test('includes playlist identity when syncing active playback', async () => {
    const syncState = vi.fn()
    const { refreshActivePlayback } = useAuthoritativePlaybackSync(syncState)

    await refreshActivePlayback()

    expect(syncState).toHaveBeenCalledWith({
      currentVideo: { id: 'sm2', title: 'Video 2' },
      currentVideoIndex: 1,
      resultsCount: 2,
      hasNext: false,
      pipActive: true,
      playlistType: 'History',
      playlistVersion: 12,
    })
  })
})
