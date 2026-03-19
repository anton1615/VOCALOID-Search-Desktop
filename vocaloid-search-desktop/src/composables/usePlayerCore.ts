import { ref, type Ref } from 'vue'
import type { PlaybackIdentityPayload, PlaybackVideoUpdatedPayload, Video, VideoSelectedPayload } from '../api/tauri-commands'
import { usePlayerEvents, type PlayerEventsOptions } from './usePlayerEvents'
import { usePlayerSettings } from './usePlayerSettings'
import { usePlayerInfo } from './usePlayerInfo'
import { resolvePlayerCommandTarget } from '../features/playlistViews/playerCommandTarget'
import { rememberPlayerMessageSource, clearPlayerMessageSource, type PostMessageTarget } from '../features/playlistViews/playerMessageSource'

/**
 * Options for creating the player core
 */
export interface PlayerCoreOptions {
  /** Called when play next is triggered */
  onPlayNext: () => void
  /** Called when play previous is triggered */
  onPlayPrevious?: () => void
  /** Called when a video is marked as watched */
  onMarkWatched: (video: Video) => void
  /** Called when player state is cleared (e.g., from active-playback-cleared event) */
  onStateCleared?: () => void
  /** Called when backend authoritative playback state changed and parent should refresh */
  onPlaybackStateChanged?: () => void | Promise<void>
  /** Returns the currently displayed playback identity for stale metadata filtering */
  getPlaybackIdentity?: () => PlaybackIdentityPayload
  /** Function to schedule a callback (used for auto-play delay) */
  schedule?: (callback: () => void) => void
  /** Whether this is the PIP window */
  isPip?: boolean
  /** Whether to set up event listeners */
  setupEvents?: boolean
}

/**
 * Return type for usePlayerCore
 */
export interface PlayerCore {
  // State
  currentVideo: Ref<Video | null>
  currentIndex: Ref<number>
  isPlaying: Ref<boolean>
  playerReady: Ref<boolean>
  hasNext: Ref<boolean>
  metadataReady: Ref<boolean>

  // Settings
  autoPlay: Ref<boolean>
  autoSkip: Ref<boolean>
  skipThreshold: Ref<number>
  
  // User info
  currentUserInfo: ReturnType<typeof usePlayerInfo>['currentUserInfo']
  getUserNickname: () => string
  getUserIconUrl: () => string | null
  
  // Actions
  setIframeRef: (iframe: HTMLIFrameElement | null) => void
  handleVideoChange: (video: Video | null, index: number, hasNextVideo: boolean) => Promise<void>
  updateIndex: (index: number, hasNextVideo: boolean) => void
  updateHasNext: (hasNextVideo: boolean) => void
  resetState: () => void
  togglePlayPause: () => void
  playNext: () => void
  playPrevious: () => void
  
  // Event handlers
  handlePlayerMessage: (event: MessageEvent) => void
  setupEventListeners: () => () => void
  
  // Settings panel
  playbackSettingsOpen: Ref<boolean>
  togglePlaybackSettingsPanel: () => void
  
  // Settings sync
  updatePlaybackSettings: (settings: { autoPlay: boolean; autoSkip: boolean; skipThreshold: number }) => void
  loadSettings: () => Promise<void>
}

/**
 * Core player composable that orchestrates all player functionality.
 * This is the main entry point for player logic in both main and PIP windows.
 */
export function usePlayerCore(options: PlayerCoreOptions): PlayerCore {
  const {
    onPlayNext,
    onPlayPrevious,
    onMarkWatched,
    onStateCleared,
    onPlaybackStateChanged,
    getPlaybackIdentity,
    schedule = (cb) => setTimeout(cb, 500),
    isPip = false,
    setupEvents = true,
  } = options

  // Core state
  const currentVideo = ref<Video | null>(null)
  const currentIndex = ref(-1)
  const isPlaying = ref(false)
  const playerReady = ref(false)
  const hasNext = ref(false)
  const metadataReady = ref(false)
  const selectedPlaybackIdentity = ref<PlaybackIdentityPayload | null>(null)
  
  // Settings panel state
  const playbackSettingsOpen = ref(false)

  // Player settings
  const settings = usePlayerSettings()

  // User info management
  const playerInfo = usePlayerInfo()

  // Iframe reference for sending commands
  let iframeRef: HTMLIFrameElement | null = null
  let lastPlayerMessageSource: PostMessageTarget | null = null

  // Internal player state for embedded controller
  let hasMarkedCurrent = false

  /**
   * Set the iframe reference
   */
  function setIframeRef(iframe: HTMLIFrameElement | null): void {
    iframeRef = iframe
  }

  /**
   * Send a command to the player iframe
   */
  function sendCommand(command: string): void {
    const target = resolvePlayerCommandTarget({
      lastMessageSource: lastPlayerMessageSource,
      iframeWindow: iframeRef?.contentWindow ?? null,
    })

    target?.postMessage(
      {
        eventName: command,
        playerId: '1',
        sourceConnectorType: 1,
      },
      'https://embed.nicovideo.jp'
    )
  }

  /**
   * Handle video change from external source (e.g., video-selected event)
   */
  async function handleVideoChange(video: Video | null, index: number, hasNextVideo: boolean): Promise<void> {
    // Clear previous message source
    lastPlayerMessageSource = clearPlayerMessageSource(lastPlayerMessageSource)

    currentVideo.value = video
    currentIndex.value = index
    hasNext.value = hasNextVideo
    playerReady.value = false
    isPlaying.value = false
    hasMarkedCurrent = false

    if (!video) {
      metadataReady.value = false
      selectedPlaybackIdentity.value = null
      playerInfo.clearCurrentUserInfo()
      return
    }

    const displayedIdentity = resolveDisplayedPlaybackIdentity(video, index)
    const selectionStillPending = !metadataReady.value && samePlaybackIdentity(displayedIdentity, selectedPlaybackIdentity.value)
    metadataReady.value = !selectionStillPending
  }

  /**
   * Update index and hasNext without changing video
   */
  function updateIndex(index: number, hasNextVideo: boolean): void {
    currentIndex.value = index
    hasNext.value = hasNextVideo
  }

  /**
   * Update hasNext flag
   */
  function updateHasNext(hasNextVideo: boolean): void {
    hasNext.value = hasNextVideo
  }

  /**
   * Reset player state (called when active-playback-cleared event is received)
   */
  function resetState(): void {
    currentVideo.value = null
    currentIndex.value = -1
    isPlaying.value = false
    playerReady.value = false
    hasNext.value = false
    metadataReady.value = false
    selectedPlaybackIdentity.value = null
    hasMarkedCurrent = false
    playerInfo.clearCurrentUserInfo()
    
    // Notify parent
    onStateCleared?.()
  }

  /**
   * Toggle play/pause
   */
  function togglePlayPause(): void {
    if (!playerReady.value) return
    sendCommand(isPlaying.value ? 'pause' : 'play')
  }

  /**
   * Play next video
   */
  function playNext(): void {
    if (hasNext.value) {
      onPlayNext()
    }
  }

  /**
   * Play previous video
   */
  function playPrevious(): void {
    if (currentIndex.value > 0 && onPlayPrevious) {
      onPlayPrevious()
    }
  }

  /**
   * Handle message from player iframe
   */
  function handlePlayerMessage(event: MessageEvent): void {
    if (!event.data || event.origin !== 'https://embed.nicovideo.jp') return

    // Remember message source for sending commands
    lastPlayerMessageSource = rememberPlayerMessageSource(event.source)

    const data = typeof event.data === 'string' ? JSON.parse(event.data) : event.data

    // Handle load complete
    if (data.eventName === 'loadComplete') {
      playerReady.value = true
      if (settings.autoPlay.value) {
        schedule(() => sendCommand('play'))
      }
      return
    }

    // Handle player status change
    if (data.eventName === 'playerStatusChange' || data.eventName === 'statusChange') {
      const status = data.data?.playerStatus
      const statusNum = typeof status === 'string' ? parseInt(status, 10) : status

      if (statusNum === 2) {
        // Playing
        isPlaying.value = true
        if (currentVideo.value && !hasMarkedCurrent) {
          onMarkWatched(currentVideo.value)
          hasMarkedCurrent = true
        }
        return
      }

      if (statusNum === 3) {
        // Paused
        isPlaying.value = false
        return
      }

      if (statusNum === 4) {
        // Ended
        isPlaying.value = false
        return
      }
      return
    }

    // Handle player metadata change (for auto-skip)
    if (data.eventName === 'playerMetadataChange') {
      const currentTime = data.data?.currentTime
      const duration = data.data?.duration
      if (currentTime && duration && settings.autoSkip.value) {
        const remaining = duration - currentTime
        if (remaining <= settings.skipThreshold.value && currentTime > 10) {
          playNext()
        }
      }
    }
  }

  /**
   * Update playback settings
   */
  function updatePlaybackSettings(newSettings: { autoPlay: boolean; autoSkip: boolean; skipThreshold: number }): void {
    settings.autoPlay.value = newSettings.autoPlay
    settings.autoSkip.value = newSettings.autoSkip
    settings.skipThreshold.value = newSettings.skipThreshold
  }

  /**
   * Load settings from backend
   */
  async function loadSettings(): Promise<void> {
    await settings.loadSettings()
  }

  function samePlaybackIdentity(left: PlaybackIdentityPayload | null | undefined, right: PlaybackIdentityPayload | null | undefined): boolean {
    if (!left || !right) {
      return false
    }

    return (
      left.playlistType === right.playlistType &&
      left.playlistVersion === right.playlistVersion &&
      left.currentIndex === right.currentIndex &&
      left.videoId === right.videoId
    )
  }

  function matchesPlaybackIdentity(identity: PlaybackIdentityPayload | null | undefined, payload: PlaybackVideoUpdatedPayload): boolean {
    if (!identity) {
      return false
    }

    return (
      identity.playlistType === payload.playlist_type &&
      identity.playlistVersion === payload.playlist_version &&
      identity.currentIndex === payload.index &&
      identity.videoId === payload.video.id
    )
  }

  function resolveDisplayedPlaybackIdentity(video: Video, index: number): PlaybackIdentityPayload | null {
    const currentIdentity = getPlaybackIdentity?.()
    if (currentIdentity?.currentIndex === index && currentIdentity.videoId === video.id) {
      return currentIdentity
    }

    if (selectedPlaybackIdentity.value?.currentIndex === index && selectedPlaybackIdentity.value.videoId === video.id) {
      return selectedPlaybackIdentity.value
    }

    return null
  }

  function matchesCurrentPlaybackIdentity(payload: PlaybackVideoUpdatedPayload): boolean {
    const currentIdentity = getPlaybackIdentity?.()
    return matchesPlaybackIdentity(currentIdentity, payload) || matchesPlaybackIdentity(selectedPlaybackIdentity.value, payload)
  }

  function handleVideoSelected(payload: VideoSelectedPayload): Promise<void> {
    selectedPlaybackIdentity.value = {
      playlistType: payload.playlist_type,
      playlistVersion: payload.playlist_version,
      currentIndex: payload.index,
      videoId: payload.video.id,
    }
    metadataReady.value = false
    return handleVideoChange(payload.video, payload.index, payload.has_next)
  }

  function handlePlaybackMetadataUpdated(payload: PlaybackVideoUpdatedPayload): Promise<void> | void {
    if (!matchesCurrentPlaybackIdentity(payload)) {
      return
    }

    currentVideo.value = payload.video
    metadataReady.value = true
    return onPlaybackStateChanged?.()
  }

  function syncVideoFromProps(video: Video | null, index: number, hasNextVideo: boolean): Promise<void> {
    return handleVideoChange(video, index, hasNextVideo)
  }

  // Set up event listeners if requested
  let eventCleanup: (() => void) | null = null

  const eventOptions: PlayerEventsOptions = {
    isPip,
    onVideoSelected: async (payload) => {
      await handleVideoSelected(payload)
      await onPlaybackStateChanged?.()
    },
    onPlaybackMetadataUpdated: async (payload) => {
      await handlePlaybackMetadataUpdated(payload)
    },
    onPlaybackSettingsChanged: (newSettings) => {
      settings.syncFromBackend(newSettings)
    },
    onVideoWatched: (videoId, isWatched) => {
      if (currentVideo.value?.id === videoId) {
        currentVideo.value = { ...currentVideo.value, is_watched: isWatched }
      }
    },
    onActivePlaybackCleared: async () => {
      resetState()
      await onPlaybackStateChanged?.()
    },
  }

  if (!setupEvents) {
    eventOptions.onVideoSelected = async (payload) => {
      await handleVideoSelected(payload)
    }
    eventOptions.onActivePlaybackCleared = () => {
      resetState()
    }
  }

  function syncFromProps(video: Video | null, index: number, hasNextVideo: boolean): Promise<void> {
    return syncVideoFromProps(video, index, hasNextVideo)
  }

  const { setupEventListeners: setupEventsInternal } = usePlayerEvents(eventOptions)

  function setupEventListeners(): () => void {
    if (setupEvents) {
      eventCleanup = setupEventsInternal()
    }
    return () => {
      if (eventCleanup) {
        eventCleanup()
        eventCleanup = null
      }
    }
  }

  /**
   * Toggle playback settings panel
   */
  function togglePlaybackSettingsPanel(): void {
    playbackSettingsOpen.value = !playbackSettingsOpen.value
  }

  /**
   * Get user nickname for display
   */
  function getUserNickname(): string {
    if (!currentVideo.value) return ''
    return playerInfo.getUserNickname(currentVideo.value)
  }

  /**
   * Get user icon URL for display
   */
  function getUserIconUrl(): string | null {
    if (!currentVideo.value) return null
    return playerInfo.getUserIconUrl(currentVideo.value)
  }

  return {
    // State
    currentVideo,
    currentIndex,
    isPlaying,
    playerReady,
    hasNext,
    metadataReady,

    // Settings
    autoPlay: settings.autoPlay,
    autoSkip: settings.autoSkip,
    skipThreshold: settings.skipThreshold,
    
    // User info
    currentUserInfo: playerInfo.currentUserInfo,
    getUserNickname,
    getUserIconUrl,
    
    // Actions
    setIframeRef,
    handleVideoChange: syncFromProps,
    updateIndex,
    updateHasNext,
    resetState,
    togglePlayPause,
    playNext,
    playPrevious,
    
    // Event handlers
    handlePlayerMessage,
    setupEventListeners,
    
    // Settings panel
    playbackSettingsOpen,
    togglePlaybackSettingsPanel,
    
    // Settings sync
    updatePlaybackSettings,
    loadSettings,
  }
}
