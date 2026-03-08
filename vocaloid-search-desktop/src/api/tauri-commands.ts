import { invoke } from '@tauri-apps/api/core'

export interface Video {
  id: string
  title: string
  thumbnail_url: string | null
  watch_url: string | null
  view_count: number
  comment_count: number
  mylist_count: number
  like_count: number
  start_time: string | null
  tags: string[]
  duration: number | null
  uploader_id: string | null
  uploader_name: string | null
  description: string | null
  is_watched: boolean
}

export interface UserInfo {
  user_id: string | null
  user_nickname: string | null
  user_icon_url: string | null
}

export interface HistoryEntry {
  video_id: string
  title: string
  thumbnail_url: string | null
  watched_at: string
}

export interface WatchLaterEntry {
  video_id: string
  title: string
  thumbnail_url: string | null
  added_at: string
}

export type PlaylistType = 'Search' | 'History' | 'WatchLater'

export interface NumericFilter {
  gte?: number
  lte?: number
}

export interface DateFilter {
  gte?: string
  lte?: string
}

export interface Filters {
  view?: NumericFilter
  mylist?: NumericFilter
  comment?: NumericFilter
  like?: NumericFilter
  start_time?: DateFilter
}

export interface SortWeights {
  view: number
  mylist: number
  comment: number
  like: number
}

export interface SortConfig {
  by: string
  direction: string
  weights?: SortWeights
}

export interface FormulaFilter {
  view_weight: number
  mylist_weight: number
  comment_weight: number
  like_weight: number
  min_score: number
}

export interface SearchRequest {
  query?: string
  page?: number
  page_size?: number
  exclude_watched?: boolean
  filters?: Filters
  sort?: SortConfig
  formula_filter?: FormulaFilter
}

export interface SearchResponse {
  total: number
  page: number
  page_size: number
  has_next: boolean
  results: Video[]
}

export interface HistoryResponse {
  total: number
  page: number
  page_size: number
  has_next: boolean
  results: HistoryEntry[]
}

export interface WatchLaterResponse {
  total: number
  page: number
  page_size: number
  has_next: boolean
  results: WatchLaterEntry[]
}

export interface HistoryState {
  page: number
  page_size: number
  has_next: boolean
  total_count: number
  sort_direction: string
  search_query: string
}

export interface WatchLaterState {
  page: number
  page_size: number
  has_next: boolean
  total_count: number
  sort_direction: string
  search_query: string
}


export interface ScraperConfig {
  query: string
  max_age_days: number | null
  targets: string
  category_filter: string | null
}

export interface ScraperProgress {
  is_running: boolean
  videos_fetched: number
  total_expected: number | null
  status: string
}

export interface DatabaseStats {
  total_videos: number
  last_update: string | null
}

export interface FreshnessCheck {
  is_fresh: boolean
  local_last_update: string | null
  api_last_update: string | null
  message: string
}

export interface PlaylistState {
  playlist_type: PlaylistType
  results: Video[]
  index: number
  has_next: boolean
  pip_active: boolean
}

export interface PlaybackSettings {
  auto_play: boolean
  auto_skip: boolean
  skip_threshold: number
}

export interface SearchState {
  query: string
  exclude_watched: boolean
  filters?: Filters
  sort?: SortConfig
  formula_filter?: FormulaFilter
  page: number
  page_size: number
  has_next: boolean
  total_count: number
}

export interface VideoSelectedPayload {
  video: Video
  index: number
  has_next: boolean
  playlist_type: PlaylistType
}

export interface PipWindowState {
  x: number
  y: number
  width: number
  height: number
}

export const api = {
  search: async (request: SearchRequest): Promise<SearchResponse> => {
    return invoke('search', { request })
  },
  
  getVideo: async (videoId: string): Promise<Video | null> => {
    return invoke('get_video', { videoId })
  },
  
  getUserInfo: async (videoId: string): Promise<UserInfo | null> => {
    return invoke('get_user_info', { videoId })
  },
  
  fetchVideoMetadata: async (videoId: string): Promise<Video | null> => {
    return invoke('fetch_video_metadata', { videoId })
  },
  
  markWatched: async (videoId: string, title: string, thumbnailUrl: string | null): Promise<void> => {
    return invoke('mark_watched', { videoId, title, thumbnailUrl })
  },
  
  getWatched: async (): Promise<string[]> => {
    return invoke('get_watched')
  },
  
  getHistory: async (page: number = 1, pageSize: number = 50, sortDirection?: string): Promise<HistoryResponse> => {
    return invoke('get_history', { page, pageSize, sortDirection })
  },
  
  getScraperConfig: async (): Promise<ScraperConfig> => {
    return invoke('get_scraper_config')
  },
  
  saveScraperConfig: async (config: ScraperConfig): Promise<void> => {
    return invoke('save_scraper_config', { config })
  },
  
  runScraper: async (): Promise<void> => {
    return invoke('run_scraper')
  },
  
  getScraperProgress: async (): Promise<ScraperProgress> => {
    return invoke('get_scraper_progress')
  },
  
  cancelScraper: async (): Promise<void> => {
    return invoke('cancel_scraper')
  },
  
  getDatabaseStats: async (): Promise<DatabaseStats> => {
    return invoke('get_database_stats')
  },
  
  checkDatabaseFreshness: async (): Promise<FreshnessCheck> => {
    return invoke('check_database_freshness')
  },
  
  openPipWindow: async (): Promise<void> => {
    return invoke('open_pip_window')
  },

  closePipWindow: async (): Promise<void> => {
    return invoke('close_pip_window')
  },

  notifyPipClosing: async (): Promise<void> => {
    return invoke('notify_pip_closing')
  },

  getPlaylistState: async (): Promise<PlaylistState> => {
    return invoke('get_playlist_state')
  },

  setPlaylistIndex: async (index: number): Promise<void> => {
    return invoke('set_playlist_index', { index })
  },

  updatePlaylistVideo: async (index: number, video: Video): Promise<void> => {
    return invoke('update_playlist_video', { index, video })
  },

  getPlaybackSettings: async (): Promise<PlaybackSettings> => {
    return invoke('get_playback_settings')
  },

  setPlaybackSettings: async (settings: PlaybackSettings): Promise<void> => {
    return invoke('set_playback_settings', { settings })
  },

  getSearchState: async (): Promise<SearchState> => {
    return invoke('get_search_state')
  },

  setSearchState: async (searchState: SearchState): Promise<void> => {
    return invoke('set_search_state', { searchState })
  },

  loadMore: async (): Promise<SearchResponse> => {
    return invoke('load_more')
  },

  savePipWindowState: async (state: PipWindowState): Promise<void> => {
    return invoke('save_pip_window_state', { state })
  },

  loadPipWindowState: async (): Promise<PipWindowState | null> => {
    return invoke('load_pip_window_state')
  },

  selectVideo: async (videoId: string): Promise<void> => {
    return invoke('select_video', { videoId })
  },
  getDatabasePath: async (): Promise<string> => {
    return invoke('get_database_path')
  },
  
  // Watch Later API
  fetchFullVideoInfo: async (videoId: string): Promise<Video> => {
    return invoke('fetch_full_video_info', { videoId })
  },
  
  addToWatchLater: async (videoId: string, title: string, thumbnailUrl: string | null): Promise<void> => {
    return invoke('add_to_watch_later', { videoId, title, thumbnailUrl })
  },
  
  removeFromWatchLater: async (videoId: string): Promise<void> => {
    return invoke('remove_from_watch_later', { videoId })
  },
  
  isInWatchLater: async (videoId: string): Promise<boolean> => {
    return invoke('is_in_watch_later', { videoId })
  },
  
  getWatchLater: async (page: number = 1, pageSize: number = 50, sortDirection?: string): Promise<WatchLaterResponse> => {
    return invoke('get_watch_later', { page, pageSize, sortDirection })
  },
  
  getWatchLaterCount: async (): Promise<number> => {
    return invoke('get_watch_later_count')
  },
  
  // State Management API
  getHistoryState: async (): Promise<HistoryState> => {
    return invoke('get_history_state')
  },
  
  setHistoryState: async (historyState: HistoryState): Promise<void> => {
    return invoke('set_history_state', { historyState })
  },
  
  getWatchLaterState: async (): Promise<WatchLaterState> => {
    return invoke('get_watch_later_state')
  },
  
  setWatchLaterState: async (watchLaterState: WatchLaterState): Promise<void> => {
    return invoke('set_watch_later_state', { watchLaterState })
  },
  
  setPlaylistType: async (playlistType: PlaylistType): Promise<void> => {
    return invoke('set_playlist_type', { playlistType })
  },
}

export function formatDuration(seconds: number | null): string {
  if (!seconds) return '--:--'
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

export function formatNumber(n: number): string {
  return n.toLocaleString('en-US')
}

export function getUploaderAvatarUrl(uploaderId: string | null): string | null {
  if (!uploaderId) return null
  const bucket = Math.floor(parseInt(uploaderId) / 10000)
  return `https://secure-dcdn.cdn.nimg.jp/nicoaccount/usericon/${bucket}/${uploaderId}.jpg`
}
