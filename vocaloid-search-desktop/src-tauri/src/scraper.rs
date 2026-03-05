use std::collections::HashSet;
use chrono::Utc;
use crate::models::{ScraperConfig, SnapshotVideo};

#[derive(Debug)]
struct CancelledError;

impl std::fmt::Display for CancelledError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cancelled")
    }
}

impl std::error::Error for CancelledError {}

const SNAPSHOT_API: &str = "https://snapshot.search.nicovideo.jp/api/v2/snapshot/video/contents/search";
const MAX_OFFSET: usize = 100000;

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

pub struct Scraper {
    client: reqwest::Client,
    config: ScraperConfig,
    cancel_receiver: Option<async_channel::Receiver<()>>,
}

#[derive(Debug)]
pub struct ScraperResult {
    pub videos: Vec<SnapshotVideo>,
    pub total_fetched: usize,
}

impl Scraper {
    pub fn new(config: ScraperConfig) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("vocaloid-search-desktop/1.0")
                .build()
                .expect("Failed to create HTTP client"),
            config,
            cancel_receiver: None,
        }
    }
    
    pub fn with_cancel(mut self, receiver: async_channel::Receiver<()>) -> Self {
        self.cancel_receiver = Some(receiver);
        self
    }
    
    pub fn is_cancelled(&self) -> bool {
        if let Some(rx) = &self.cancel_receiver {
            rx.try_recv().is_ok()
        } else {
            false
        }
    }
    
    pub async fn fetch_videos<F: Fn(usize, Option<usize>) + Send + Sync + Clone>(
        &self,
        progress_callback: F,
    ) -> Result<ScraperResult, Box<dyn std::error::Error + Send>> {
        let mut all_videos = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut total_fetched = 0;
        let mut last_start_time: Option<String> = None;
        let mut api_reported_total: Option<usize> = None;
        
        let start_after = self.config.max_age_days.map(|days| {
            let date = Utc::now() - chrono::Duration::days(days);
            date.format("%Y-%m-%dT%H:%M:%S+09:00").to_string()
        });
        
        let mut round_num = 1;
        
        loop {
            if self.is_cancelled() {
                println!("Scraper cancelled at round {}", round_num);
                return Err(Box::new(CancelledError));
            }
            
            println!("Fetching round {}...", round_num);
            
            let mut params = vec![
                ("q", self.config.query.as_str()),
                ("targets", self.config.targets.as_str()),
                ("fields", "contentId,title,thumbnailUrl,viewCounter,commentCounter,mylistCounter,likeCounter,startTime,tags,lengthSeconds,genre,description,userId"),
                ("_sort", "-startTime"),
                ("_limit", "100"),
                ("context", "vocaloid-search-desktop"),
            ];
            
            if let Some(ref start) = start_after {
                params.push(("filters[start_time][gte]", start.as_str()));
            }
            
            if let Some(ref category) = self.config.category_filter {
                let genre = CATEGORY_TO_GENRE.iter()
                    .find(|(k, _)| *k == category)
                    .map(|(_, v)| *v)
                    .unwrap_or(category.as_str());
                params.push(("filters[genre][0]", genre));
            }
            
            if let Some(ref last) = last_start_time {
                params.push(("filters[start_time][lte]", last.as_str()));
            }
            
            let mut offset = 0;
            let mut round_count = 0;
            let mut round_last_time: Option<String> = None;
            
            while offset < MAX_OFFSET {
                if self.is_cancelled() {
                    println!("Scraper cancelled at offset {}", offset);
                    return Err(Box::new(CancelledError));
                }
                
                let offset_str = offset.to_string();
                let mut request_params = params.clone();
                request_params.push(("_offset", offset_str.as_str()));
                
                // Build URL for logging
                let url = reqwest::Url::parse_with_params(SNAPSHOT_API, &request_params).unwrap();
                println!("  Requesting: offset={}, url={}", offset, url);
                
                let response = match self.client
                    .get(SNAPSHOT_API)
                    .query(&request_params)
                    .timeout(std::time::Duration::from_secs(60))
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        println!("HTTP error: {}", e);
                        break;
                    }
                };
                
                let status = response.status().as_u16();
                println!("  Response status: {}", status);
                
                if !response.status().is_success() {
                    if status == 400 && offset >= MAX_OFFSET - 100 {
                        println!("Reached API offset limit at {}", offset);
                        break;
                    }
                    let body = response.text().await.unwrap_or_default();
                    println!("API error {}: {}", status, body);
                    break;
                }
                
                let data: serde_json::Value = match response.json().await {
                    Ok(d) => d,
                    Err(e) => {
                        println!("JSON parse error: {}", e);
                        break;
                    }
                };
                
                if api_reported_total.is_none() {
                    api_reported_total = data.get("meta")
                        .and_then(|m| m.get("totalCount"))
                        .and_then(|c| c.as_u64())
                        .map(|c| c as usize);
                    if let Some(total) = api_reported_total {
                        println!("  API reports total: {}", total);
                    }
                }
                
                let videos = data.get("data")
                    .and_then(|d| d.as_array())
                    .cloned()
                    .unwrap_or_default();
                
                println!("  Received {} videos in response", videos.len());
                
                if videos.is_empty() {
                    println!("  No more videos from API");
                    break;
                }
                
                let videos_len = videos.len();
                let mut parse_success = 0usize;
                let mut parse_fail = 0usize;
                let mut duplicate_count = 0usize;
                
                for (idx, video_value) in videos.iter().enumerate() {
                    match serde_json::from_value::<SnapshotVideo>(video_value.clone()) {
                        Ok(video) => {
                            parse_success += 1;
                            let video_id = video.contentId.clone();
                            let video_start_time = video.startTime.clone();
                            if !seen_ids.contains(&video_id) {
                                seen_ids.insert(video_id);
                                all_videos.push(video);
                                total_fetched += 1;
                                round_count += 1;
                                round_last_time = video_start_time;
                            } else {
                                duplicate_count += 1;
                            }
                        }
                        Err(e) => {
                            parse_fail += 1;
                            if parse_fail <= 3 {
                                println!("  Parse error #{}: {}", parse_fail, e);
                                if idx == 0 {
                                    println!("  First video JSON: {:?}", video_value);
                                }
                            }
                        }
                    }
                }
                
                println!("  Parsed: {} success, {} fail, {} duplicates", parse_success, parse_fail, duplicate_count);
                
                progress_callback(total_fetched, api_reported_total);
                
                offset += videos_len;
                
                if total_fetched % 10000 == 0 {
                    println!("  Total fetched: {}", total_fetched);
                }
            }
            
            println!("  Round {} complete: {} new videos", round_num, round_count);
            
            if round_last_time.is_none() || round_last_time == last_start_time {
                println!("  No progress, stopping");
                break;
            }
            
            last_start_time = round_last_time;
            round_num += 1;
            println!("  Continuing with start_time <= {:?}", last_start_time);
        }
        
        println!("Fetch complete: {} total videos", total_fetched);
        
        Ok(ScraperResult {
            videos: all_videos,
            total_fetched,
        })
    }
}

pub fn snapshot_to_db_row(video: &SnapshotVideo) -> (String, String, Option<String>, Option<String>, i64, i64, i64, i64, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>, Option<String>, Option<String>) {
    let thumbnail_url = if video.thumbnailUrl.is_object() {
        video.thumbnailUrl.get("large").and_then(|u| u.as_str()).map(|s| s.to_string())
    } else {
        video.thumbnailUrl.as_str().map(|s| s.to_string())
    };
    
    let tags = video.tags.as_ref().map(|t| {
        if t.is_array() {
            t.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default()
        } else {
            t.as_str().unwrap_or("").to_string()
        }
    });
    
    (
        video.contentId.clone(),
        video.title.clone(),
        thumbnail_url,
        Some(format!("https://www.nicovideo.jp/watch/{}", video.contentId)),
        video.viewCounter.unwrap_or(0),
        video.commentCounter.unwrap_or(0),
        video.mylistCounter.unwrap_or(0),
        video.likeCounter.unwrap_or(0),
        video.startTime.clone(),
        tags,
        video.lengthSeconds,
        video.genre.clone(),
        video.description.clone(),
        video.userId.clone(),
        None,
    )
}

pub async fn check_snapshot_api_last_update() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://snapshot.search.nicovideo.jp/api/v2/snapshot/video/contents/search")
        .query(&[
            ("q", "VOCALOID"),
            ("targets", "tags"),
            ("fields", "contentId"),
            ("_limit", "1"),
        ])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    let data: serde_json::Value = response.json().await?;
    
    let last_update = data.get("meta")
        .and_then(|m| m.get("updatedAt"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());
    
    Ok(last_update)
}

pub fn get_daily_update_threshold() -> String {
    use chrono::{Datelike, TimeZone, Utc};
    
    let now = Utc::now();
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let now_jst = now.with_timezone(&jst_offset);
    
    let today_6am_jst = jst_offset.with_ymd_and_hms(now_jst.year(), now_jst.month(), now_jst.day(), 6, 0, 0)
        .single()
        .unwrap_or_else(|| now_jst);
    
    today_6am_jst.format("%Y-%m-%d %H:%M:%S").to_string()
}
