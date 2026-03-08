use crate::database::Database;
use crate::models::{HistoryState, PlaylistType, ScraperConfig, ScraperProgress, SearchState, Video, WatchLaterState};
use async_channel::Sender;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub db: Database,
    pub current_video: Arc<RwLock<Option<Video>>>,
    pub pip_active: Arc<RwLock<bool>>,
    pub playlist_type: Arc<RwLock<PlaylistType>>,
    pub search_results: Arc<RwLock<Vec<Video>>>,
    pub history_results: Arc<RwLock<Vec<Video>>>,
    pub watch_later_results: Arc<RwLock<Vec<Video>>>,
    pub playlist_index: Arc<RwLock<usize>>,
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
            playlist_index: Arc::new(RwLock::new(0)),
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
