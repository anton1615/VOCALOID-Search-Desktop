use crate::database::Database;
use crate::models::{ActivePlayback, HistoryState, ListContext, ListContextId, PlaylistType, ScraperConfig, ScraperProgress, SearchState, Video, WatchLaterState};
use async_channel::Sender;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub fn should_accept_list_load_more(
    _active_playlist_type: crate::models::PlaylistType,
    requested_playlist_type: crate::models::PlaylistType,
    current_version: u64,
    expected_version: u64,
) -> bool {
    // For Search, accept regardless of active playlist (browsing is independent of playback)
    // For other lists, we only validate version
    let _ = _active_playlist_type; // suppress unused warning
    requested_playlist_type == crate::models::PlaylistType::Search && current_version == expected_version
}

pub fn should_bump_list_version(current_sort: &str, next_sort: &str) -> bool {
    current_sort != next_sort
}

pub fn should_reset_playback_for_list_change(
    active_playlist_type: crate::models::PlaylistType,
    changed_playlist_type: crate::models::PlaylistType,
) -> bool {
    active_playlist_type == changed_playlist_type
}

pub fn should_accept_list_context_load_more(
    active_list_id: &ListContextId,
    requested_list_id: &ListContextId,
    current_version: u64,
    expected_version: u64,
) -> bool {
    active_list_id == requested_list_id && current_version == expected_version
}

pub struct AppState {
    pub db: Database,
    pub current_video: Arc<RwLock<Option<Video>>>,
    pub pip_active: Arc<RwLock<bool>>,
    // Legacy fields (kept for migration compatibility)
    pub playlist_type: Arc<RwLock<PlaylistType>>,
    pub search_results: Arc<RwLock<Vec<Video>>>,
    pub history_results: Arc<RwLock<Vec<Video>>>,
    pub watch_later_results: Arc<RwLock<Vec<Video>>>,
    pub playlist_index: Arc<RwLock<Option<usize>>>,
    // New list-context model
    pub list_contexts: Arc<RwLock<HashMap<ListContextId, ListContext>>>,
    pub active_playback: Arc<RwLock<Option<ActivePlayback>>>,
    // Other state
    pub auto_play: Arc<RwLock<bool>>,
    pub auto_skip: Arc<RwLock<bool>>,
    pub skip_threshold: Arc<RwLock<u32>>,
    pub scraper_progress: Arc<RwLock<ScraperProgress>>,
    pub scraper_cancel: Arc<RwLock<Option<Sender<()>>>>,
    pub config: Arc<RwLock<ScraperConfig>>,
    pub search_state: Arc<RwLock<SearchState>>,
    pub history_state: Arc<RwLock<HistoryState>>,
    pub watch_later_state: Arc<RwLock<WatchLaterState>>,
}

impl AppState {
    pub fn new(videos_path: PathBuf, user_data_path: PathBuf) -> Self {
        Self {
            db: Database::new(videos_path, user_data_path),
            current_video: Arc::new(RwLock::new(None)),
            pip_active: Arc::new(RwLock::new(false)),
            playlist_type: Arc::new(RwLock::new(PlaylistType::default())),
            search_results: Arc::new(RwLock::new(Vec::new())),
            history_results: Arc::new(RwLock::new(Vec::new())),
            watch_later_results: Arc::new(RwLock::new(Vec::new())),
            playlist_index: Arc::new(RwLock::new(None)),
            list_contexts: Arc::new(RwLock::new(HashMap::new())),
            active_playback: Arc::new(RwLock::new(None)),
            auto_play: Arc::new(RwLock::new(true)),
            auto_skip: Arc::new(RwLock::new(false)),
            skip_threshold: Arc::new(RwLock::new(30)),
            scraper_progress: Arc::new(RwLock::new(ScraperProgress {
                is_running: false,
                videos_fetched: 0,
                total_expected: None,
                status: "idle".to_string(),
            })),
            scraper_cancel: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(ScraperConfig::default())),
            search_state: Arc::new(RwLock::new(SearchState::default())),
            history_state: Arc::new(RwLock::new(HistoryState::default())),
            watch_later_state: Arc::new(RwLock::new(WatchLaterState::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        should_accept_list_load_more,
        should_bump_list_version,
        should_reset_playback_for_list_change,
    };

    #[test]
    fn accepts_search_load_more_even_when_active_playback_is_history() {
        assert!(should_accept_list_load_more(
            crate::models::PlaylistType::History,
            crate::models::PlaylistType::Search,
            4,
            4,
        ));
    }

    #[test]
    fn rejects_stale_search_load_more_requests() {
        assert!(!should_accept_list_load_more(
            crate::models::PlaylistType::Search,
            crate::models::PlaylistType::Search,
            4,
            3,
        ));
    }

    #[test]
    fn accepts_matching_search_load_more_requests() {
        assert!(should_accept_list_load_more(
            crate::models::PlaylistType::Search,
            crate::models::PlaylistType::Search,
            4,
            4,
        ));
    }

    #[test]
    fn bumps_list_version_when_sort_changes() {
        assert!(should_bump_list_version("desc", "asc"));
    }

    #[test]
    fn keeps_list_version_when_sort_does_not_change() {
        assert!(!should_bump_list_version("desc", "desc"));
    }

    #[test]
    fn resets_playback_only_for_the_active_list() {
        assert!(should_reset_playback_for_list_change(
            crate::models::PlaylistType::Search,
            crate::models::PlaylistType::Search,
        ));
        assert!(!should_reset_playback_for_list_change(
            crate::models::PlaylistType::Search,
            crate::models::PlaylistType::History,
        ));
    }
}
