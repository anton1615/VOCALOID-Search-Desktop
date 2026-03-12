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
    /// Get or create a list context for the given list ID
    pub fn get_or_create_list_context(&self, list_id: ListContextId) -> ListContext {
        let contexts = self.list_contexts.read();
        contexts.get(&list_id).cloned().unwrap_or_default()
    }


    /// Update a list context with new items and browsing state
    /// If page == 1, replaces items, increments version, and updates browsing state
    /// If page > 1, appends items (version and browsing state unchanged)
    pub fn update_list_context(
        &self,
        list_id: ListContextId,
        items: Vec<Video>,
        page: usize,
        page_size: usize,
        has_next: bool,
        total_count: usize,
        query: String,
        sort: Option<crate::models::SortConfig>,
        filters: Option<crate::models::Filters>,
        exclude_watched: bool,
        formula_filter: Option<crate::models::FormulaFilter>,
    ) -> u64 {
        let mut contexts = self.list_contexts.write();
        let context = contexts.entry(list_id.clone()).or_default();
        
        if page == 1 {
            // New search - increment version and replace items and browsing state
            context.version += 1;
            context.items = items;
            context.query = query;
            context.sort = sort;
            context.filters = filters;
            context.exclude_watched = exclude_watched;
            context.formula_filter = formula_filter;
        } else {
            // Pagination - append items (version and browsing state unchanged)
            context.items.extend(items);
        }
        context.page = page;
        context.page_size = page_size;
        context.has_next = has_next;
        context.total_count = total_count;
        context.version
    }


    /// Increment list context version (for sort/filter changes)
    /// Returns the new version
    pub fn bump_list_context_version(&self, list_id: &ListContextId) -> u64 {
        let mut contexts = self.list_contexts.write();
        if let Some(context) = contexts.get_mut(list_id) {
            context.version += 1;
            context.items.clear();
            context.page = 1;
            context.has_next = false;
            context.version
        } else {
            1
        }
    }
    
    /// Get the list context for a given list ID
    pub fn get_list_context(&self, list_id: &ListContextId) -> Option<ListContext> {
        let contexts = self.list_contexts.read();
        contexts.get(list_id).cloned()
    }


    /// Set active playback to a specific list and index
    pub fn set_active_playback(&self, list_id: ListContextId, list_version: u64, index: usize) {
        let mut playback = self.active_playback.write();
        *playback = Some(ActivePlayback {
            list_id,
            list_version,
            current_index: index,
            is_playing: true,
        });
    }

    /// Clear active playback
    pub fn clear_active_playback(&self) {
        let mut playback = self.active_playback.write();
        *playback = None;
    }

    /// Clear active playback if it belongs to the specified list
    pub fn clear_active_playback_for_list(&self, list_id: &ListContextId) {
        let mut playback = self.active_playback.write();
        if let Some(ref p) = *playback {
            if &p.list_id == list_id {
                *playback = None;
            }
        }
    }
    /// Update list context pagination state after load_more
    /// This only updates page and has_next, does not modify items or version
    pub fn update_list_context_pagination(&self, list_id: &ListContextId, page: usize, has_next: bool) {
        let mut contexts = self.list_contexts.write();
        if let Some(context) = contexts.get_mut(list_id) {
            context.page = page;
            context.has_next = has_next;
        }
    }

    /// Extend list context items after load_more
    /// This appends new items and updates pagination state, validating version
    /// Returns true if successful, false if version mismatch
    pub fn extend_list_context_items(
        &self,
        list_id: &ListContextId,
        expected_version: u64,
        new_items: Vec<Video>,
        page: usize,
        has_next: bool,
    ) -> bool {
        let mut contexts = self.list_contexts.write();
        if let Some(context) = contexts.get_mut(list_id) {
            // Verify version hasn't changed
            if context.version != expected_version {
                println!("[extend_list_context_items] Version mismatch: expected={}, actual={}", expected_version, context.version);
                return false;
            }
            context.items.extend(new_items);
            context.page = page;
            context.has_next = has_next;
            return true;
        }
        false
    }

    /// Finalize a list context after search completes
    /// This updates items and browsing state without incrementing version
    /// The version must match the reserved version from reserve_list_context_version
    pub fn finalize_list_context_search(
        &self,
        list_id: ListContextId,
        reserved_version: u64,
        items: Vec<Video>,
        page: usize,
        page_size: usize,
        has_next: bool,
        total_count: usize,
        query: String,
        sort: Option<crate::models::SortConfig>,
        filters: Option<crate::models::Filters>,
        exclude_watched: bool,
        formula_filter: Option<crate::models::FormulaFilter>,
    ) -> bool {
        let mut contexts = self.list_contexts.write();
        if let Some(context) = contexts.get_mut(&list_id) {
            // Verify version hasn't changed
            if context.version != reserved_version {
                println!("[finalize_list_context_search] Version mismatch: expected={}, actual={}", reserved_version, context.version);
                return false;
            }
            context.items = items;
            context.page = page;
            context.page_size = page_size;
            context.has_next = has_next;
            context.total_count = total_count;
            context.query = query;
            context.sort = sort;
            context.filters = filters;
            context.exclude_watched = exclude_watched;
            context.formula_filter = formula_filter;
            return true;
        }
        false
    }

    /// Reserve a new version for a list context at the start of a search
    /// This invalidates any in-flight load_more requests
    /// Returns the new reserved version
    pub fn reserve_list_context_version(&self, list_id: &ListContextId) -> u64 {
        let mut contexts = self.list_contexts.write();
        let context = contexts.entry(list_id.clone()).or_default();
        context.version += 1;
        context.items.clear();
        context.page = 1;
        context.has_next = false;
        context.version
    }



    /// Update active playback index
    pub fn set_active_playback_index(&self, index: usize) {
        let mut playback = self.active_playback.write();
        if let Some(ref mut p) = *playback {
            p.current_index = index;
        }
    }

    /// Get the current list context version
    pub fn get_list_context_version(&self, list_id: &ListContextId) -> u64 {
        let contexts = self.list_contexts.read();
        contexts.get(list_id).map(|c| c.version).unwrap_or(1)
    }

    /// Get list context items
    pub fn get_list_context_items(&self, list_id: &ListContextId) -> Vec<Video> {
        let contexts = self.list_contexts.read();
        contexts.get(list_id).map(|c| c.items.clone()).unwrap_or_default()
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
