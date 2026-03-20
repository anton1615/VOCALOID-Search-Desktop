export type VideoMetaPanelDisplayMode = 'header' | 'details' | 'full'

export type VideoMetaPanelPresentationMode = 'full' | 'compact'

export type VideoMetaPanelLayout = {
  showHeader: boolean
  showDetails: boolean
}

export type VideoMetaPresentationContract = {
  titleClampLines: 1 | 2
  uploaderClampLines: 1
  avatarSize: 'sm' | 'md'
  statsGap: 'normal' | 'spacious'
  statsInlineSpacing: boolean
  urlTreatment: 'surface'
  showTagDescriptionDivider: boolean
  emphasizedMeta: boolean
  fixedMetaRowHeight: boolean
}

const FULL_MODE_PRESENTATION_CONTRACT: VideoMetaPresentationContract = {
  titleClampLines: 1,
  uploaderClampLines: 1,
  avatarSize: 'md',
  statsGap: 'normal',
  statsInlineSpacing: false,
  urlTreatment: 'surface',
  showTagDescriptionDivider: true,
  emphasizedMeta: true,
  fixedMetaRowHeight: true,
}

const COMPACT_MODE_PRESENTATION_CONTRACT: VideoMetaPresentationContract = {
  titleClampLines: 2,
  uploaderClampLines: 1,
  avatarSize: 'sm',
  statsGap: 'spacious',
  statsInlineSpacing: true,
  urlTreatment: 'surface',
  showTagDescriptionDivider: true,
  emphasizedMeta: false,
  fixedMetaRowHeight: true,
}

export function getVideoMetaPresentationContract(
  presentationMode: VideoMetaPanelPresentationMode,
): VideoMetaPresentationContract {
  return presentationMode === 'compact'
    ? COMPACT_MODE_PRESENTATION_CONTRACT
    : FULL_MODE_PRESENTATION_CONTRACT
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
