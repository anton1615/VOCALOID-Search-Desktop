use crate::models::ScraperConfig;
use chrono::Utc;
use fs2::available_space;
use reqwest::Client;
use serde_json::Value;
use std::path::Path;

const FALLBACK_BYTES_PER_VIDEO: f64 = 1024.0;
const SNAPSHOT_API: &str = "https://snapshot.search.nicovideo.jp/api/v2/snapshot/video/contents/search";
const CATEGORY_TO_GENRE: &[(&str, &str)] = &[
    ("MUSIC", "音楽・サウンド"),
    ("GAME", "ゲーム"),
    ("ANIME", "アニメ"),
    ("ENTERTAINMENT", "エンターテイメント"),
    ("DANCE", "ダンス"),
    ("ANIMAL", "動物"),
    ("NATURE", "自然"),
    ("COOKING", "料理"),
    ("TRAVEL", "旅行・アウトドア"),
    ("VEHICLE", "乗り物"),
    ("SPORTS", "スポーツ"),
    ("SOCIAL", "社会・政治・時事"),
    ("TECHNICAL", "技術・工作"),
    ("LECTURE", "解説・講座"),
    ("OTHER", "その他"),
    ("RADIO", "ラジオ"),
];

pub fn estimate_database_size_kb(
    estimated_video_count: Option<usize>,
    current_file_size_kb: Option<u64>,
    live_bytes_per_row: Option<f64>,
) -> Option<u64> {
    let count = estimated_video_count? as u64;

    let bytes_per_row = live_bytes_per_row.unwrap_or(FALLBACK_BYTES_PER_VIDEO);
    let required_kb = (count as f64 * bytes_per_row / 1024.0).ceil() as u64;

    // SQLite never shrinks a database file in place: auto_vacuum is off and the
    // sync replaces rows with DELETE + INSERT, which recycles freed pages but
    // keeps the file at its high-water mark. The synced file therefore cannot be
    // smaller than what is already on disk.
    Some(match current_file_size_kb {
        Some(current) => required_kb.max(current),
        None => required_kb,
    })
}

fn build_estimate_query_params(config: &ScraperConfig) -> Vec<(String, String)> {
    let mut params = vec![
        ("q".to_string(), config.query.clone()),
        ("targets".to_string(), config.targets.clone()),
        ("fields".to_string(), "contentId".to_string()),
        ("_sort".to_string(), "-startTime".to_string()),
        ("_limit".to_string(), "0".to_string()),
        ("context".to_string(), "vocaloid-search-desktop".to_string()),
    ];

    if let Some(days) = config.max_age_days {
        if days > 0 {
            let date = Utc::now() - chrono::Duration::days(days);
            params.push((
                "filters[start_time][gte]".to_string(),
                date.format("%Y-%m-%dT%H:%M:%S+09:00").to_string(),
            ));
        }
    }

    if let Some(category) = &config.category_filter {
        let genre = CATEGORY_TO_GENRE
            .iter()
            .find(|(key, _)| *key == category)
            .map(|(_, value)| (*value).to_string())
            .unwrap_or_else(|| category.clone());
        params.push(("filters[genre][0]".to_string(), genre));
    }

    params
}

pub async fn estimate_video_count(config: &ScraperConfig) -> Option<usize> {
    let client = Client::builder()
        .user_agent("vocaloid-search-desktop/1.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .ok()?;

    let params = build_estimate_query_params(config);

    let response = client
        .get(SNAPSHOT_API)
        .query(&params)
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let data: Value = response.json().await.ok()?;
    data.get("meta")?
        .get("totalCount")?
        .as_u64()
        .map(|count| count as usize)
}

pub fn lookup_free_space_kb(path: &Path) -> Option<u64> {
    available_space(path).ok().map(|bytes| bytes / 1024)
}

#[cfg(test)]
mod tests {
    use super::{build_estimate_query_params, estimate_database_size_kb, lookup_free_space_kb};
    use crate::models::ScraperConfig;
    use std::path::Path;

    #[test]
    fn scales_from_live_bytes_per_row() {
        let estimated = estimate_database_size_kb(Some(200_000), Some(4_000), Some(1024.0));

        assert_eq!(estimated, Some(200_000));
    }

    #[test]
    fn never_reports_less_than_the_current_file_size() {
        let estimated = estimate_database_size_kb(Some(10), Some(50_000), Some(1024.0));

        assert_eq!(estimated, Some(50_000));
    }

    #[test]
    fn falls_back_to_default_bytes_per_row_when_database_is_empty() {
        let estimated = estimate_database_size_kb(Some(10), None, None);

        assert_eq!(estimated, Some(10));
    }

    #[test]
    fn returns_none_when_video_count_estimate_is_unavailable() {
        let estimated = estimate_database_size_kb(None, Some(4_000), Some(1024.0));

        assert_eq!(estimated, None);
    }

    #[test]
    fn builds_snapshot_estimate_query_from_scraper_config() {
        let config = ScraperConfig {
            query: "VOCALOID".to_string(),
            max_age_days: Some(30),
            targets: "tags,title".to_string(),
            category_filter: Some("MUSIC".to_string()),
        };

        let params = build_estimate_query_params(&config);

        assert!(params.iter().any(|(key, value)| key == "q" && value == "VOCALOID"));
        assert!(params.iter().any(|(key, value)| key == "targets" && value == "tags,title"));
        assert!(params.iter().any(|(key, value)| key == "_limit" && value == "0"));
        assert!(params.iter().any(|(key, value)| key == "_sort" && value == "-startTime"));
        assert!(params.iter().any(|(key, _)| key == "filters[start_time][gte]"));
        assert!(params.iter().any(|(key, value)| key == "filters[genre][0]" && value == "音楽・サウンド"));
    }

    #[test]
    fn treats_zero_max_age_as_unlimited() {
        let config = ScraperConfig {
            query: "VOCALOID".to_string(),
            max_age_days: Some(0),
            targets: "tags".to_string(),
            category_filter: Some("MUSIC".to_string()),
        };

        let params = build_estimate_query_params(&config);

        assert!(!params.iter().any(|(key, _)| key == "filters[start_time][gte]"));
    }

    #[test]
    fn skips_genre_filter_when_category_is_none() {
        let config = ScraperConfig {
            query: "VOCALOID".to_string(),
            max_age_days: Some(30),
            targets: "tags".to_string(),
            category_filter: None,
        };

        let params = build_estimate_query_params(&config);

        assert!(!params.iter().any(|(key, _)| key == "filters[genre][0]"));
    }

    #[test]
    fn looks_up_free_space_for_existing_path() {
        let free_space = lookup_free_space_kb(Path::new("."));

        assert!(free_space.is_some());
    }
}
