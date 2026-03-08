import type { VideoMetaPanelDisplayMode } from './videoMetaPanelLayout'

export type PipSidebarControl = 'watchLater' | 'previous' | 'playPause' | 'next'

export type PipContentSection =
  | {
      section: 'header' | 'details'
      videoMetaPanelMode: Extract<VideoMetaPanelDisplayMode, 'header' | 'details'>
    }
  | {
      section: 'player'
    }

export interface PipLayout {
  sidebar: PipSidebarControl[]
  content: PipContentSection[]
}

export function getPipLayout(): PipLayout {
  return {
    sidebar: ['watchLater', 'previous', 'playPause', 'next'],
    content: [
      {
        section: 'header',
        videoMetaPanelMode: 'header',
      },
      {
        section: 'player',
      },
      {
        section: 'details',
        videoMetaPanelMode: 'details',
      },
    ],
  }
}
