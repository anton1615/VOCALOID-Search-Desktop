import { ref, watch, type Ref } from 'vue'
import { api, type PlaybackSettings } from '../api/tauri-commands'

/**
 * Options for player settings
 */
export interface PlayerSettingsOptions {
  /** Called when settings are changed */
  onSettingsChanged?: (settings: { autoPlay: boolean; autoSkip: boolean; skipThreshold: number }) => void
}

/**
 * Return type for usePlayerSettings
 */
export interface PlayerSettings {
  autoPlay: Ref<boolean>
  autoSkip: Ref<boolean>
  skipThreshold: Ref<number>
  syncing: Ref<boolean>
  
  loadSettings: () => Promise<void>
  syncFromBackend: (settings: PlaybackSettings) => void
  persistSettings: (updates: Partial<{ autoPlay: boolean; autoSkip: boolean }>) => Promise<void>
}

/**
 * Composable for managing playback settings.
 * Handles loading, syncing, and persisting settings.
 */
export function usePlayerSettings(options: PlayerSettingsOptions = {}): PlayerSettings {
  const { onSettingsChanged } = options

  // Settings state
  const autoPlay = ref(localStorage.getItem('vocaloidAutoPlay') !== 'false')
  const autoSkip = ref(localStorage.getItem('vocaloidAutoSkip') === 'true')
  const skipThreshold = ref(parseInt(localStorage.getItem('vocaloidSkipThreshold') || '30', 10))
  
  // Syncing flag to prevent loops
  const syncing = ref(false)

  /**
   * Load settings from backend
   */
  async function loadSettings(): Promise<void> {
    try {
      const settings = await api.getPlaybackSettings()
      syncFromBackend(settings)
    } catch (e) {
      console.error('Failed to load playback settings:', e)
    }
  }

  /**
   * Sync settings from backend event
   */
  function syncFromBackend(settings: PlaybackSettings): void {
    syncing.value = true
    autoPlay.value = settings.auto_play
    autoSkip.value = settings.auto_skip
    skipThreshold.value = settings.skip_threshold
    syncing.value = false
  }

  /**
   * Persist settings to backend
   */
  async function persistSettings(updates: Partial<{ autoPlay: boolean; autoSkip: boolean }>): Promise<void> {
    if (syncing.value) return

    const nextSettings = {
      autoPlay: updates.autoPlay ?? autoPlay.value,
      autoSkip: updates.autoSkip ?? autoSkip.value,
      skipThreshold: skipThreshold.value,
    }

    try {
      await api.setPlaybackSettings({
        auto_play: nextSettings.autoPlay,
        auto_skip: nextSettings.autoSkip,
        skip_threshold: nextSettings.skipThreshold,
      })
      
      onSettingsChanged?.(nextSettings)
    } catch (e) {
      console.error('Failed to persist playback settings:', e)
    }
  }

  // Watch for changes and persist to localStorage
  watch(autoPlay, (val, oldVal) => {
    localStorage.setItem('vocaloidAutoPlay', val.toString())
    if (oldVal !== undefined && val !== oldVal && !syncing.value) {
      void persistSettings({ autoPlay: val, autoSkip: autoSkip.value })
    }
  })

  watch(autoSkip, (val, oldVal) => {
    localStorage.setItem('vocaloidAutoSkip', val.toString())
    if (oldVal !== undefined && val !== oldVal && !syncing.value) {
      void persistSettings({ autoPlay: autoPlay.value, autoSkip: val })
    }
  })

  watch(skipThreshold, (val) => {
    localStorage.setItem('vocaloidSkipThreshold', val.toString())
  })

  return {
    autoPlay,
    autoSkip,
    skipThreshold,
    syncing,
    loadSettings,
    syncFromBackend,
    persistSettings,
  }
}
