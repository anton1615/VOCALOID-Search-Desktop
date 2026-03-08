export interface PlaybackSettingsState {
  autoPlay: boolean
  autoSkip: boolean
  skipThreshold: number
}

export interface MainWindowPlaybackSettingsViewModel {
  panelOpen: boolean
  visibleToggles: Array<'autoPlay' | 'autoSkip'>
  showSkipThreshold: boolean
}

export function createMainWindowPlaybackSettingsViewModel(panelOpen: boolean): MainWindowPlaybackSettingsViewModel {
  return {
    panelOpen,
    visibleToggles: panelOpen ? ['autoPlay', 'autoSkip'] : [],
    showSkipThreshold: false,
  }
}

export function updatePlaybackSettings(
  current: PlaybackSettingsState,
  updates: Pick<PlaybackSettingsState, 'autoPlay' | 'autoSkip'>,
): PlaybackSettingsState {
  return {
    ...current,
    ...updates,
  }
}
