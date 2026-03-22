export type VideoMetaPanelDisplayMode = 'header' | 'details' | 'full'

export type VideoMetaPanelPresentationMode = 'full' | 'compact'

export type CompactHeaderLineState = 'single-line' | 'two-line'

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
  statsFirstInlinePriority: boolean
  uploaderTruncatesBeforeStats: boolean
  urlTreatment: 'surface'
  showTagDescriptionDivider: boolean
  emphasizedMeta: boolean
  fixedMetaRowHeight: boolean
  compactHeaderLineState?: CompactHeaderLineState
}

export type CompactTitleLineMetrics = {
  titleHeight: number
  lineHeight: number
  tolerance?: number
}

export type CompactTitleContentHeightMetrics = {
  elementHeight: number
  minHeight: number
}

const DEFAULT_COMPACT_TITLE_TOLERANCE = 2

const FULL_MODE_PRESENTATION_CONTRACT: VideoMetaPresentationContract = {
  titleClampLines: 1,
  uploaderClampLines: 1,
  avatarSize: 'md',
  statsGap: 'normal',
  statsInlineSpacing: false,
  statsFirstInlinePriority: false,
  uploaderTruncatesBeforeStats: false,
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
  statsFirstInlinePriority: true,
  uploaderTruncatesBeforeStats: true,
  urlTreatment: 'surface',
  showTagDescriptionDivider: true,
  emphasizedMeta: false,
  fixedMetaRowHeight: true,
  compactHeaderLineState: 'two-line',
}

export function getVideoMetaPresentationContract(
  presentationMode: VideoMetaPanelPresentationMode,
  compactHeaderLineState: CompactHeaderLineState = 'two-line',
): VideoMetaPresentationContract {
  if (presentationMode !== 'compact') {
    return FULL_MODE_PRESENTATION_CONTRACT
  }

  return {
    ...COMPACT_MODE_PRESENTATION_CONTRACT,
    compactHeaderLineState,
  }
}

export function getCompactTitleContentHeight({
  elementHeight,
  minHeight,
}: CompactTitleContentHeightMetrics): number {
  return Math.max(elementHeight - minHeight, 0)
}

export function getCompactTitleMeasuredHeight({
  contentHeight,
  lineHeight,
}: {
  contentHeight: number
  lineHeight: number
}): number {
  return contentHeight + lineHeight
}

export function getCompactTitleLineState({
  titleHeight,
  lineHeight,
  tolerance = DEFAULT_COMPACT_TITLE_TOLERANCE,
}: CompactTitleLineMetrics): CompactHeaderLineState {
  return titleHeight <= lineHeight + tolerance ? 'single-line' : 'two-line'
}

export function observeCompactTitleResize(
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
  return observeCompactTitleResize(element, onResize)
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
