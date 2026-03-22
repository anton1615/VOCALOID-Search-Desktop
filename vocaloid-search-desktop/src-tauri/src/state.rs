use crate::database::Database;
use crate::models::{
    ActivePlayback, HistoryState, ListContext, ListContextId, ScraperConfig, ScraperProgress,
    SearchPlaybackSnapshot, SearchState, Video, WatchLaterState,
};
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
    requested_playlist_type == crate::models::PlaylistType::Search
        && current_version == expected_version
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
    pub browsing_list: Arc<RwLock<ListContextId>>,
    // New list-context model
    pub list_contexts: Arc<RwLock<HashMap<ListContextId, ListContext>>>,
    pub active_playback: Arc<RwLock<Option<ActivePlayback>>>,
    /// Search playback snapshot for frozen watched boundary
    /// Only valid when active_playback.list_id == ListContextId::Search
    pub search_playback_snapshot: Arc<RwLock<Option<SearchPlaybackSnapshot>>>,
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
            browsing_list: Arc::new(RwLock::new(ListContextId::Search)),
            list_contexts: Arc::new(RwLock::new(HashMap::new())),
            active_playback: Arc::new(RwLock::new(None)),
            search_playback_snapshot: Arc::new(RwLock::new(None)),
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

    fn clear_active_playback_session(&self) -> Option<ListContextId> {
        let mut playback = self.active_playback.write();
        let cleared_list_id = playback.take().map(|playback| playback.list_id);
        drop(playback);

        if cleared_list_id.is_none() {
            return None;
        }

        let mut snapshot = self.search_playback_snapshot.write();
        *snapshot = None;
        drop(snapshot);

        cleared_list_id
    }

    /// Clear active playback.
    /// Returns true when playback was actually cleared.
    pub fn clear_active_playback(&self) -> bool {
        self.clear_active_playback_session().is_some()
    }

    /// Clear active playback and return the cleared playback list when present.
    pub fn clear_active_playback_with_list(&self) -> Option<ListContextId> {
        self.clear_active_playback_session()
    }

    /// Clear active playback if it belongs to the specified list.
    /// Returns true when playback was actually cleared.
    pub fn clear_active_playback_for_list(&self, list_id: &ListContextId) -> bool {
        let mut playback = self.active_playback.write();
        if let Some(ref p) = *playback {
            if &p.list_id == list_id {
                *playback = None;
                // Also clear Search playback snapshot if clearing Search playback
                if *list_id == ListContextId::Search {
                    let mut snapshot = self.search_playback_snapshot.write();
                    *snapshot = None;
                }
                return true;
            }
        }
        false
    }

    /// Create or reuse Search playback snapshot for the current Search session
    /// Returns true if a new snapshot was created, false if reusing existing
    pub fn create_or_reuse_search_playback_snapshot(
        &self,
        list_version: u64,
        max_watched_seq: i64,
    ) -> bool {
        let mut snapshot = self.search_playback_snapshot.write();
        match *snapshot {
            Some(ref existing) if existing.list_version == list_version => {
                // Reuse existing snapshot for same version
                false
            }
            _ => {
                // Create new snapshot
                *snapshot = Some(SearchPlaybackSnapshot {
                    list_version,
                    frozen_watched_boundary_seq: max_watched_seq,
                });
                true
            }
        }
    }

    /// Get the current Search playback snapshot if valid for the given version
    pub fn get_search_playback_snapshot(
        &self,
        list_version: u64,
    ) -> Option<SearchPlaybackSnapshot> {
        let snapshot = self.search_playback_snapshot.read();
        match *snapshot {
            Some(ref s) if s.list_version == list_version => Some(s.clone()),
            _ => None,
        }
    }

    /// Invalidate the Search playback snapshot (called when Search conditions change)
    pub fn invalidate_search_playback_snapshot(&self) {
        let mut snapshot = self.search_playback_snapshot.write();
        *snapshot = None;
    }
    /// Update list context pagination state after load_more
    /// This only updates page and has_next, does not modify items or version
    pub fn update_list_context_pagination(
        &self,
        list_id: &ListContextId,
        page: usize,
        has_next: bool,
    ) {
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
    /// This invalidates any in-flight load_more requests and atomically updates browsing params
    /// Returns the new reserved version
    pub fn reserve_list_context_version(
        &self,
        list_id: &ListContextId,
        query: String,
        sort: Option<crate::models::SortConfig>,
        filters: Option<crate::models::Filters>,
        exclude_watched: bool,
        formula_filter: Option<crate::models::FormulaFilter>,
    ) -> u64 {
        let mut contexts = self.list_contexts.write();
        let context = contexts.entry(list_id.clone()).or_default();
        context.version += 1;
        context.items.clear();
        context.page = 1;
        context.has_next = false;
        // Atomically update browsing params to prevent race with load_more
        context.query = query;
        context.sort = sort;
        context.filters = filters;
        context.exclude_watched = exclude_watched;
        context.formula_filter = formula_filter;
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
        contexts
            .get(list_id)
            .map(|c| c.items.clone())
            .unwrap_or_default()
    }

    /// Update a single video in a list context
    /// Returns true if successful, false if index out of bounds or context not found
    pub fn update_list_context_item(
        &self,
        list_id: &ListContextId,
        index: usize,
        video: Video,
    ) -> bool {
        let mut contexts = self.list_contexts.write();
        if let Some(context) = contexts.get_mut(list_id) {
            if index < context.items.len() {
                context.items[index] = video;
                return true;
            }
        }
        false
    }

    pub fn matches_active_playback_metadata_update(
        &self,
        list_id: &ListContextId,
        playlist_version: u64,
        index: usize,
        video: &Video,
    ) -> bool {
        let playback = self.active_playback.read();
        let Some(active_playback) = playback.as_ref() else {
            return false;
        };

        if &active_playback.list_id != list_id
            || active_playback.list_version != playlist_version
            || active_playback.current_index != index
        {
            return false;
        }
        drop(playback);

        let contexts = self.list_contexts.read();
        let Some(context) = contexts.get(list_id) else {
            return false;
        };

        if context.version != playlist_version {
            return false;
        }

        context
            .items
            .get(index)
            .map(|current_video| current_video.id == video.id)
            .unwrap_or(false)
    }

    /// Set the currently visible browsing list without changing active playback.
    pub fn set_browsing_list(&self, list_id: ListContextId) {
        let mut browsing = self.browsing_list.write();
        *browsing = list_id;
    }

    pub fn get_browsing_list(&self) -> ListContextId {
        self.browsing_list.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        should_accept_list_load_more, should_bump_list_version,
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

    #[test]
    fn switching_browsing_list_preserves_existing_active_playback_binding() {
        let test = TestAppState::new();
        let search_version = test.state.get_list_context_version(&ListContextId::Search);
        test.state
            .set_active_playback(ListContextId::Search, search_version, 2);

        test.state.set_browsing_list(ListContextId::History);

        let playback = test.state.active_playback.read().clone();
        assert_eq!(
            playback.as_ref().map(|p| &p.list_id),
            Some(&ListContextId::Search),
            "switching visible pages must not rewrite the playback-bound list"
        );
        assert_eq!(
            playback.as_ref().map(|p| p.current_index),
            Some(2),
            "switching visible pages must preserve the active playback index"
        );
        assert_eq!(
            playback.as_ref().map(|p| p.list_version),
            Some(search_version),
            "switching visible pages must preserve the active playback version"
        );
    }

    #[test]
    fn switching_browsing_list_without_playback_does_not_create_active_playback() {
        let test = TestAppState::new();

        test.state.set_browsing_list(ListContextId::History);

        let playback = test.state.active_playback.read().clone();
        assert!(
            playback.is_none(),
            "switching visible pages without an explicit selection must not create active playback"
        );
    }

    #[test]
    fn clear_active_playback_for_list_reports_false_for_non_active_list() {
        let test = TestAppState::new();
        let search_version = test.state.get_list_context_version(&ListContextId::Search);
        test.state
            .set_active_playback(ListContextId::Search, search_version, 1);

        let cleared = test
            .state
            .clear_active_playback_for_list(&ListContextId::History);

        assert!(
            !cleared,
            "non-active list refreshes must not report playback cleared"
        );
        let playback = test.state.active_playback.read().clone();
        assert_eq!(
            playback.as_ref().map(|p| &p.list_id),
            Some(&ListContextId::Search),
            "non-active list refreshes must preserve active playback"
        );
    }

    #[test]
    fn clear_active_playback_for_list_reports_true_for_active_list() {
        let test = TestAppState::new();
        let history_version = test.state.get_list_context_version(&ListContextId::History);
        test.state
            .set_active_playback(ListContextId::History, history_version, 0);

        let cleared = test
            .state
            .clear_active_playback_for_list(&ListContextId::History);

        assert!(
            cleared,
            "active list invalidation must report playback cleared"
        );
        assert!(
            test.state.active_playback.read().is_none(),
            "active list invalidation must clear playback"
        );
    }

    #[test]
    fn sync_route_reset_clears_playback_without_touching_browsing_contexts() {
        let test = TestAppState::new();

        test.state.update_list_context(
            ListContextId::Search,
            vec![sample_video("sm9")],
            1,
            50,
            true,
            99,
            "miku".to_string(),
            None,
            None,
            true,
            None,
        );
        test.state
            .update_list_context_pagination(&ListContextId::Search, 3, true);
        test.state.set_browsing_list(ListContextId::History);
        {
            let mut search_state = test.state.search_state.write();
            search_state.query = "miku".to_string();
            search_state.page = 3;
            search_state.has_next = true;
            search_state.total_count = 99;
            search_state.results = vec![sample_video("sm9")];
        }
        test.state.history_state.write().page = 2;
        test.state.watch_later_state.write().page = 4;

        let search_version = test.state.get_list_context_version(&ListContextId::Search);
        test.state
            .create_or_reuse_search_playback_snapshot(search_version, 42);
        test.state
            .set_active_playback(ListContextId::Search, search_version, 0);

        let cleared = test.state.clear_active_playback();

        assert!(
            cleared,
            "sync-route reset should report when playback was terminated"
        );
        assert!(test.state.active_playback.read().is_none());
        assert!(test.state.search_playback_snapshot.read().is_none());
        assert_eq!(test.state.get_browsing_list(), ListContextId::History);

        let search_context = test.state.get_list_context(&ListContextId::Search).unwrap();
        assert_eq!(search_context.query, "miku");
        assert_eq!(search_context.page, 3);
        assert_eq!(search_context.total_count, 99);
        assert_eq!(search_context.items.len(), 1);

        assert_eq!(test.state.search_state.read().query, "miku");
        assert_eq!(test.state.search_state.read().page, 3);
        assert_eq!(test.state.history_state.read().page, 2);
        assert_eq!(test.state.watch_later_state.read().page, 4);
    }

    #[test]
    fn sync_route_reset_without_active_playback_still_invalidates_search_snapshot() {
        let test = TestAppState::new();

        test.state.update_list_context(
            ListContextId::Search,
            vec![sample_video("sm9")],
            1,
            50,
            true,
            1,
            "miku".to_string(),
            None,
            None,
            false,
            None,
        );
        let search_version = test.state.get_list_context_version(&ListContextId::Search);
        test.state
            .create_or_reuse_search_playback_snapshot(search_version, 7);
        test.state.set_active_playback(ListContextId::History, 1, 0);

        let history_cleared = test
            .state
            .clear_active_playback_for_list(&ListContextId::History);

        assert!(history_cleared);
        assert!(test.state.active_playback.read().is_none());
        assert!(
            test.state.search_playback_snapshot.read().is_some(),
            "search snapshot can remain when a non-search playback session is cleared"
        );

        test.state.invalidate_search_playback_snapshot();

        assert!(
            test.state.search_playback_snapshot.read().is_none(),
            "sync-route reset must invalidate any remaining search snapshot even without active playback"
        );
    }

    // Search playback snapshot tests
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestAppState {
        state: AppState,
        _root: std::path::PathBuf,
    }

    impl TestAppState {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("snapshot-test-{}", unique));
            std::fs::create_dir_all(&root).unwrap();
            let videos_path = root.join("videos.db");
            let user_data_path = root.join("user_data.db");
            crate::database::init_db(&videos_path, &user_data_path).unwrap();
            let state = AppState::new(videos_path, user_data_path);
            Self { state, _root: root }
        }
    }

    impl Drop for TestAppState {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self._root);
        }
    }

    #[test]
    fn creates_new_search_playback_snapshot_on_first_playback() {
        let test = TestAppState::new();
        test.state.update_list_context(
            ListContextId::Search,
            vec![],
            1,
            50,
            false,
            0,
            String::new(),
            None,
            None,
            true,
            None,
        );
        let version = test.state.get_list_context_version(&ListContextId::Search);

        let created = test
            .state
            .create_or_reuse_search_playback_snapshot(version, 42);
        assert!(created, "should return true when creating new snapshot");

        let snapshot = test.state.get_search_playback_snapshot(version);
        assert!(
            snapshot.is_some(),
            "snapshot should exist for matching version"
        );
        let s = snapshot.unwrap();
        assert_eq!(s.frozen_watched_boundary_seq, 42);
        assert_eq!(s.list_version, version);
    }

    #[test]
    fn reuses_existing_snapshot_for_same_search_version() {
        let test = TestAppState::new();
        test.state.update_list_context(
            ListContextId::Search,
            vec![],
            1,
            50,
            false,
            0,
            String::new(),
            None,
            None,
            true,
            None,
        );
        let version = test.state.get_list_context_version(&ListContextId::Search);

        test.state
            .create_or_reuse_search_playback_snapshot(version, 10);
        let reused = test
            .state
            .create_or_reuse_search_playback_snapshot(version, 20);
        assert!(
            !reused,
            "should return false when reusing existing snapshot"
        );

        let snapshot = test.state.get_search_playback_snapshot(version).unwrap();
        assert_eq!(
            snapshot.frozen_watched_boundary_seq, 10,
            "should keep original frozen boundary"
        );
    }

    #[test]
    fn invalidates_snapshot_when_search_conditions_change() {
        let test = TestAppState::new();
        test.state.update_list_context(
            ListContextId::Search,
            vec![],
            1,
            50,
            false,
            0,
            String::new(),
            None,
            None,
            true,
            None,
        );
        let version = test.state.get_list_context_version(&ListContextId::Search);
        test.state
            .create_or_reuse_search_playback_snapshot(version, 5);

        test.state.invalidate_search_playback_snapshot();

        let snapshot = test.state.get_search_playback_snapshot(version);
        assert!(snapshot.is_none(), "snapshot should be invalidated");
    }

    #[test]
    fn clears_snapshot_when_active_playback_is_cleared_for_search() {
        let test = TestAppState::new();
        test.state.update_list_context(
            ListContextId::Search,
            vec![],
            1,
            50,
            false,
            0,
            String::new(),
            None,
            None,
            true,
            None,
        );
        let version = test.state.get_list_context_version(&ListContextId::Search);
        test.state
            .create_or_reuse_search_playback_snapshot(version, 5);
        test.state
            .set_active_playback(ListContextId::Search, version, 0);

        test.state
            .clear_active_playback_for_list(&ListContextId::Search);

        let snapshot = test.state.search_playback_snapshot.read();
        assert!(
            snapshot.is_none(),
            "snapshot should be cleared when Search playback is cleared"
        );
    }

    #[test]
    fn does_not_clear_snapshot_when_clearing_other_list_playback() {
        let test = TestAppState::new();
        test.state.update_list_context(
            ListContextId::Search,
            vec![],
            1,
            50,
            false,
            0,
            String::new(),
            None,
            None,
            true,
            None,
        );
        let version = test.state.get_list_context_version(&ListContextId::Search);
        test.state
            .create_or_reuse_search_playback_snapshot(version, 5);
        test.state.set_active_playback(ListContextId::History, 1, 0);

        test.state
            .clear_active_playback_for_list(&ListContextId::History);

        let snapshot = test.state.search_playback_snapshot.read();
        assert!(
            snapshot.is_some(),
            "Search snapshot should remain when History playback is cleared"
        );
    }

    #[test]
    fn playback_metadata_update_matches_active_playback_identity() {
        let test = TestAppState::new();
        let history_items = vec![
            sample_video("sm1"),
            sample_video("sm2"),
            sample_video("sm9"),
        ];
        test.state.update_list_context(
            ListContextId::History,
            history_items,
            1,
            50,
            false,
            3,
            String::new(),
            None,
            None,
            false,
            None,
        );
        let history_version = test.state.get_list_context_version(&ListContextId::History);
        let video = sample_video("sm9");
        test.state
            .set_active_playback(ListContextId::History, history_version, 2);

        assert!(test.state.matches_active_playback_metadata_update(
            &ListContextId::History,
            history_version,
            2,
            &video,
        ));
    }

    #[test]
    fn playback_metadata_update_rejects_stale_identity() {
        let test = TestAppState::new();
        let history_items = vec![
            sample_video("sm1"),
            sample_video("sm2"),
            sample_video("sm9"),
        ];
        test.state.update_list_context(
            ListContextId::History,
            history_items,
            1,
            50,
            false,
            3,
            String::new(),
            None,
            None,
            false,
            None,
        );
        let history_version = test.state.get_list_context_version(&ListContextId::History);
        let video = sample_video("sm9");
        test.state
            .set_active_playback(ListContextId::History, history_version, 2);

        assert!(!test.state.matches_active_playback_metadata_update(
            &ListContextId::History,
            history_version + 1,
            2,
            &video,
        ));
        assert!(!test.state.matches_active_playback_metadata_update(
            &ListContextId::WatchLater,
            history_version,
            2,
            &video,
        ));
        assert!(!test.state.matches_active_playback_metadata_update(
            &ListContextId::History,
            history_version,
            3,
            &video,
        ));
        assert!(!test.state.matches_active_playback_metadata_update(
            &ListContextId::History,
            history_version,
            2,
            &sample_video("sm10"),
        ));
    }

    fn sample_video(id: &str) -> Video {
        Video {
            id: id.to_string(),
            title: format!("title-{id}"),
            thumbnail_url: Some("https://example.com/thumb.jpg".to_string()),
            watch_url: Some(format!("https://www.nicovideo.jp/watch/{id}")),
            view_count: 1,
            comment_count: 2,
            mylist_count: 3,
            like_count: 4,
            start_time: Some("2025-01-01T00:00:00+09:00".to_string()),
            tags: vec!["vocaloid".to_string()],
            duration: Some(123),
            uploader_id: Some("user-1".to_string()),
            uploader_name: Some("miku".to_string()),
            description: Some("desc".to_string()),
            is_watched: false,
        }
    }
}
