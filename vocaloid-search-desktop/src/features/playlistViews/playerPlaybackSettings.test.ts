import { describe, expect, test } from 'vitest'
import {
  createMainWindowPlaybackSettingsViewModel,
  updatePlaybackSettings,
} from './playerPlaybackSettings'

describe('playerPlaybackSettings', () => {
  test('main window settings panel is collapsed by default', () => {
    expect(createMainWindowPlaybackSettingsViewModel(false)).toEqual({
      panelOpen: false,
      visibleToggles: [],
      showSkipThreshold: false,
    })
  })

  test('opening the main window settings panel shows only autoPlay and autoSkip toggles', () => {
    expect(createMainWindowPlaybackSettingsViewModel(true)).toEqual({
      panelOpen: true,
      visibleToggles: ['autoPlay', 'autoSkip'],
      showSkipThreshold: false,
    })
  })

  test('updating playback settings preserves the internal skip threshold', () => {
    expect(updatePlaybackSettings(
      { autoPlay: true, autoSkip: false, skipThreshold: 30 },
      { autoPlay: false, autoSkip: true },
    )).toEqual({
      autoPlay: false,
      autoSkip: true,
      skipThreshold: 30,
    })
  })
})
