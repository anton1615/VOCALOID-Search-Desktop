import type { Video } from '../../api/tauri-commands'

interface PlaybackSettingsState {
  autoPlay: boolean
  autoSkip: boolean
  skipThreshold: number
}

interface PlayerControllerState {
  playerReady: boolean
  isPlaying: boolean
  hasMarkedCurrent: boolean
}

interface PlayerEventPayload {
  eventName: string
  data?: {
    playerStatus?: number | string
    currentTime?: number
    duration?: number
  }
}

interface CreateEmbeddedPlayerControllerOptions {
  sendCommand: (command: string) => void
  onPlayNext: () => void
  onMarkWatched: (video: Video) => void
  schedule: (callback: () => void) => void
}

export function createEmbeddedPlayerController({
  sendCommand,
  onPlayNext,
  onMarkWatched,
  schedule,
}: CreateEmbeddedPlayerControllerOptions) {
  const state: PlayerControllerState = {
    playerReady: false,
    isPlaying: false,
    hasMarkedCurrent: false,
  }

  const playbackSettings: PlaybackSettingsState = {
    autoPlay: true,
    autoSkip: false,
    skipThreshold: 30,
  }

  let currentVideo: Video | null = null

  return {
    state,

    setCurrentVideo(video: Video | null) {
      currentVideo = video
      state.playerReady = false
      state.isPlaying = false
      state.hasMarkedCurrent = false
    },

    setPlaybackSettings(settings: PlaybackSettingsState) {
      playbackSettings.autoPlay = settings.autoPlay
      playbackSettings.autoSkip = settings.autoSkip
      playbackSettings.skipThreshold = settings.skipThreshold
    },

    handlePlayerEvent(payload: PlayerEventPayload) {
      if (payload.eventName === 'loadComplete') {
        state.playerReady = true
        if (playbackSettings.autoPlay) {
          schedule(() => sendCommand('play'))
        }
        return
      }

      if (payload.eventName === 'playerStatusChange' || payload.eventName === 'statusChange') {
        const status = payload.data?.playerStatus
        const statusNum = typeof status === 'string' ? parseInt(status, 10) : status

        if (statusNum === 2) {
          state.isPlaying = true
          if (currentVideo && !state.hasMarkedCurrent) {
            onMarkWatched(currentVideo)
            state.hasMarkedCurrent = true
          }
          return
        }

        if (statusNum === 3) {
          state.isPlaying = false
          return
        }

        if (statusNum === 4) {
          state.isPlaying = false
          if (playbackSettings.autoPlay) {
            onPlayNext()
          }
        }
        return
      }

      if (payload.eventName === 'playerMetadataChange') {
        const currentTime = payload.data?.currentTime
        const duration = payload.data?.duration
        if (currentTime && duration && playbackSettings.autoSkip) {
          const remaining = duration - currentTime
          if (remaining <= playbackSettings.skipThreshold && currentTime > 10) {
            onPlayNext()
          }
        }
      }
    },
  }
}
