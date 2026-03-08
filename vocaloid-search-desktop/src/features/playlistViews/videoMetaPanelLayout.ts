export type VideoMetaPanelDisplayMode = 'header' | 'details' | 'full'

export type VideoMetaPanelLayout = {
  showHeader: boolean
  showDetails: boolean
}

export function getVideoMetaPanelLayout(displayMode: VideoMetaPanelDisplayMode): VideoMetaPanelLayout {
  if (displayMode === 'header') {
    return {
      showHeader: true,
      showDetails: false,
    }
  }

  if (displayMode === 'details') {
    return {
      showHeader: false,
      showDetails: true,
    }
  }

  return {
    showHeader: true,
    showDetails: true,
  }
}
