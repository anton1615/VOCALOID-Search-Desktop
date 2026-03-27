use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: Option<String>,
    pub user_nickname: Option<String>,
    pub user_icon_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub video_id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub watched_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchLaterEntry {
    pub video_id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub added_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum PlaylistType {
    #[default]
    Search,
    History,
    WatchLater,
}

/// Unique identifier for a list context.
/// Supports built-in lists (Search, History, WatchLater) and future custom lists.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ListContextId {
    Search,
    History,
    WatchLater,
    Custom(String),
}

impl From<PlaylistType> for ListContextId {
    fn from(playlist_type: PlaylistType) -> Self {
        match playlist_type {
            PlaylistType::Search => ListContextId::Search,
            PlaylistType::History => ListContextId::History,
            PlaylistType::WatchLater => ListContextId::WatchLater,
        }
    }
}

impl From<&ListContextId> for PlaylistType {
    fn from(id: &ListContextId) -> Self {
        match id {
            ListContextId::Search => PlaylistType::Search,
            ListContextId::History => PlaylistType::History,
            ListContextId::WatchLater => PlaylistType::WatchLater,
            ListContextId::Custom(_) => PlaylistType::Search, // Default fallback for custom lists
        }
    }
}

/// A browseable list context that stores its own browsing state, loaded items,
/// pagination progress, and version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListContext {
    /// Unique identifier for this list
    pub id: ListContextId,
    /// Loaded video items
    pub items: Vec<Video>,
    /// Current page number (1-indexed)
    pub page: usize,
    /// Items per page
    pub page_size: usize,
    /// Whether more items are available
    pub has_next: bool,
    /// Total count of items (if known)
    pub total_count: usize,
    /// Monotonically increasing version, incremented when result set changes
    pub version: u64,
    /// Search query
    pub query: String,
    /// Sort configuration
    pub sort: Option<SortConfig>,
    /// Filters
    pub filters: Option<Filters>,
    /// Exclude watched
    pub exclude_watched: bool,
    /// Formula filter
    pub formula_filter: Option<FormulaFilter>,
}

impl Default for ListContext {
    fn default() -> Self {
        Self {
            id: ListContextId::Search,
            items: Vec::new(),
            page: 1,
            page_size: 50,
            has_next: false,
            total_count: 0,
            version: 1,
            query: String::new(),
            sort: None,
            filters: None,
            exclude_watched: false,
            formula_filter: None,
        }
    }
}

/// The single active playback reference that points to one list context and one item index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePlayback {
    /// The list context that owns playback
    pub list_id: ListContextId,
    /// The version of the list when playback started
    pub list_version: u64,
    /// Current item index within the list
    pub current_index: usize,
    /// Whether playback is currently active
    pub is_playing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumericFilter {
    pub gte: Option<f64>,
    pub lte: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateFilter {
    pub gte: Option<String>,
    pub lte: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Filters {
    pub view: Option<NumericFilter>,
    pub mylist: Option<NumericFilter>,
    pub comment: Option<NumericFilter>,
    pub like: Option<NumericFilter>,
    pub start_time: Option<DateFilter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortField {
    #[default]
    View,
    Mylist,
    Comment,
    Like,
    StartTime,
    Custom,
    WatchedAt,
    AddedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    #[default]
    Desc,
}

impl SortDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }
}

impl From<SortDirection> for String {
    fn from(dir: SortDirection) -> Self {
        dir.as_str().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaFilter {
    pub view_weight: f64,
    pub mylist_weight: f64,
    pub comment_weight: f64,
    pub like_weight: f64,
    pub min_score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SortConfig {
    #[serde(default)]
    pub by: SortField,
    #[serde(default)]
    pub direction: SortDirection,
    pub weights: Option<SortWeights>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub results: Vec<Video>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub results: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WatchLaterState {
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub total_count: usize,
    pub sort_direction: String,
    pub search_query: String,
    pub version: u64,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_videos: usize,
    pub last_update: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreshnessCheck {
    pub is_fresh: bool,
    pub local_last_update: Option<String>,
    pub api_last_update: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageInfo {
    pub data_directory: String,
    pub database_size_kb: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncPreflightEstimate {
    pub estimated_video_count: Option<usize>,
    pub estimated_database_size_kb: Option<u64>,
    pub free_space_kb: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScraperProgress {
    pub is_running: bool,
    pub videos_fetched: usize,
    pub total_expected: Option<usize>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaylistState {
    pub playlist_type: PlaylistType,
    pub results: Vec<Video>,
    pub index: Option<usize>,
    pub current_video_id: Option<String>,
    pub has_next: bool,
    pub pip_active: bool,
    pub playlist_version: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub version: u64,
    pub results: Vec<Video>,
    pub loading: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoSelectedPayload {
    pub video: Video,
    pub index: usize,
    pub has_next: bool,
    pub playlist_type: PlaylistType,
    pub playlist_version: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackVideoUpdatedPayload {
    pub list_id: ListContextId,
    pub playlist_type: PlaylistType,
    pub playlist_version: u64,
    pub index: usize,
    pub video: Video,
}

impl PlaybackVideoUpdatedPayload {
    pub fn new(list_id: ListContextId, playlist_version: u64, index: usize, video: Video) -> Self {
        let playlist_type = PlaylistType::from(&list_id);
        Self {
            list_id,
            playlist_type,
            playlist_version,
            index,
            video,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Search playback snapshot metadata that binds to a specific Search session.
/// When active, Search pagination uses the frozen watched boundary instead of live history state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchPlaybackSnapshot {
    /// The Search list context this snapshot belongs to
    pub list_version: u64,
    /// Frozen watched boundary: the max first_watched_seq when playback started
    /// Search pagination will exclude only videos with first_watched_seq <= this value
    pub frozen_watched_boundary_seq: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct SnapshotVideo {
    pub contentId: String,
    pub title: String,
    pub thumbnailUrl: serde_json::Value,
    #[serde(default, deserialize_with = "deserialize_optional_i64_flexible")]
    pub viewCounter: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_flexible")]
    pub commentCounter: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_flexible")]
    pub mylistCounter: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_flexible")]
    pub likeCounter: Option<i64>,
    pub startTime: Option<String>,
    pub tags: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_flexible")]
    pub lengthSeconds: Option<i64>,
    pub genre: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "deserialize_user_id")]
    pub userId: Option<String>,
}

fn deserialize_optional_i64_flexible<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct OptionalI64Visitor;

    impl<'de> Visitor<'de> for OptionalI64Visitor {
        type Value = Option<i64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer, numeric string, or null")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            i64::try_from(value)
                .map(Some)
                .map_err(|_| de::Error::custom("u64 value does not fit into i64"))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let trimmed = value.trim();

            if trimmed.is_empty() {
                return Ok(None);
            }

            trimmed
                .parse::<i64>()
                .map(Some)
                .map_err(|_| de::Error::custom(format!("invalid numeric string: {trimmed}")))
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

    deserializer.deserialize_any(OptionalI64Visitor)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_field_serializes_to_lowercase() {
        assert_eq!(serde_json::to_string(&SortField::View).unwrap(), "\"view\"");
        assert_eq!(
            serde_json::to_string(&SortField::Mylist).unwrap(),
            "\"mylist\""
        );
        assert_eq!(
            serde_json::to_string(&SortField::Comment).unwrap(),
            "\"comment\""
        );
        assert_eq!(serde_json::to_string(&SortField::Like).unwrap(), "\"like\"");
        assert_eq!(
            serde_json::to_string(&SortField::StartTime).unwrap(),
            "\"starttime\""
        );
        assert_eq!(
            serde_json::to_string(&SortField::Custom).unwrap(),
            "\"custom\""
        );
    }

    #[test]
    fn sort_field_deserializes_from_lowercase_string() {
        assert_eq!(
            serde_json::from_str::<SortField>("\"view\"").unwrap(),
            SortField::View
        );
        assert_eq!(
            serde_json::from_str::<SortField>("\"mylist\"").unwrap(),
            SortField::Mylist
        );
        assert_eq!(
            serde_json::from_str::<SortField>("\"comment\"").unwrap(),
            SortField::Comment
        );
        assert_eq!(
            serde_json::from_str::<SortField>("\"like\"").unwrap(),
            SortField::Like
        );
        assert_eq!(
            serde_json::from_str::<SortField>("\"starttime\"").unwrap(),
            SortField::StartTime
        );
        assert_eq!(
            serde_json::from_str::<SortField>("\"custom\"").unwrap(),
            SortField::Custom
        );
    }

    #[test]
    fn sort_direction_serializes_to_lowercase() {
        assert_eq!(
            serde_json::to_string(&SortDirection::Asc).unwrap(),
            "\"asc\""
        );
        assert_eq!(
            serde_json::to_string(&SortDirection::Desc).unwrap(),
            "\"desc\""
        );
    }

    #[test]
    fn sort_direction_deserializes_from_lowercase_string() {
        assert_eq!(
            serde_json::from_str::<SortDirection>("\"asc\"").unwrap(),
            SortDirection::Asc
        );
        assert_eq!(
            serde_json::from_str::<SortDirection>("\"desc\"").unwrap(),
            SortDirection::Desc
        );
    }

    #[test]
    fn invalid_sort_field_rejected() {
        let result = serde_json::from_str::<SortField>("\"invalid\"");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_sort_direction_rejected() {
        let result = serde_json::from_str::<SortDirection>("\"invalid\"");
        assert!(result.is_err());
    }

    #[test]
    fn playback_video_updated_payload_serializes_identity_and_list_context() {
        let payload = PlaybackVideoUpdatedPayload {
            list_id: ListContextId::History,
            playlist_type: PlaylistType::History,
            playlist_version: 7,
            index: 2,
            video: sample_video("sm9"),
        };

        let json = serde_json::to_value(&payload).unwrap();

        assert_eq!(json["list_id"], serde_json::json!("History"));
        assert_eq!(json["playlist_type"], serde_json::json!("History"));
        assert_eq!(json["playlist_version"], serde_json::json!(7));
        assert_eq!(json["index"], serde_json::json!(2));
        assert_eq!(json["video"]["id"], serde_json::json!("sm9"));
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
