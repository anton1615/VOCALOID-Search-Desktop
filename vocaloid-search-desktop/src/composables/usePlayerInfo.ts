import { ref, reactive, type Ref } from 'vue'
import { api, type Video, type UserInfo, getUploaderAvatarUrl } from '../api/tauri-commands'

/**
 * Return type for usePlayerInfo
 */
export interface PlayerInfo {
  userInfoCache: Map<string, UserInfo>
  currentUserInfo: Ref<UserInfo | null>
  
  fetchUserInfo: (video: Video) => Promise<void>
  getUserNickname: (video: Video | null) => string
  getUserIconUrl: (video: Video | null) => string | null
  clearCurrentUserInfo: () => void
}

/**
 * Composable for managing user info caching.
 * Avoids redundant API calls for uploader information.
 */
export function usePlayerInfo(): PlayerInfo {
  // Cache for user info by video ID
  const userInfoCache = reactive(new Map<string, UserInfo>())
  
  // Current video's user info
  const currentUserInfo = ref<UserInfo | null>(null)
  
  // Track which video the current user info belongs to
  let currentVideoId: string | null = null

  /**
   * Fetch user info for a video, using cache if available
   */
  async function fetchUserInfo(video: Video): Promise<void> {
    if (!video.uploader_id) {
      currentUserInfo.value = null
      currentVideoId = null
      return
    }

    // Check cache first
    if (!userInfoCache.has(video.id)) {
      try {
        const userInfo = await api.getUserInfo(video.id)
        if (userInfo) {
          userInfoCache.set(video.id, userInfo)
        }
      } catch (e) {
        console.error('Failed to fetch user info:', e)
      }
    }

    currentUserInfo.value = userInfoCache.get(video.id) || null
    currentVideoId = video.id
  }

  /**
   * Get user nickname for display
   */
  function getUserNickname(video: Video | null): string {
    if (!video) return ''
    
    // Check if we have cached info for this video
    if (video.id === currentVideoId && currentUserInfo.value?.user_nickname) {
      return currentUserInfo.value.user_nickname
    }
    
    // Check cache
    const cached = userInfoCache.get(video.id)
    if (cached?.user_nickname) {
      return cached.user_nickname
    }
    
    // Fallback to uploader name or ID
    return video.uploader_name || video.uploader_id || ''
  }

  /**
   * Get user icon URL for display
   */
  function getUserIconUrl(video: Video | null): string | null {
    if (!video) return null
    
    // Check if we have cached info for this video
    if (video.id === currentVideoId && currentUserInfo.value?.user_icon_url) {
      return currentUserInfo.value.user_icon_url
    }
    
    // Check cache
    const cached = userInfoCache.get(video.id)
    if (cached?.user_icon_url) {
      return cached.user_icon_url
    }
    
    // Fallback to avatar URL
    return getUploaderAvatarUrl(video.uploader_id)
  }

  /**
   * Clear current user info (called when player state is reset)
   */
  function clearCurrentUserInfo(): void {
    currentUserInfo.value = null
    currentVideoId = null
  }

  return {
    userInfoCache,
    currentUserInfo,
    fetchUserInfo,
    getUserNickname,
    getUserIconUrl,
    clearCurrentUserInfo,
  }
}
