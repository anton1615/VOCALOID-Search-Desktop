import { ref, type Ref } from 'vue'
import { type Video, type UserInfo, getUploaderAvatarUrl } from '../api/tauri-commands'

/**
 * Return type for usePlayerInfo
 */
export interface PlayerInfo {
  currentUserInfo: Ref<UserInfo | null>
  getUserNickname: (video: Video | null) => string
  getUserIconUrl: (video: Video | null) => string | null
  clearCurrentUserInfo: () => void
}

/**
 * Composable for deriving uploader presentation from the Rust-provided playback video.
 */
export function usePlayerInfo(): PlayerInfo {
  const currentUserInfo = ref<UserInfo | null>(null)

  /**
   * Get user nickname for display
   */
  function getUserNickname(video: Video | null): string {
    if (!video) return ''
    return video.uploader_name || video.uploader_id || ''
  }

  /**
   * Get user icon URL for display
   */
  function getUserIconUrl(video: Video | null): string | null {
    if (!video) return null
    return getUploaderAvatarUrl(video.uploader_id)
  }

  /**
   * Clear current user info (called when player state is reset)
   */
  function clearCurrentUserInfo(): void {
    currentUserInfo.value = null
  }

  return {
    currentUserInfo,
    getUserNickname,
    getUserIconUrl,
    clearCurrentUserInfo,
  }
}
