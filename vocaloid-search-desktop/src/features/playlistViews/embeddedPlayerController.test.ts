import { describe, expect, test, vi } from 'vitest'
import type { Video } from '../../api/tauri-commands'
import { createEmbeddedPlayerController } from './embeddedPlayerController'

const video: Video = {
  id: 'sm9',
  title: 'Test Video',
  thumbnail_url: 'https://example.com/thumb.jpg',
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

describe('createEmbeddedPlayerController', () => {
  test('loadComplete marks player ready and schedules autoplay command when enabled', () => {
    const sendCommand = vi.fn()
    const schedule = vi.fn((cb: () => void) => cb())
    const controller = createEmbeddedPlayerController({
      sendCommand,
      onPlayNext: vi.fn(),
      onMarkWatched: vi.fn(),
      schedule,
    })

    controller.setPlaybackSettings({ autoPlay: true, autoSkip: false, skipThreshold: 30 })
    controller.handlePlayerEvent({ eventName: 'loadComplete' })

    expect(controller.state.playerReady).toBe(true)
    expect(schedule).toHaveBeenCalled()
    expect(sendCommand).toHaveBeenCalledWith('play')
  })

  test('playing status marks current video watched only once', () => {
    const onMarkWatched = vi.fn()
    const controller = createEmbeddedPlayerController({
      sendCommand: vi.fn(),
      onPlayNext: vi.fn(),
      onMarkWatched,
      schedule: vi.fn(),
    })

    controller.setCurrentVideo(video)
    controller.handlePlayerEvent({ eventName: 'playerStatusChange', data: { playerStatus: 2 } })
    controller.handlePlayerEvent({ eventName: 'playerStatusChange', data: { playerStatus: 2 } })

    expect(controller.state.isPlaying).toBe(true)
    expect(onMarkWatched).toHaveBeenCalledTimes(1)
    expect(onMarkWatched).toHaveBeenCalledWith(video)
  })

  test('ended status does not advance to next video when auto-skip is disabled', () => {
    const onPlayNext = vi.fn()
    const controller = createEmbeddedPlayerController({
      sendCommand: vi.fn(),
      onPlayNext,
      onMarkWatched: vi.fn(),
      schedule: vi.fn(),
    })

    controller.setPlaybackSettings({ autoPlay: true, autoSkip: false, skipThreshold: 30 })
    controller.handlePlayerEvent({ eventName: 'playerStatusChange', data: { playerStatus: 4 } })

    expect(onPlayNext).not.toHaveBeenCalled()
  })

  test('metadata event triggers auto-skip near the end when enabled', () => {
    const onPlayNext = vi.fn()
    const controller = createEmbeddedPlayerController({
      sendCommand: vi.fn(),
      onPlayNext,
      onMarkWatched: vi.fn(),
      schedule: vi.fn(),
    })

    controller.setPlaybackSettings({ autoPlay: false, autoSkip: true, skipThreshold: 30 })
    controller.handlePlayerEvent({
      eventName: 'playerMetadataChange',
      data: { currentTime: 100, duration: 120 },
    })

    expect(onPlayNext).toHaveBeenCalledTimes(1)
  })

  test('ended status stays on the current video when both autoplay and auto-skip are disabled', () => {
    const onPlayNext = vi.fn()
    const controller = createEmbeddedPlayerController({
      sendCommand: vi.fn(),
      onPlayNext,
      onMarkWatched: vi.fn(),
      schedule: vi.fn(),
    })

    controller.setPlaybackSettings({ autoPlay: false, autoSkip: false, skipThreshold: 30 })
    controller.handlePlayerEvent({ eventName: 'playerStatusChange', data: { playerStatus: 4 } })

    expect(onPlayNext).not.toHaveBeenCalled()
  })

  test('loadComplete after auto-skip leaves next video paused when autoplay is disabled', () => {
    const sendCommand = vi.fn()
    const onPlayNext = vi.fn()
    const schedule = vi.fn((cb: () => void) => cb())
    const controller = createEmbeddedPlayerController({
      sendCommand,
      onPlayNext,
      onMarkWatched: vi.fn(),
      schedule,
    })

    controller.setPlaybackSettings({ autoPlay: false, autoSkip: true, skipThreshold: 30 })
    controller.handlePlayerEvent({
      eventName: 'playerMetadataChange',
      data: { currentTime: 100, duration: 120 },
    })
    controller.handlePlayerEvent({ eventName: 'loadComplete' })

    expect(onPlayNext).toHaveBeenCalledTimes(1)
    expect(schedule).not.toHaveBeenCalled()
    expect(sendCommand).not.toHaveBeenCalled()
  })
})
