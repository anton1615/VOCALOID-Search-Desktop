import type { VideoMetaPanelDisplayMode } from './videoMetaPanelLayout'

export type PlayerColumnSection =
  | {
      section: 'header' | 'details'
      videoMetaPanelMode: Extract<VideoMetaPanelDisplayMode, 'header' | 'details'>
    }
  | {
      section: 'player' | 'controls'
    }

export function getPlayerColumnLayout(): PlayerColumnSection[] {
  return [
    {
      section: 'header',
      videoMetaPanelMode: 'header',
    },
    {
      section: 'player',
    },
    {
      section: 'controls',
    },
    {
      section: 'details',
      videoMetaPanelMode: 'details',
    },
  ]
}
