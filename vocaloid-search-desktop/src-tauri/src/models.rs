use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: Option<String>,
    pub user_nickname: Option<String>,
    pub user_icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub video_id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub watched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchLaterEntry {
    pub video_id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub added_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlaylistType {
    Search,
    History,
    WatchLater,
}

impl Default for PlaylistType {
    fn default() -> Self {
        PlaylistType::Search
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub watch_url: Option<String>,
    pub view_count: i64,
    pub comment_count: i64,
    pub mylist_count: i64,
    pub like_count: i64,
    pub start_time: Option<String>,
    pub tags: Vec<String>,
    pub duration: Option<i64>,
    pub uploader_id: Option<String>,
    pub uploader_name: Option<String>,
    pub description: Option<String>,
    pub is_watched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericFilter {
    pub gte: Option<f64>,
    pub lte: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFilter {
    pub gte: Option<String>,
    pub lte: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Filters {
    pub view: Option<NumericFilter>,
    pub mylist: Option<NumericFilter>,
    pub comment: Option<NumericFilter>,
    pub like: Option<NumericFilter>,
    pub start_time: Option<DateFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortWeights {
    #[serde(default = "default_weight")]
    pub view: f64,
    #[serde(default = "default_weight")]
    pub mylist: f64,
    #[serde(default = "default_weight")]
    pub comment: f64,
    #[serde(default = "default_weight")]
    pub like: f64,
}

fn default_weight() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaFilter {
    pub view_weight: f64,
    pub mylist_weight: f64,
    pub comment_weight: f64,
    pub like_weight: f64,
    pub min_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortConfig {
    pub by: String,
    pub direction: String,
    pub weights: Option<SortWeights>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    #[serde(default)]
    pub query: String,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub exclude_watched: bool,
    pub filters: Option<Filters>,
    pub sort: Option<SortConfig>,
    pub formula_filter: Option<FormulaFilter>,
}

fn default_page() -> usize {
    1
}
fn default_page_size() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub results: Vec<Video>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub results: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchLaterResponse {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub results: Vec<WatchLaterEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryState {
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub total_count: usize,
    pub sort_direction: String,
    pub search_query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WatchLaterState {
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub total_count: usize,
    pub sort_direction: String,
    pub search_query: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScraperConfig {
    #[serde(default = "default_query")]
    pub query: String,
    #[serde(default = "default_max_age")]
    pub max_age_days: Option<i64>,
    #[serde(default = "default_targets")]
    pub targets: String,
    #[serde(default = "default_category")]
    pub category_filter: Option<String>,
}

fn default_query() -> String {
    "VOCALOID".to_string()
}
fn default_max_age() -> Option<i64> {
    Some(365)
}
fn default_targets() -> String {
    "tags".to_string()
}
fn default_category() -> Option<String> {
    Some("MUSIC".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_videos: usize,
    pub last_update: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessCheck {
    pub is_fresh: bool,
    pub local_last_update: Option<String>,
    pub api_last_update: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperProgress {
    pub is_running: bool,
    pub videos_fetched: usize,
    pub total_expected: Option<usize>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 1200,
            height: 800,
            maximized: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistState {
    pub playlist_type: PlaylistType,
    pub results: Vec<Video>,
    pub index: usize,
    pub has_next: bool,
    pub pip_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSettings {
    pub auto_play: bool,
    pub auto_skip: bool,
    pub skip_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchState {
    pub query: String,
    pub exclude_watched: bool,
    pub filters: Option<Filters>,
    pub sort: Option<SortConfig>,
    pub formula_filter: Option<FormulaFilter>,
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSelectedPayload {
    pub video: Video,
    pub index: usize,
    pub has_next: bool,
    pub playlist_type: PlaylistType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipWindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for PipWindowState {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 450,
            height: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct SnapshotVideo {
    pub contentId: String,
    pub title: String,
    pub thumbnailUrl: serde_json::Value,
    pub viewCounter: Option<i64>,
    pub commentCounter: Option<i64>,
    pub mylistCounter: Option<i64>,
    pub likeCounter: Option<i64>,
    pub startTime: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub lengthSeconds: Option<i64>,
    pub genre: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "deserialize_user_id")]
    pub userId: Option<String>,
}

fn deserialize_user_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct UserIdVisitor;

    impl<'de> Visitor<'de> for UserIdVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string, integer, or null")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(UserIdVisitor)
}
