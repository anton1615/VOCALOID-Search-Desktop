export const BLANK_AVATAR_URL = 'https://secure-dcdn.cdn.nimg.jp/nicoaccount/usericon/defaults/blank.jpg'

export interface UploaderAvatarState {
  currentSrc: string | null
  showPlaceholder: boolean
}

export function createUploaderAvatarState(primarySrc: string | null): UploaderAvatarState {
  if (!primarySrc) {
    return {
      currentSrc: null,
      showPlaceholder: true,
    }
  }

  return {
    currentSrc: primarySrc,
    showPlaceholder: false,
  }
}

export function resolveUploaderAvatarStateAfterError(state: UploaderAvatarState): UploaderAvatarState {
  if (!state.currentSrc) {
    return {
      currentSrc: null,
      showPlaceholder: true,
    }
  }

  if (state.currentSrc === BLANK_AVATAR_URL) {
    return {
      currentSrc: null,
      showPlaceholder: true,
    }
  }

  return {
    currentSrc: BLANK_AVATAR_URL,
    showPlaceholder: false,
  }
}
