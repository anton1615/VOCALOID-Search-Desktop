use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use crate::models::*;
use crate::scraper::{Scraper, snapshot_to_db_row, check_snapshot_api_last_update};
use crate::state::AppState;
use async_channel;
use quick_xml::Reader;
use quick_xml::events::Event;

#[tauri::command]
pub async fn get_playlist_state(
    state: tauri::State<'_, AppState>,
) -> Result<PlaylistState, String> {
    let results = state.search_results.read().clone();
    let index = *state.playlist_index.read();
    let has_next = index + 1 < results.len();
    let pip_active = *state.pip_active.read();

    Ok(PlaylistState {
        results,
        index,
        has_next,
        pip_active,
    })
}

#[tauri::command]
pub async fn set_playlist_index(
    app: AppHandle,
    index: usize,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    println!("[set_playlist_index] Called with index: {}", index);
    
    let results = state.search_results.read();
    println!("[set_playlist_index] Results length: {}", results.len());
    
    if index >= results.len() {
        println!("[set_playlist_index] ERROR: Index {} >= len {}", index, results.len());
        return Err("Index out of bounds".to_string());
    }

    {
        let mut current_index = state.playlist_index.write();
        *current_index = index;
        println!("[set_playlist_index] Updated playlist_index to: {}", index);
    }

    let video = results[index].clone();
    let has_next = index + 1 < results.len();
    
    println!("[set_playlist_index] Emitting video-selected: video_id={}, index={}, has_next={}", video.id, index, has_next);

    app.emit("video-selected", VideoSelectedPayload {
        video,
        index,
        has_next,
    }).map_err(|e| e.to_string())?;

    println!("[set_playlist_index] Event emitted successfully");
    Ok(())
}

#[tauri::command]
pub async fn get_playback_settings(
    state: tauri::State<'_, AppState>,
) -> Result<PlaybackSettings, String> {
    let auto_play = *state.auto_play.read();
    let auto_skip = *state.auto_skip.read();
    let skip_threshold = *state.skip_threshold.read();
    
    Ok(PlaybackSettings {
        auto_play,
        auto_skip,
        skip_threshold,
    })
}

#[tauri::command]
pub async fn set_playback_settings(
    app: AppHandle,
    settings: PlaybackSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut auto_play = state.auto_play.write();
        *auto_play = settings.auto_play;
    }
    {
        let mut auto_skip = state.auto_skip.write();
        *auto_skip = settings.auto_skip;
    }
    {
        let mut skip_threshold = state.skip_threshold.write();
        *skip_threshold = settings.skip_threshold;
    }
    
    app.emit("playback-settings-changed", &settings).map_err(|e| e.to_string())?;
    
    Ok(())
}


#[tauri::command]
pub async fn get_search_state(
    state: tauri::State<'_, AppState>,
) -> Result<SearchState, String> {
    let search_state = state.search_state.read().clone();
    Ok(search_state)
}

#[tauri::command]
pub async fn set_search_state(
    app: AppHandle,
    search_state: SearchState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut current = state.search_state.write();
        *current = search_state;
    }
    app.emit("search-state-changed", &state.search_state.read().clone()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn load_more(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<SearchResponse, String> {
    // Get current search state
    let search_state = state.search_state.read().clone();
    
    // Check if there are more results
    if !search_state.has_next {
        return Err("No more results to load".to_string());
    }
    
    // Increment page and update state
    let next_page = search_state.page + 1;
    {
        let mut ss = state.search_state.write();
        ss.page = next_page;
    }
    
    // Construct SearchRequest from SearchState
    let request = SearchRequest {
        query: search_state.query.clone(),
        page: next_page,
        page_size: search_state.page_size,
        exclude_watched: search_state.exclude_watched,
        filters: search_state.filters.clone(),
        sort: search_state.sort.clone(),
        formula_filter: search_state.formula_filter.clone(),
    };
    
    // Execute search using helper
    let response = execute_search(&state, &request)?;
    
    // Update search state with new has_next
    {
        let mut ss = state.search_state.write();
        ss.has_next = response.has_next;
        ss.total_count = response.total;
    }
    
    // Emit event for UI update
    app.emit("search-results-updated", &response).map_err(|e| e.to_string())?;
    
    Ok(response)
}

fn execute_search(state: &AppState, request: &SearchRequest) -> Result<SearchResponse, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    
    let mut sql = String::from(
        "SELECT v.id, v.title, v.thumbnail_url, v.watch_url,          v.view_count, v.comment_count, v.mylist_count, v.like_count,          v.start_time, v.tags, v.duration, v.uploader_id, v.uploader_name, v.description          FROM videos v"
    );
    
    let mut count_sql = String::from("SELECT COUNT(*) as total FROM videos v");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut where_clauses: Vec<String> = Vec::new();
    
    if !request.query.is_empty() {
        where_clauses.push("(v.title LIKE ? OR v.tags LIKE ?)".to_string());
        let query_pattern = format!("%{}%", request.query.replace('%', r"\%").replace('_', r"\_"));
        params.push(Box::new(query_pattern.clone()));
        params.push(Box::new(query_pattern));
    }
    
    if let Some(ref filters) = request.filters {
        if let Some(ref v) = filters.view {
            if let Some(gte) = v.gte { where_clauses.push("v.view_count >= ?".to_string()); params.push(Box::new(gte as i64)); }
            if let Some(lte) = v.lte { where_clauses.push("v.view_count <= ?".to_string()); params.push(Box::new(lte as i64)); }
        }
        if let Some(ref m) = filters.mylist {
            if let Some(gte) = m.gte { where_clauses.push("v.mylist_count >= ?".to_string()); params.push(Box::new(gte as i64)); }
            if let Some(lte) = m.lte { where_clauses.push("v.mylist_count <= ?".to_string()); params.push(Box::new(lte as i64)); }
        }
        if let Some(ref c) = filters.comment {
            if let Some(gte) = c.gte { where_clauses.push("v.comment_count >= ?".to_string()); params.push(Box::new(gte as i64)); }
            if let Some(lte) = c.lte { where_clauses.push("v.comment_count <= ?".to_string()); params.push(Box::new(lte as i64)); }
        }
        if let Some(ref l) = filters.like {
            if let Some(gte) = l.gte { where_clauses.push("v.like_count >= ?".to_string()); params.push(Box::new(gte as i64)); }
            if let Some(lte) = l.lte { where_clauses.push("v.like_count <= ?".to_string()); params.push(Box::new(lte as i64)); }
        }
        if let Some(ref t) = filters.start_time {
            if let Some(ref gte) = t.gte {
                let gte_str = format!("{}T00:00:00+09:00", gte);
                where_clauses.push("v.start_time >= ?".to_string());
                params.push(Box::new(gte_str));
            }
            if let Some(ref lte) = t.lte {
                let lte_str = format!("{}T23:59:59+09:00", lte);
                where_clauses.push("v.start_time <= ?".to_string());
                params.push(Box::new(lte_str));
            }
        }
    }
    
    if request.exclude_watched {
        where_clauses.push("NOT EXISTS (SELECT 1 FROM history h WHERE h.video_id = v.id)".to_string());
    }
    
    if !where_clauses.is_empty() {
        let where_clause = format!(" WHERE {}", where_clauses.join(" AND "));
        sql.push_str(&where_clause);
        count_sql.push_str(&where_clause);
    }
    
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    
    let total: usize = conn.query_row(&count_sql, &params_refs[..], |row| {
        row.get::<_, i64>(0).map(|n| n as usize)
    }).unwrap_or(0);
    
    let order_by = if let Some(ref sort) = request.sort {
        let field = match sort.by.as_str() {
            "view" => "v.view_count",
            "mylist" => "v.mylist_count",
            "comment" => "v.comment_count",
            "like" => "v.like_count",
            "start_time" => "v.start_time",
            "custom" => {
                if let Some(ref weights) = sort.weights {
                    &format!(
                        "({} * v.view_count + {} * v.mylist_count + {} * v.comment_count + {} * v.like_count)",
                        weights.view, weights.mylist, weights.comment, weights.like
                    ).leak()
                } else {
                    "v.view_count"
                }
            }
            _ => "v.view_count",
        };
        let direction = if sort.direction == "asc" { "ASC" } else { "DESC" };
        format!(" ORDER BY {} {}", field, direction)
    } else {
        " ORDER BY v.view_count DESC".to_string()
    };
    
    sql.push_str(&order_by);
    sql.push_str(&format!(" LIMIT {} OFFSET {}", request.page_size, (request.page - 1) * request.page_size));
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    
    let results: Vec<Video> = stmt.query_map(&params_refs[..], |row| {
        let id: String = row.get(0)?;
        let is_watched = state.db.is_video_watched(&id).unwrap_or(false);
        
        Ok(Video {
            id,
            title: row.get(1)?,
            thumbnail_url: row.get(2)?,
            watch_url: row.get(3)?,
            view_count: row.get(4)?,
            comment_count: row.get(5)?,
            mylist_count: row.get(6)?,
            like_count: row.get(7)?,
            start_time: row.get(8)?,
            tags: parse_tags(row.get::<_, Option<String>>(9)?.as_deref()),
            duration: row.get(10)?,
            uploader_id: row.get(11)?,
            uploader_name: row.get(12)?,
            description: row.get(13)?,
            is_watched,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|v| v.ok())
    .collect();
    
    let has_next = (request.page * request.page_size) < total;

    {
        let mut results_lock = state.search_results.write();
        if request.page == 1 {
            *results_lock = results.clone();
            let mut index_lock = state.playlist_index.write();
            *index_lock = 0;
        } else {
            results_lock.extend(results.clone());
        }
    }
    
    Ok(SearchResponse {
        total,
        page: request.page,
        page_size: request.page_size,
        has_next,
        results,
    })
}

fn parse_tags(tags: Option<&str>) -> Vec<String> {
    tags.map(|t| t.split_whitespace().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn search(
    request: SearchRequest,
    state: tauri::State<'_, AppState>,
) -> Result<SearchResponse, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    
    let mut sql = String::from(
        "SELECT v.id, v.title, v.thumbnail_url, v.watch_url, \
         v.view_count, v.comment_count, v.mylist_count, v.like_count, \
         v.start_time, v.tags, v.duration, v.uploader_id, v.uploader_name, v.description \
         FROM videos v"
    );
    
    let mut count_sql = String::from("SELECT COUNT(*) as total FROM videos v");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut where_clauses: Vec<String> = Vec::new();
    
    if !request.query.is_empty() {
        where_clauses.push("(v.title LIKE ? OR v.tags LIKE ?)".to_string());
        let query_pattern = format!("%{}%", request.query.replace('%', "\\%").replace('_', "\\_"));
        params.push(Box::new(query_pattern.clone()));
        params.push(Box::new(query_pattern));
    }
    
    if let Some(ref filters) = request.filters {
        if let Some(ref v) = filters.view {
            if let Some(gte) = v.gte {
                where_clauses.push("v.view_count >= ?".to_string());
                params.push(Box::new(gte as i64));
            }
            if let Some(lte) = v.lte {
                where_clauses.push("v.view_count <= ?".to_string());
                params.push(Box::new(lte as i64));
            }
        }
        if let Some(ref m) = filters.mylist {
            if let Some(gte) = m.gte {
                where_clauses.push("v.mylist_count >= ?".to_string());
                params.push(Box::new(gte as i64));
            }
            if let Some(lte) = m.lte {
                where_clauses.push("v.mylist_count <= ?".to_string());
                params.push(Box::new(lte as i64));
            }
        }
        if let Some(ref c) = filters.comment {
            if let Some(gte) = c.gte {
                where_clauses.push("v.comment_count >= ?".to_string());
                params.push(Box::new(gte as i64));
            }
            if let Some(lte) = c.lte {
                where_clauses.push("v.comment_count <= ?".to_string());
                params.push(Box::new(lte as i64));
            }
        }
        if let Some(ref l) = filters.like {
            if let Some(gte) = l.gte {
                where_clauses.push("v.like_count >= ?".to_string());
                params.push(Box::new(gte as i64));
            }
            if let Some(lte) = l.lte {
                where_clauses.push("v.like_count <= ?".to_string());
                params.push(Box::new(lte as i64));
            }
        }
        if let Some(ref t) = filters.start_time {
            if let Some(ref gte) = t.gte {
                let gte_str = format!("{}T00:00:00+09:00", gte);
                where_clauses.push("v.start_time >= ?".to_string());
                params.push(Box::new(gte_str));
            }
            if let Some(ref lte) = t.lte {
                let lte_str = format!("{}T23:59:59+09:00", lte);
                where_clauses.push("v.start_time <= ?".to_string());
                params.push(Box::new(lte_str));
            }
        }
    }
    
    if request.exclude_watched {
        where_clauses.push("NOT EXISTS (SELECT 1 FROM history h WHERE h.video_id = v.id)".to_string());
    }
    
    if !where_clauses.is_empty() {
        let where_clause = format!(" WHERE {}", where_clauses.join(" AND "));
        sql.push_str(&where_clause);
        count_sql.push_str(&where_clause);
    }
    
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    
    let total: usize = conn.query_row(&count_sql, &params_refs[..], |row| {
        row.get::<_, i64>(0).map(|n| n as usize)
    }).unwrap_or(0);
    
    let order_by = if let Some(ref sort) = request.sort {
        let field = match sort.by.as_str() {
            "view" => "v.view_count",
            "mylist" => "v.mylist_count",
            "comment" => "v.comment_count",
            "like" => "v.like_count",
            "start_time" => "v.start_time",
            "custom" => {
                if let Some(ref weights) = sort.weights {
                    &format!(
                        "({} * v.view_count + {} * v.mylist_count + {} * v.comment_count + {} * v.like_count)",
                        weights.view, weights.mylist, weights.comment, weights.like
                    ).leak()
                } else {
                    "v.view_count"
                }
            }
            _ => "v.view_count",
        };
        let direction = if sort.direction == "asc" { "ASC" } else { "DESC" };
        format!(" ORDER BY {} {}", field, direction)
    } else {
        " ORDER BY v.view_count DESC".to_string()
    };
    
    sql.push_str(&order_by);
    sql.push_str(&format!(" LIMIT {} OFFSET {}", request.page_size, (request.page - 1) * request.page_size));
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    
    let results: Vec<Video> = stmt.query_map(&params_refs[..], |row| {
        let id: String = row.get(0)?;
        let is_watched = state.db.is_video_watched(&id).unwrap_or(false);
        
        Ok(Video {
            id,
            title: row.get(1)?,
            thumbnail_url: row.get(2)?,
            watch_url: row.get(3)?,
            view_count: row.get(4)?,
            comment_count: row.get(5)?,
            mylist_count: row.get(6)?,
            like_count: row.get(7)?,
            start_time: row.get(8)?,
            tags: parse_tags(row.get::<_, Option<String>>(9)?.as_deref()),
            duration: row.get(10)?,
            uploader_id: row.get(11)?,
            uploader_name: row.get(12)?,
            description: row.get(13)?,
            is_watched,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|v| v.ok())
    .collect();
    
    let has_next = (request.page * request.page_size) < total;

    // Update search_state when starting a new search (page 1)
    if request.page == 1 {
        let mut ss = state.search_state.write();
        ss.query = request.query.clone();
        ss.exclude_watched = request.exclude_watched;
        ss.filters = request.filters.clone();
        ss.sort = request.sort.clone();
        ss.formula_filter = request.formula_filter.clone();
        ss.page = 1;
        ss.page_size = request.page_size;
        ss.has_next = has_next;
        ss.total_count = total;
    } else {
        // Update pagination state for subsequent pages
        let mut ss = state.search_state.write();
        ss.page = request.page;
        ss.has_next = has_next;
    }

    {
        let mut results_lock = state.search_results.write();
        if request.page == 1 {
            *results_lock = results.clone();
            let mut index_lock = state.playlist_index.write();
            *index_lock = 0;
        } else {
            results_lock.extend(results.clone());
        }
    }

    Ok(SearchResponse {
        total,
        page: request.page,
        page_size: request.page_size,
        has_next,
        results,
    })
}

#[tauri::command]
pub async fn get_video(
    video_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<Video>, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    
    let result = conn.query_row(
        "SELECT id, title, thumbnail_url, watch_url, view_count, comment_count, \
         mylist_count, like_count, start_time, tags, duration, uploader_id, uploader_name, description \
         FROM videos WHERE id = ?",
        [&video_id],
        |row| {
            let id: String = row.get(0)?;
            let is_watched = state.db.is_video_watched(&id).unwrap_or(false);
            
            Ok(Video {
                id,
                title: row.get(1)?,
                thumbnail_url: row.get(2)?,
                watch_url: row.get(3)?,
                view_count: row.get(4)?,
                comment_count: row.get(5)?,
                mylist_count: row.get(6)?,
                like_count: row.get(7)?,
                start_time: row.get(8)?,
                tags: parse_tags(row.get::<_, Option<String>>(9)?.as_deref()),
                duration: row.get(10)?,
                uploader_id: row.get(11)?,
                uploader_name: row.get(12)?,
                description: row.get(13)?,
                is_watched,
            })
        }
    );
    
    match result {
        Ok(video) => Ok(Some(video)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_user_info(
    video_id: String,
) -> Result<Option<UserInfo>, String> {
    let client = reqwest::Client::builder()
        .user_agent("vocaloid-search-desktop/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    
    let url = format!("https://ext.nicovideo.jp/api/getthumbinfo/{}", video_id);
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Ok(None);
    }
    
    let body = response.text().await.map_err(|e| e.to_string())?;
    
    let mut reader = Reader::from_str(&body);
    reader.config_mut().trim_text(true);
    
    let mut user_id: Option<String> = None;
    let mut user_nickname: Option<String> = None;
    let mut current_tag: String = String::new();
    let mut in_thumb = false;
    
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "thumb" {
                    in_thumb = true;
                }
                current_tag = name;
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_thumb {
                    match current_tag.as_str() {
                        "user_id" => user_id = Some(text),
                        "user_nickname" => user_nickname = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "thumb" {
                    break;
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    
    let user_icon_url = user_id.as_ref().map(|id| {
        let bucket = id.parse::<u64>().unwrap_or(0) / 10000;
        format!("https://secure-dcdn.cdn.nimg.jp/nicoaccount/usericon/{}/{}.jpg", bucket, id)
    });
    
    Ok(Some(UserInfo {
        user_id,
        user_nickname,
        user_icon_url,
    }))
}

#[tauri::command]
pub async fn fetch_video_metadata(
    video_id: String,
) -> Result<Option<Video>, String> {
    let client = reqwest::Client::builder()
        .user_agent("vocaloid-search-desktop/1.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    
    let url = "https://snapshot.search.nicovideo.jp/api/v2/snapshot/video/contents/search";
    
    let response = client
        .get(url)
        .query(&[
            ("q", video_id.as_str()),
            ("targets", "contentId"),
            ("fields", "contentId,title,thumbnailUrl,viewCounter,commentCounter,mylistCounter,likeCounter,startTime,tags,lengthSeconds,genre,description,userId"),
            ("_limit", "1"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Ok(None);
    }
    
    let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    let videos = data.get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    
    if videos.is_empty() {
        return Ok(None);
    }
    
    match serde_json::from_value::<crate::models::SnapshotVideo>(videos[0].clone()) {
        Ok(snapshot) => {
            let thumbnail_url = if snapshot.thumbnailUrl.is_object() {
                snapshot.thumbnailUrl.get("large").and_then(|u| u.as_str()).map(|s| s.to_string())
            } else {
                snapshot.thumbnailUrl.as_str().map(|s| s.to_string())
            };
            
            let tags = snapshot.tags.as_ref().map(|t| {
                if t.is_array() {
                    t.as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                        .unwrap_or_default()
                } else {
                    t.as_str()
                        .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                        .unwrap_or_default()
                }
            }).unwrap_or_default();
            
            Ok(Some(Video {
                id: snapshot.contentId,
                title: snapshot.title,
                thumbnail_url,
                watch_url: None,
                view_count: snapshot.viewCounter.unwrap_or(0),
                comment_count: snapshot.commentCounter.unwrap_or(0),
                mylist_count: snapshot.mylistCounter.unwrap_or(0),
                like_count: snapshot.likeCounter.unwrap_or(0),
                start_time: snapshot.startTime,
                tags,
                duration: snapshot.lengthSeconds,
                uploader_id: snapshot.userId,
                uploader_name: None,
                description: snapshot.description,
                is_watched: false,
            }))
        }
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub async fn mark_watched(
    app: AppHandle,
    video_id: String,
    title: String,
    thumbnail_url: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 1. Update database
    state.db.mark_watched(&video_id, &title, thumbnail_url.as_deref()).map_err(|e| e.to_string())?;
    
    // 2. Update AppState.search_results
    {
        let mut results = state.search_results.write();
        if let Some(video) = results.iter_mut().find(|v| v.id == video_id) {
            video.is_watched = true;
        }
    }
    
    // 3. Emit event for UI update
    app.emit("video-watched", serde_json::json!({
        "video_id": video_id,
        "is_watched": true
    })).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_watched(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT video_id FROM watched ORDER BY watched_at DESC")
        .map_err(|e| e.to_string())?;
    
    let ids: Vec<String> = stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    
    Ok(ids)
}

#[tauri::command]
pub async fn get_history(
    page: usize,
    page_size: usize,
    state: tauri::State<'_, AppState>,
) -> Result<HistoryResponse, String> {
    let total = state.db.get_history_count().map_err(|e| e.to_string())?;
    let entries = state.db.get_history(page, page_size).map_err(|e| e.to_string())?;
    
    let has_next = (page * page_size) < total;
    
    Ok(HistoryResponse {
        total,
        page,
        page_size,
        has_next,
        results: entries,
    })
}

#[tauri::command]
pub async fn get_scraper_config(
    state: tauri::State<'_, AppState>,
) -> Result<ScraperConfig, String> {
    let stored = state.db.get_config().map_err(|e| e.to_string())?;
    let config = ScraperConfig {
        query: stored.query,
        max_age_days: stored.max_age_days,
        targets: stored.targets,
        category_filter: stored.category_filter,
    };
    
    let mut current = state.config.write();
    *current = config.clone();
    
    Ok(config)
}

#[tauri::command]
pub async fn save_scraper_config(
    config: ScraperConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let stored = crate::database::StoredConfig {
        query: config.query.clone(),
        max_age_days: config.max_age_days,
        targets: config.targets.clone(),
        category_filter: config.category_filter.clone(),
    };
    
    state.db.save_config(&stored).map_err(|e| e.to_string())?;
    
    let mut current = state.config.write();
    *current = config;
    Ok(())
}

#[tauri::command]
pub async fn run_scraper(
    _app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.read().clone();
    
    {
        let mut progress = state.scraper_progress.write();
        progress.is_running = true;
        progress.videos_fetched = 0;
        progress.total_expected = None;
        progress.status = "clearing".to_string();
    }
    
    state.db.clear_videos().map_err(|e| e.to_string())?;
    
    {
        let mut progress = state.scraper_progress.write();
        progress.status = "fetching".to_string();
    }
    
    let (tx, rx) = async_channel::bounded::<()>(1);
    {
        let mut cancel = state.scraper_cancel.write();
        *cancel = Some(tx);
    }
    
    let scraper = Scraper::new(config).with_cancel(rx);
    let db = state.db.clone();
    let progress = state.scraper_progress.clone();
    let cancel_receiver = state.scraper_cancel.clone();
    
    tokio::spawn(async move {
        let progress_clone = progress.clone();
        let db_clone = db.clone();
        
        let result = scraper.fetch_videos(move |fetched, total| {
            let mut p = progress_clone.write();
            p.videos_fetched = fetched;
            p.total_expected = total;
        }).await;
        
        {
            let mut cancel = cancel_receiver.write();
            *cancel = None;
        }
        
        match result {
            Ok(scraper_result) => {
                {
                    let mut p = progress.write();
                    p.status = "inserting".to_string();
                }
                
                let batch_size = 1000;
                let videos: Vec<_> = scraper_result.videos.iter().map(snapshot_to_db_row).collect();
                
                for chunk in videos.chunks(batch_size) {
                    if let Err(e) = db_clone.insert_videos_batch(chunk) {
                        let mut p = progress.write();
                        p.status = format!("error: {}", e);
                        p.is_running = false;
                        return;
                    }
                }
                
                let mut p = progress.write();
                p.status = "complete".to_string();
                p.is_running = false;
            }
            Err(e) => {
                let mut p = progress.write();
                if e.to_string().contains("cancelled") {
                    p.status = "cancelled".to_string();
                } else {
                    p.status = format!("error: {}", e);
                }
                p.is_running = false;
            }
        }
    });
    
    Ok(())
}

#[tauri::command]
pub async fn get_scraper_progress(
    state: tauri::State<'_, AppState>,
) -> Result<ScraperProgress, String> {
    let progress = state.scraper_progress.read().clone();
    Ok(progress)
}

#[tauri::command]
pub async fn cancel_scraper(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let sender = {
        let cancel = state.scraper_cancel.read();
        cancel.clone()
    };
    
    if let Some(tx) = sender {
        let _ = tx.send(()).await;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn get_database_stats(
    state: tauri::State<'_, AppState>,
) -> Result<DatabaseStats, String> {
    let total_videos = state.db.get_total_videos().map_err(|e| e.to_string())?;
    let last_update = state.db.get_last_update().map_err(|e| e.to_string())?;
    
    Ok(DatabaseStats {
        total_videos,
        last_update,
    })
}

#[tauri::command]
pub async fn check_database_freshness(
    state: tauri::State<'_, AppState>,
) -> Result<FreshnessCheck, String> {
    let local_last_update = state.db.get_last_update().map_err(|e| e.to_string())?;
    let api_last_update = check_snapshot_api_last_update().await.ok().flatten();
    
    // Calculate the most recent 6:00 JST threshold
    // Logic: Look back from now, find the first 6:00 JST
    // If local update time >= this threshold, database is fresh
    use chrono::{Utc, TimeZone, Datelike};
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let now_jst = Utc::now().with_timezone(&jst_offset);
    
    // Calculate today's 6:00 JST
    let today_6am_jst = jst_offset.with_ymd_and_hms(
        now_jst.year(),
        now_jst.month(),
        now_jst.day(),
        6, 0, 0
    ).single().unwrap_or(now_jst);
    
    // If current time is before today's 6:00, the threshold is yesterday's 6:00
    // Otherwise, the threshold is today's 6:00
    let threshold_6am_jst = if now_jst < today_6am_jst {
        // Before today's 6:00, use yesterday's 6:00
        today_6am_jst - chrono::Duration::days(1)
    } else {
        // After today's 6:00, use today's 6:00
        today_6am_jst
    };
    let threshold_str = threshold_6am_jst.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let is_fresh = if local_last_update.is_none() {
        false
    } else if let Some(ref local) = local_last_update {
        let local_str = local.as_str();
        // Database is fresh if last update >= the most recent 6:00 JST
        local_str >= threshold_str.as_str()
    } else {
        false
    };
    
    let message = if is_fresh {
        "資料庫已是最新".to_string()
    } else if local_last_update.is_none() {
        "資料庫為空，請先同步資料".to_string()
    } else if let Some(ref local) = local_last_update {
        format!(
            "資料庫過時，建議更新 (上次更新: {}, 分界點: {})",
            local,
            threshold_str
        )
    } else {
        "資料庫狀態未知".to_string()
    };
    
    Ok(FreshnessCheck {
        is_fresh,
        local_last_update,
        api_last_update,
        message,
    })
}

#[tauri::command]
pub async fn open_pip_window(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let current_index = *state.playlist_index.read();
    let results = state.search_results.read();
    let current_video = results.get(current_index).cloned();
    
    let saved_state = crate::database::load_pip_window_state(&app);
    let width = saved_state.as_ref().map(|s| s.width as f64).unwrap_or(450.0);
    let height = saved_state.as_ref().map(|s| s.height as f64).unwrap_or(500.0);
    let x = saved_state.as_ref().map(|s| s.x);
    let y = saved_state.as_ref().map(|s| s.y);
    
    #[cfg(target_os = "windows")]
    {
        let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let webview_data_dir = data_dir.join("webview_data");

        let mut builder = WebviewWindowBuilder::new(
            &app,
            "pip",
            WebviewUrl::App("pip.html".into())
        )
        .data_directory(webview_data_dir)
        .title("VOCALOID Search - PiP")
        .inner_size(width, height)
        .min_inner_size(300.0, 300.0)
        .resizable(true)
        .always_on_top(true)
        .decorations(true);
        
        if let (Some(px), Some(py)) = (x, y) {
            builder = builder.position(px as f64, py as f64);
        }
        
        let window = builder.build().map_err(|e| e.to_string())?;

        if let Some(video) = current_video {
            let has_next = current_index + 1 < results.len();
            window.emit("video-selected", VideoSelectedPayload {
                video,
                index: current_index,
                has_next,
            }).map_err(|e| e.to_string())?;
        }
        
        {
            let mut pip_active = state.pip_active.write();
            *pip_active = true;
        }
        app.emit("pip-opened", &current_index).map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut builder = WebviewWindowBuilder::new(
            &app,
            "pip",
            WebviewUrl::App("pip.html".into())
        )
        .title("VOCALOID Search - PiP")
        .inner_size(width, height)
        .min_inner_size(300.0, 300.0)
        .resizable(true)
        .always_on_top(true)
        .decorations(true);
        
        if let (Some(px), Some(py)) = (x, y) {
            builder = builder.position(px as f64, py as f64);
        }
        
        let window = builder.build().map_err(|e| e.to_string())?;

        if let Some(video) = current_video {
            let has_next = current_index + 1 < results.len();
            window.emit("video-selected", VideoSelectedPayload {
                video,
                index: current_index,
                has_next,
            }).map_err(|e| e.to_string())?;
        }
        
        {
            let mut pip_active = state.pip_active.write();
            *pip_active = true;
        }
        app.emit("pip-opened", &current_index).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn notify_pip_closing(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut pip_active = state.pip_active.write();
        *pip_active = false;
    }
    app.emit("pip-closed", ()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn close_pip_window(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("pip") {
        window.close().map_err(|e| e.to_string())?;
    }
    {
        let mut pip_active = state.pip_active.write();
        *pip_active = false;
    }
    app.emit("pip-closed", ()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn select_video(
    app: AppHandle,
    video_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pip_active = *state.pip_active.read();
    
    if pip_active {
        if let Some(window) = app.get_webview_window("pip") {
            window.emit("play-video", &video_id).map_err(|e| e.to_string())?;
        }
    } else {
        app.emit("main-play-video", &video_id).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn play_next(
    state: tauri::State<'_, AppState>,
) -> Result<Option<Video>, String> {
    let mut index = state.playlist_index.write();
    let results = state.search_results.read();
    
    if *index + 1 < results.len() {
        *index += 1;
        Ok(Some(results[*index].clone()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn play_previous(
    state: tauri::State<'_, AppState>,
) -> Result<Option<Video>, String> {
    let mut index = state.playlist_index.write();
    
    if *index > 0 {
        *index -= 1;
        let results = state.search_results.read();
        Ok(Some(results[*index].clone()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn get_database_path(
    app: tauri::AppHandle,
) -> Result<String, String> {
    use crate::database::{get_data_dir, get_db_path};
    let data_dir = get_data_dir(&app);
    let db_path = get_db_path(&app);
    Ok(format!("Data directory: {}\nDatabase: {}", data_dir.display(), db_path.display()))
}

#[tauri::command]
pub async fn save_window_state(
    app: tauri::AppHandle,
    state: WindowState,
) -> Result<(), String> {
    crate::database::save_window_state(&app, &state)
}

#[tauri::command]
pub async fn load_window_state(
    app: tauri::AppHandle,
) -> Result<Option<WindowState>, String> {
    Ok(crate::database::load_window_state(&app))
}

#[tauri::command]
pub async fn save_pip_window_state(
    app: tauri::AppHandle,
    state: PipWindowState,
) -> Result<(), String> {
    crate::database::save_pip_window_state(&app, &state)
}

#[tauri::command]
pub async fn load_pip_window_state(
    app: tauri::AppHandle,
) -> Result<Option<PipWindowState>, String> {
    Ok(crate::database::load_pip_window_state(&app))
}
