import type { HistoryEntry, Video, WatchLaterEntry } from '../api/tauri-commands'

function createPlaceholderVideo(entry: {
  video_id: string
  title: string
  thumbnail_url: string | null
  start_time: string
  is_watched: boolean
}): Video {
  return {
    id: entry.video_id,
    title: entry.title,
    thumbnail_url: entry.thumbnail_url,
    watch_url: null,
    is_watched: entry.is_watched,
    start_time: entry.start_time,
    duration: null,
    view_count: 0,
    like_count: 0,
    mylist_count: 0,
    comment_count: 0,
    description: null,
    tags: [],
    uploader_id: null,
    uploader_name: null,
  }
}

export function mapHistoryEntryToVideo(entry: HistoryEntry): Video {
  return createPlaceholderVideo({
    video_id: entry.video_id,
    title: entry.title,
    thumbnail_url: entry.thumbnail_url,
    start_time: entry.watched_at,
    is_watched: true,
  })
}

export function mapWatchLaterEntryToVideo(entry: WatchLaterEntry): Video {
  return createPlaceholderVideo({
    video_id: entry.video_id,
    title: entry.title,
    thumbnail_url: entry.thumbnail_url,
    start_time: entry.added_at,
    is_watched: false,
  })
}
