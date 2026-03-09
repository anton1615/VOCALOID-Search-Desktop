import { describe, expect, test } from 'vitest'
import {
  BLANK_AVATAR_URL,
  createUploaderAvatarState,
  resolveUploaderAvatarStateAfterError,
} from './uploaderAvatarState'

describe('uploaderAvatarState', () => {
  test('uses primary avatar URL before any image error', () => {
    expect(createUploaderAvatarState('https://example.com/avatar.jpg')).toEqual({
      currentSrc: 'https://example.com/avatar.jpg',
      showPlaceholder: false,
    })
  })

  test('falls back to Niconico blank avatar after primary image error', () => {
    expect(resolveUploaderAvatarStateAfterError({
      currentSrc: 'https://example.com/avatar.jpg',
      showPlaceholder: false,
    })).toEqual({
      currentSrc: BLANK_AVATAR_URL,
      showPlaceholder: false,
    })
  })

  test('shows placeholder when no avatar URL is available', () => {
    expect(createUploaderAvatarState(null)).toEqual({
      currentSrc: null,
      showPlaceholder: true,
    })
  })

  test('shows placeholder when blank avatar also fails', () => {
    expect(resolveUploaderAvatarStateAfterError({
      currentSrc: BLANK_AVATAR_URL,
      showPlaceholder: false,
    })).toEqual({
      currentSrc: null,
      showPlaceholder: true,
    })
  })
})
