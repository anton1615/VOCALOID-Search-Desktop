export type VideoMetaPanelDisplayMode = 'header' | 'details' | 'full'

export type VideoMetaPanelLayout = {
  showHeader: boolean
  showDetails: boolean
}

export function shouldShowDescriptionToggle({
  scrollHeight,
  clientHeight,
}: {
  scrollHeight: number
  clientHeight: number
}): boolean {
  return scrollHeight > clientHeight
}

export function observeDescriptionToggleResize(
  element: HTMLElement,
  onResize: () => void,
): () => void {
  if (typeof ResizeObserver === 'undefined') {
    return () => {}
  }

  const observer = new ResizeObserver(() => {
    onResize()
  })

  observer.observe(element)

  return () => {
    observer.disconnect()
  }
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
