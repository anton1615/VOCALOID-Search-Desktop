use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use crate::models::*;
use crate::playback_settings_config::{playback_settings_from_stored_config, stored_config_with_playback_settings};
use crate::scraper::{Scraper, snapshot_to_db_row, check_snapshot_api_last_update};
use crate::state::AppState;
use async_channel;
use quick_xml::Reader;
use quick_xml::events::Event;

fn playlist_results_for_type(state: &AppState, playlist_type: PlaylistType) -> Vec<Video> {
    let list_id = ListContextId::from(playlist_type);
    state.get_list_context_items(&list_id)
}

fn playlist_version_for_type(state: &AppState, playlist_type: PlaylistType) -> u64 {
    match playlist_type {
        PlaylistType::Search => state.search_state.read().version,
        PlaylistType::History => state.history_state.read().version,
        PlaylistType::WatchLater => state.watch_later_state.read().version,
    }
}

fn build_playback_metadata_update_payload(
    state: &AppState,
    list_id: &ListContextId,
    playlist_version: u64,
    index: usize,
    video: Video,
) -> Option<PlaybackVideoUpdatedPayload> {
    if !state.matches_active_playback_metadata_update(list_id, playlist_version, index, &video) {
        return None;
    }

    Some(PlaybackVideoUpdatedPayload::new(
        list_id.clone(),
        playlist_version,
        index,
        video,
    ))
}

fn apply_playback_metadata_update(
    state: &AppState,
    list_id: &ListContextId,
    playlist_version: u64,
    index: usize,
    video: Video,
) -> Option<PlaybackVideoUpdatedPayload> {
    if !state.update_list_context_item(list_id, index, video.clone()) {
        return None;
    }

    build_playback_metadata_update_payload(state, list_id, playlist_version, index, video)
}

fn emit_active_playback_cleared(app: &AppHandle, list_id: &ListContextId) -> Result<(), String> {
    let payload = match list_id {
        ListContextId::Search => "Search",
        ListContextId::History => "History",
        ListContextId::WatchLater => "WatchLater",
        ListContextId::Custom(name) => name.as_str(),
    };

    app.emit("active-playback-cleared", payload)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PlaybackEnrichmentKind {
    FetchFullVideoInfo,
    FetchUserInfo,
}

#[derive(Debug, Clone, PartialEq)]
struct PlaybackEnrichmentRequest {
    kind: PlaybackEnrichmentKind,
    list_id: ListContextId,
    playlist_version: u64,
    index: usize,
    video: Video,
}

fn build_playback_enrichment_request(
    list_id: ListContextId,
    playlist_version: u64,
    index: usize,
    video: Video,
) -> PlaybackEnrichmentRequest {
    let kind = match list_id {
        ListContextId::Search => PlaybackEnrichmentKind::FetchUserInfo,
        _ => PlaybackEnrichmentKind::FetchFullVideoInfo,
    };

    PlaybackEnrichmentRequest {
        kind,
        list_id,
        playlist_version,
        index,
        video,
    }
}

fn merge_user_info_into_video(video: Video, user_info: Option<UserInfo>) -> Video {
    let Some(user_info) = user_info else {
        return video;
    };

    Video {
        uploader_id: user_info.user_id.or(video.uploader_id),
        uploader_name: user_info.user_nickname.or(video.uploader_name),
        ..video
    }
}

fn resolve_playback_enrichment_video(
    request: &PlaybackEnrichmentRequest,
    full_video: Option<Video>,
    user_info: Option<UserInfo>,
) -> Video {
    match request.kind {
        PlaybackEnrichmentKind::FetchUserInfo => {
            merge_user_info_into_video(request.video.clone(), user_info)
        }
        PlaybackEnrichmentKind::FetchFullVideoInfo => full_video.unwrap_or_else(|| request.video.clone()),
    }
}

fn build_active_playback_reentry_request(
    state: &AppState,
) -> Option<(VideoSelectedPayload, PlaybackEnrichmentRequest)> {
    let active_playback = state.active_playback.read();
    let playback = active_playback.as_ref()?.clone();
    drop(active_playback);

    let playlist_type = PlaylistType::from(&playback.list_id);
    let results = state.get_list_context_items(&playback.list_id);
    let video = results.get(playback.current_index)?.clone();
    let has_next = playback.current_index + 1 < results.len();

    Some((
        VideoSelectedPayload {
            video: video.clone(),
            index: playback.current_index,
            has_next,
            playlist_type,
            playlist_version: playback.list_version,
        },
        build_playback_enrichment_request(
            playback.list_id,
            playback.list_version,
            playback.current_index,
            video,
        ),
    ))
}

fn parse_duration_seconds(length: Option<&str>) -> Option<i64> {
    let length = length?;
    let parts: Vec<&str> = length.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let mins: i64 = parts[0].parse().ok()?;
    let secs: i64 = parts[1].parse().ok()?;
    Some(mins * 60 + secs)
}

fn build_full_video_from_thumbinfo(
    video_id: &str,
    thumb_info: ThumbInfo,
    snapshot_video: Option<SnapshotVideo>,
) -> Video {
    let like_count = snapshot_video
        .and_then(|video| video.likeCounter)
        .unwrap_or(0);

    Video {
        id: thumb_info.id,
        title: thumb_info.title,
        thumbnail_url: thumb_info.thumbnail_url,
        watch_url: Some(format!("https://www.nicovideo.jp/watch/{}", video_id)),
        view_count: thumb_info.view_counter.unwrap_or(0),
        comment_count: thumb_info.comment_num.unwrap_or(0),
        mylist_count: thumb_info.mylist_counter.unwrap_or(0),
        like_count,
        start_time: thumb_info.first_retrieve,
        tags: thumb_info.tags.unwrap_or_default(),
        duration: parse_duration_seconds(thumb_info.length.as_deref()),
        uploader_id: thumb_info.user_id,
        uploader_name: thumb_info.user_nickname,
        description: thumb_info.description,
        is_watched: false,
    }
}

async fn fetch_non_search_enrichment_video<ThumbFetch, ThumbFuture, SnapshotFetch, SnapshotFuture>(
    video_id: &str,
    fetch_thumbinfo: ThumbFetch,
    fetch_snapshot: SnapshotFetch,
) -> Result<Video, String>
where
    ThumbFetch: FnOnce() -> ThumbFuture,
    ThumbFuture: std::future::Future<Output = Result<ThumbInfo, String>>,
    SnapshotFetch: FnOnce() -> SnapshotFuture,
    SnapshotFuture: std::future::Future<Output = Option<SnapshotVideo>>,
{
    let (thumb_info, snapshot_video) = tokio::join!(fetch_thumbinfo(), fetch_snapshot());
    let thumb_info = thumb_info?;

    Ok(build_full_video_from_thumbinfo(
        video_id,
        thumb_info,
        snapshot_video,
    ))
}

async fn fetch_thumbinfo(client: &reqwest::Client, video_id: &str) -> Result<ThumbInfo, String> {
    let thumb_url = format!("https://ext.nicovideo.jp/api/getthumbinfo/{}", video_id);
    let thumb_response = client
        .get(&thumb_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch thumbinfo: {}", e))?;
    let thumb_xml = thumb_response
        .text()
        .await
        .map_err(|e| format!("Failed to read thumbinfo response: {}", e))?;

    parse_thumbinfo_xml(&thumb_xml, video_id)
}

async fn fetch_snapshot_video(client: &reqwest::Client, video_id: &str) -> Option<SnapshotVideo> {
    #[derive(Debug, serde::Deserialize)]
    struct SnapshotResponse {
        data: Vec<SnapshotVideo>,
    }

    let snapshot_url = format!(
        "https://snapshot.search.nicovideo.jp/api/v2/snapshot/video/contents/search?q={}&fields=likeCounter&targets=contentId",
        video_id
    );

    match client.get(&snapshot_url).send().await {
        Ok(snapshot_response) => match snapshot_response.json::<SnapshotResponse>().await {
            Ok(snapshot_data) => snapshot_data.data.into_iter().next(),
            Err(e) => {
                eprintln!(
                    "[fetch_full_video_info] Failed to parse snapshot response for {}: {}",
                    video_id, e
                );
                None
            }
        },
        Err(e) => {
            eprintln!(
                "[fetch_full_video_info] Failed to fetch snapshot for {}: {}",
                video_id, e
            );
            None
        }
    }
}

async fn fetch_full_video_info_with_client(
    client: reqwest::Client,
    video_id: String,
) -> Result<Video, String> {
    let thumb_client = client.clone();
    let snapshot_client = client;
    let thumb_video_id = video_id.clone();
    let snapshot_video_id = video_id.clone();

    fetch_non_search_enrichment_video(
        &video_id,
        move || async move { fetch_thumbinfo(&thumb_client, &thumb_video_id).await },
        move || async move { fetch_snapshot_video(&snapshot_client, &snapshot_video_id).await },
    )
    .await
}

#[tauri::command]
pub async fn fetch_full_video_info(
    video_id: String,
) -> Result<Video, String> {
    let client = reqwest::Client::builder()
        .user_agent("vocaloid-search-desktop/1.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let video = fetch_full_video_info_with_client(client, video_id.clone()).await?;

    eprintln!(
        "[fetch_full_video_info] {} -> title={:?}, start_time={:?}, uploader_id={:?}, uploader_name={:?}, view_count={}, tags={}, has_description={}",
        video_id,
        video.title,
        video.start_time,
        video.uploader_id,
        video.uploader_name,
        video.view_count,
        video.tags.len(),
        video.description.is_some()
    );

    Ok(video)
}

#[tauri::command]
pub async fn get_playlist_state(
    state: tauri::State<'_, AppState>,
) -> Result<PlaylistState, String> {
    // Check if there's active playback
    let active_playback = state.active_playback.read().clone();
    
    if let Some(ref playback) = active_playback {
        // Get the list context for active playback
        let list_contexts = state.list_contexts.read();
        if let Some(context) = list_contexts.get(&playback.list_id) {
            let playlist_type = PlaylistType::from(&playback.list_id);
            let results = context.items.clone();
            let index = Some(playback.current_index);
            let has_next = index.map(|i| i + 1 < results.len()).unwrap_or(false);
            let pip_active = *state.pip_active.read();
            let playlist_version = context.version;

            return Ok(PlaylistState {
                playlist_type,
                results,
                index,
                has_next,
                pip_active,
                playlist_version,
            });
        }
    }
    
    // No active playback - return empty state with default Search
    let pip_active = *state.pip_active.read();

    Ok(PlaylistState {
        playlist_type: PlaylistType::Search,
        results: vec![],
        index: None,
        has_next: false,
        pip_active,
        playlist_version: 1,
    })
}

#[tauri::command]
pub async fn set_playlist_index(
    app: AppHandle,
    index: usize,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Use the current browsing list as the source for explicit selection.
    let list_id = state.get_browsing_list();
    let playlist_type = PlaylistType::from(&list_id);
    // Get results from list_contexts
    let results = state.get_list_context_items(&list_id);
    let results = if results.is_empty() {
        playlist_results_for_type(&state, playlist_type)
    } else {
        results
    };
    
    if index >= results.len() {
        return Err("Index out of bounds".to_string());
    }

    // Get current version from list_context
    let list_version = state.get_list_context_version(&list_id);

    // For Search, create or reuse the Search playback snapshot with frozen watched boundary
    if list_id == ListContextId::Search {
        let max_watched_seq = state.db.get_max_first_watched_seq().map_err(|e| e.to_string())?;
        state.create_or_reuse_search_playback_snapshot(list_version, max_watched_seq);
    }

    // Set active playback
    state.set_active_playback(list_id.clone(), list_version, index);

    let video = results[index].clone();
    let has_next = index + 1 < results.len();
    let playlist_version = state.get_list_context_version(&list_id);
    let enrichment_request = build_playback_enrichment_request(
        list_id.clone(),
        playlist_version,
        index,
        video.clone(),
    );

    app.emit("video-selected", VideoSelectedPayload {
        video,
        index,
        has_next,
        playlist_type,
        playlist_version,
    }).map_err(|e| e.to_string())?;

    let app_handle = app.clone();
    tokio::spawn(async move {
        let enriched_video = match enrichment_request.kind {
            PlaybackEnrichmentKind::FetchUserInfo => {
                let user_info = get_user_info(enrichment_request.video.id.clone())
                    .await
                    .ok()
                    .flatten();
                resolve_playback_enrichment_video(&enrichment_request, None, user_info)
            }
            PlaybackEnrichmentKind::FetchFullVideoInfo => {
                let full_video = fetch_full_video_info(enrichment_request.video.id.clone())
                    .await
                    .ok();
                resolve_playback_enrichment_video(&enrichment_request, full_video, None)
            }
        };

        let state = app_handle.state::<AppState>();
        if let Some(payload) = apply_playback_metadata_update(
            &state,
            &enrichment_request.list_id,
            enrichment_request.playlist_version,
            enrichment_request.index,
            enriched_video,
        ) {
            let _ = app_handle.emit("playback-video-updated", payload);
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn update_playlist_video(
    app: AppHandle,
    index: usize,
    video: Video,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let active_playback = state.active_playback.read();
    let playback = active_playback
        .as_ref()
        .cloned()
        .ok_or_else(|| "No active playback.".to_string())?;
    drop(active_playback);

    let payload = apply_playback_metadata_update(
        &state,
        &playback.list_id,
        playback.list_version,
        index,
        video,
    );

    if index >= state.get_list_context_items(&playback.list_id).len() {
        return Err("Index out of bounds or list not found".to_string());
    }

    if let Some(payload) = payload {
        app.emit("playback-video-updated", payload)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_playback_settings(
    state: tauri::State<'_, AppState>,
) -> Result<PlaybackSettings, String> {
    let stored = state.db.get_config().map_err(|e| e.to_string())?;
    let settings = playback_settings_from_stored_config(&stored);

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

    Ok(settings)
}

#[tauri::command]
pub async fn set_playback_settings(
    app: AppHandle,
    settings: PlaybackSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let stored = state.db.get_config().map_err(|e| e.to_string())?;
    let next_config = stored_config_with_playback_settings(&stored, &settings);
    state.db.save_config(&next_config).map_err(|e| e.to_string())?;

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
    let mut search_state = state.search_state.read().clone();
    // Sync all relevant fields from list_context (the authoritative source)
    if let Some(list_context) = state.get_list_context(&ListContextId::Search) {
        search_state.version = list_context.version;
        search_state.page = list_context.page;
        search_state.has_next = list_context.has_next;
        search_state.total_count = list_context.total_count;
        search_state.sort = list_context.sort.clone();
        search_state.results = list_context.items.clone();
    }
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
pub async fn set_search_loading(
    app: AppHandle,
    loading: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut current = state.search_state.write();
        current.loading = loading;
    }
    app.emit("search-state-changed", &state.search_state.read().clone()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn load_more(
    app: AppHandle,
    requested_playlist_type: Option<PlaylistType>,
    expected_version: Option<u64>,
    state: tauri::State<'_, AppState>,
) -> Result<SearchResponse, String> {
    let requested_playlist_type = requested_playlist_type.unwrap_or(PlaylistType::Search);
    let list_id = ListContextId::from(requested_playlist_type);
    
    // Get list context (the authoritative source for browsing state)
    let list_context = state.get_list_context(&list_id);
    
    // Validate version matches
    let current_version = list_context.as_ref().map(|c| c.version).unwrap_or(1);
    let expected_version = expected_version.unwrap_or(current_version);
    
    if current_version != expected_version {
        return Err("Stale load-more request: list context has changed".to_string());
    }
    
    // Get browsing state from list_context
    let context = match list_context {
        Some(ctx) => ctx,
        None => return Err("No list context found".to_string()),
    };
    
    // Check if there are more results
    if !context.has_next {
        return Err("No more results to load".to_string());
    }
    
    // Calculate next page
    let next_page = context.page + 1;
    
    // Construct SearchRequest from list_context browsing state
    let request = SearchRequest {
        query: context.query.clone(),
        page: next_page,
        page_size: context.page_size,
        exclude_watched: context.exclude_watched,
        filters: context.filters.clone(),
        sort: context.sort.clone(),
        formula_filter: context.formula_filter.clone(),
    };
    
    
    // Execute search using helper (does not update any list_context)
    let response = execute_search(&state, &request)?;
    
    // Append new items to list_context and update pagination state
    let extend_success = state.extend_list_context_items(&list_id, current_version, response.results.clone(), next_page, response.has_next);
    if !extend_success {
        return Err("Failed to extend list context: version mismatch or context not found".to_string());
    }
    // Also sync search_state with list_context for restore compatibility
    {
        let mut ss = state.search_state.write();
        ss.page = next_page;
        ss.has_next = response.has_next;
        ss.version = current_version;
        // Sync results from list_context so restore works correctly when playback is on another list
        ss.results = state.list_contexts.read().get(&list_id).map(|c| c.items.clone()).unwrap_or_default();
    }
    
    // Emit event for UI update
    app.emit("search-results-updated", &response).map_err(|e| e.to_string())?;
    
    Ok(response)
}

#[cfg(test)]
fn build_search_query(request: &SearchRequest, watched_ids: &[String]) -> (String, Vec<String>, String) {
    let mut sql = String::from(
        "SELECT v.id, v.title, v.thumbnail_url, v.watch_url, \
         v.view_count, v.comment_count, v.mylist_count, v.like_count, \
         v.start_time, v.tags, v.duration, v.uploader_id, v.uploader_name, v.description \
         FROM videos v"
    );
    
    let mut count_sql = String::from("SELECT COUNT(*) as total FROM videos v");
    let mut params: Vec<String> = Vec::new();
    let mut where_clauses: Vec<String> = Vec::new();
    
    if !request.query.is_empty() {
        where_clauses.push("(v.title LIKE ? OR v.tags LIKE ?)".to_string());
        let query_pattern = format!("%{}%", request.query.replace('%', r"\%").replace('_', r"\_"));
        params.push(query_pattern.clone());
        params.push(query_pattern);
    }
    
    if let Some(ref filters) = request.filters {
        if let Some(ref v) = filters.view {
            if let Some(gte) = v.gte { 
                where_clauses.push("v.view_count >= ?".to_string()); 
                params.push(gte.to_string()); 
            }
            if let Some(lte) = v.lte { 
                where_clauses.push("v.view_count <= ?".to_string()); 
                params.push(lte.to_string()); 
            }
        }
        if let Some(ref m) = filters.mylist {
            if let Some(gte) = m.gte { 
                where_clauses.push("v.mylist_count >= ?".to_string()); 
                params.push(gte.to_string()); 
            }
            if let Some(lte) = m.lte { 
                where_clauses.push("v.mylist_count <= ?".to_string()); 
                params.push(lte.to_string()); 
            }
        }
        if let Some(ref c) = filters.comment {
            if let Some(gte) = c.gte { 
                where_clauses.push("v.comment_count >= ?".to_string()); 
                params.push(gte.to_string()); 
            }
            if let Some(lte) = c.lte { 
                where_clauses.push("v.comment_count <= ?".to_string()); 
                params.push(lte.to_string()); 
            }
        }
        if let Some(ref l) = filters.like {
            if let Some(gte) = l.gte { 
                where_clauses.push("v.like_count >= ?".to_string()); 
                params.push(gte.to_string()); 
            }
            if let Some(lte) = l.lte { 
                where_clauses.push("v.like_count <= ?".to_string()); 
                params.push(lte.to_string()); 
            }
        }
        if let Some(ref t) = filters.start_time {
            if let Some(ref gte) = t.gte {
                let gte_str = format!("{}T00:00:00+09:00", gte);
                where_clauses.push("v.start_time >= ?".to_string());
                params.push(gte_str);
            }
            if let Some(ref lte) = t.lte {
                let lte_str = format!("{}T23:59:59+09:00", lte);
                where_clauses.push("v.start_time <= ?".to_string());
                params.push(lte_str);
            }
        }
    }
    
    if !watched_ids.is_empty() {
        let placeholders: Vec<String> = watched_ids.iter().map(|_| "?".to_string()).collect();
        where_clauses.push(format!("v.id NOT IN ({})", placeholders.join(", ")));
        for id in watched_ids {
            params.push(id.clone());
        }
    }
    
    if !where_clauses.is_empty() {
        let where_clause = format!(" WHERE {}", where_clauses.join(" AND "));
        sql.push_str(&where_clause);
        count_sql.push_str(&where_clause);
    }
    
    let order_by = if let Some(ref sort) = request.sort {
        let field = match sort.by {
            SortField::View => "v.view_count",
            SortField::Mylist => "v.mylist_count",
            SortField::Comment => "v.comment_count",
            SortField::Like => "v.like_count",
            SortField::StartTime => "v.start_time",
            SortField::Custom => {
                if let Some(ref weights) = sort.weights {
                    &format!(
                        "({} * v.view_count + {} * v.mylist_count + {} * v.comment_count + {} * v.like_count)",
                        weights.view, weights.mylist, weights.comment, weights.like
                    )
                } else {
                    "v.view_count"
                }
            }
            SortField::WatchedAt | SortField::AddedAt => "v.view_count",
        };
        let direction = match sort.direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        format!(" ORDER BY {} {}, v.id {}", field, direction, direction)
    } else {
        " ORDER BY v.view_count DESC, v.id DESC".to_string()
    };
    
    sql.push_str(&order_by);
    
    (sql, params, count_sql)
}

fn execute_search(state: &AppState, request: &SearchRequest) -> Result<SearchResponse, String> {
    // Determine watched exclusion: use frozen boundary if Search playback snapshot is active
    let watched_ids = if request.exclude_watched {
        let list_version = state.get_list_context_version(&ListContextId::Search);
        if let Some(snapshot) = state.get_search_playback_snapshot(list_version) {
            state.db.get_watched_ids_up_to_boundary(snapshot.frozen_watched_boundary_seq).unwrap_or_default()
        } else {
            state.db.get_all_watched_video_ids().unwrap_or_default()
        }
    } else {
        vec![]
    };

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
        // Use frozen boundary if Search playback snapshot is active, otherwise live state
        // The watched_ids variable is already computed above with the correct boundary
        if !watched_ids.is_empty() {
            let placeholders: Vec<String> = watched_ids.iter().map(|_| "?".to_string()).collect();
            where_clauses.push(format!("v.id NOT IN ({})", placeholders.join(", ")));
            for id in watched_ids {
                params.push(Box::new(id));
            }
        }
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
        let field = match sort.by {
            SortField::View => "v.view_count",
            SortField::Mylist => "v.mylist_count",
            SortField::Comment => "v.comment_count",
            SortField::Like => "v.like_count",
            SortField::StartTime => "v.start_time",
            SortField::Custom => {
                if let Some(ref weights) = sort.weights {
                    format!(
                        "({} * v.view_count + {} * v.mylist_count + {} * v.comment_count + {} * v.like_count)",
                        weights.view, weights.mylist, weights.comment, weights.like
                    ).leak()
                } else {
                    "v.view_count"
                }
            }
            SortField::WatchedAt | SortField::AddedAt => "v.view_count",
        };
        let direction = match sort.direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        format!(" ORDER BY {} {}, v.id {}", field, direction, direction)
    } else {
        " ORDER BY v.view_count DESC, v.id DESC".to_string()
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
    
    // Note: This function only returns results. The caller is responsible for updating list_context.
    // This ensures each list (Search, History, WatchLater) can manage its own state.
    
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
    app: AppHandle,
    request: SearchRequest,
    state: tauri::State<'_, AppState>,
) -> Result<SearchResponse, String> {
    
    // For new searches (page 1), reserve version upfront to invalidate in-flight load_more
    let reserved_version = if request.page == 1 {
        let v = state.reserve_list_context_version(
            &ListContextId::Search,
            request.query.clone(),
            request.sort.clone(),
            request.filters.clone(),
            request.exclude_watched,
            request.formula_filter.clone(),
        );
        Some(v)
    } else {
        None
    };
    
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
        // Get watched video IDs from user_data.db (history table is now separate)
        let watched_ids = state.db.get_all_watched_video_ids().unwrap_or_default();
        if !watched_ids.is_empty() {
            let placeholders: Vec<String> = watched_ids.iter().map(|_| "?".to_string()).collect();
            where_clauses.push(format!("v.id NOT IN ({})", placeholders.join(", ")));
            for id in watched_ids {
                params.push(Box::new(id));
            }
        }
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
        let field = match sort.by {
            SortField::View => "v.view_count",
            SortField::Mylist => "v.mylist_count",
            SortField::Comment => "v.comment_count",
            SortField::Like => "v.like_count",
            SortField::StartTime => "v.start_time",
            SortField::Custom => {
                if let Some(ref weights) = sort.weights {
                    format!(
                        "({} * v.view_count + {} * v.mylist_count + {} * v.comment_count + {} * v.like_count)",
                        weights.view, weights.mylist, weights.comment, weights.like
                    ).leak()
                } else {
                    "v.view_count"
                }
            }
            SortField::WatchedAt | SortField::AddedAt => "v.view_count",
        };
        let direction = match sort.direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        format!(" ORDER BY {} {}, v.id {}", field, direction, direction)
    } else {
        " ORDER BY v.view_count DESC, v.id DESC".to_string()
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

    // Update list_context for the new list-context model
    let list_version = if let Some(v) = reserved_version {
        // Use finalize for reserved version (page 1)
        let success = state.finalize_list_context_search(
            ListContextId::Search,
            v,
            results.clone(),
            request.page,
            request.page_size,
            has_next,
            total,
            request.query.clone(),
            request.sort.clone(),
            request.filters.clone(),
            request.exclude_watched,
            request.formula_filter.clone(),
        );
        if success {
            v
        } else {
            state.get_list_context_version(&ListContextId::Search)
        }
    } else {
        // For pagination (page > 1), use update_list_context which appends items
        state.update_list_context(
            ListContextId::Search,
            results.clone(),
            request.page,
            request.page_size,
            has_next,
            total,
            request.query.clone(),
            request.sort.clone(),
            request.filters.clone(),
            request.exclude_watched,
            request.formula_filter.clone(),
        )
    };

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
        ss.version = list_version;
        ss.results = results.clone();
        
        // Clear active playback for Search when query changes
        let playback_cleared = state.clear_active_playback_for_list(&ListContextId::Search);
        // Invalidate Search playback snapshot when Search conditions change
        state.invalidate_search_playback_snapshot();
        if playback_cleared {
            emit_active_playback_cleared(&app, &ListContextId::Search)?;
        }
    } else {
        // Update pagination state for subsequent pages
        let mut ss = state.search_state.write();
        ss.page = request.page;
        ss.has_next = has_next;
        ss.results = state.list_contexts.read().get(&ListContextId::Search).map(|c| c.items.clone()).unwrap_or_default();
        ss.version = list_version;
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
    
    // 2. Update is_watched in all list contexts
    {
        let mut contexts = state.list_contexts.write();
        for (_, context) in contexts.iter_mut() {
            if let Some(video) = context.items.iter_mut().find(|v| v.id == video_id) {
                video.is_watched = true;
            }
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
    app: AppHandle,
    page: usize,
    page_size: usize,
    sort_direction: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<HistoryResponse, String> {
    let total = state.db.get_history_count().map_err(|e| e.to_string())?;
    let entries = state.db.get_history(page, page_size, sort_direction.as_deref()).map_err(|e| e.to_string())?;
    
    let results: Vec<Video> = entries.iter().map(|entry| Video {
        id: entry.video_id.clone(),
        title: entry.title.clone(),
        thumbnail_url: entry.thumbnail_url.clone(),
        watch_url: None,
        view_count: 0,
        comment_count: 0,
        mylist_count: 0,
        like_count: 0,
        start_time: Some(entry.watched_at.clone()),
        tags: vec![],
        duration: None,
        uploader_id: None,
        uploader_name: None,
        description: None,
        is_watched: true,
    }).collect();
    
    let has_next = (page * page_size) < total;
    let requested_sort: SortDirection = sort_direction
        .as_deref()
        .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
        .unwrap_or(SortDirection::Desc);
    
    // Update History list_context
    if page == 1 {
        // Reserve version and update list_context for new history view
        let sort_config = crate::models::SortConfig {
            by: SortField::WatchedAt,
            direction: requested_sort,
            weights: None,
        };
        
        let mut contexts = state.list_contexts.write();
        let context = contexts.entry(ListContextId::History).or_default();
        
        // Check if sort changed to bump version
        let should_bump = context.sort.as_ref().map(|s| s.direction != requested_sort).unwrap_or(true);
        if should_bump {
            context.version += 1;
            // Clear active playback if History was playing
            drop(contexts);
            let playback_cleared = state.clear_active_playback_for_list(&ListContextId::History);
            if playback_cleared {
                emit_active_playback_cleared(&app, &ListContextId::History)?;
            }
            contexts = state.list_contexts.write();
        }
        
        let context = contexts.get_mut(&ListContextId::History).unwrap();
        context.id = ListContextId::History;
        context.items = results.clone();
        context.page = page;
        context.page_size = page_size;
        context.has_next = has_next;
        context.total_count = total;
        context.sort = Some(sort_config);
        // Note: version already bumped above if sort changed
        let current_version = context.version;
        
        // Also update legacy history_state
        let mut history_state = state.history_state.write();
        history_state.sort_direction = requested_sort.into();
        history_state.page = page;
        history_state.page_size = page_size;
        history_state.total_count = total;
        history_state.version = current_version;
    } else {
        // Append items for pagination
        let mut contexts = state.list_contexts.write();
        if let Some(context) = contexts.get_mut(&ListContextId::History) {
            context.items.extend(results.clone());
            context.page = page;
            context.has_next = has_next;
        }
    }

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
    let current = state.db.get_config().map_err(|e| e.to_string())?;
    let stored = crate::database::StoredConfig {
        query: config.query.clone(),
        max_age_days: config.max_age_days,
        targets: config.targets.clone(),
        category_filter: config.category_filter.clone(),
        auto_play: current.auto_play,
        auto_skip: current.auto_skip,
        skip_threshold: current.skip_threshold,
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
    // Read from active_playback
    let active_playback = state.active_playback.read();
    let current_index = active_playback.as_ref().map(|p| p.current_index);
    let playlist_type = active_playback.as_ref()
        .map(|p| PlaylistType::from(&p.list_id))
        .unwrap_or(PlaylistType::Search);
    drop(active_playback);

    let results = playlist_results_for_type(&state, playlist_type);
    let current_video = current_index.and_then(|index| results.get(index).cloned());
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

        if let (Some(video), Some(index)) = (current_video.clone(), current_index) {
            let has_next = index + 1 < results.len();
            window.emit("video-selected", VideoSelectedPayload {
                video,
                index,
                has_next,
                playlist_type,
                playlist_version: playlist_version_for_type(&state, playlist_type),
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

        if let (Some(video), Some(index)) = (current_video.clone(), current_index) {
            let has_next = index + 1 < results.len();
            window.emit("video-selected", VideoSelectedPayload {
                video,
                index,
                has_next,
                playlist_type,
                playlist_version: playlist_version_for_type(&state, playlist_type),
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

async fn run_playback_enrichment_reentry(
    app: &AppHandle,
    enrichment_request: PlaybackEnrichmentRequest,
) {
    let enriched_video = match enrichment_request.kind {
        PlaybackEnrichmentKind::FetchUserInfo => {
            let user_info = get_user_info(enrichment_request.video.id.clone())
                .await
                .ok()
                .flatten();
            resolve_playback_enrichment_video(&enrichment_request, None, user_info)
        }
        PlaybackEnrichmentKind::FetchFullVideoInfo => {
            let full_video = fetch_full_video_info(enrichment_request.video.id.clone())
                .await
                .ok();
            resolve_playback_enrichment_video(&enrichment_request, full_video, None)
        }
    };

    let state = app.state::<AppState>();
    if let Some(payload) = apply_playback_metadata_update(
        &state,
        &enrichment_request.list_id,
        enrichment_request.playlist_version,
        enrichment_request.index,
        enriched_video,
    ) {
        let _ = app.emit("playback-video-updated", payload);
    }
}

fn emit_video_selected_and_spawn_enrichment(
    app: &AppHandle,
    selected_payload: VideoSelectedPayload,
    enrichment_request: PlaybackEnrichmentRequest,
) -> Result<(), String> {
    app.emit("video-selected", selected_payload)
        .map_err(|e| e.to_string())?;

    let app_handle = app.clone();
    tokio::spawn(async move {
        run_playback_enrichment_reentry(&app_handle, enrichment_request).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn reenter_active_playback_metadata(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let Some((selected_payload, enrichment_request)) = build_active_playback_reentry_request(&state) else {
        return Ok(());
    };

    emit_video_selected_and_spawn_enrichment(&app, selected_payload, enrichment_request)
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
    // Read from active_playback (new model)
    let active_playback = state.active_playback.read();
    let Some(playback) = active_playback.as_ref() else {
        return Ok(None);
    };
    let playlist_type = PlaylistType::from(&playback.list_id);
    let current_index = playback.current_index;
    drop(active_playback);

    let results = playlist_results_for_type(&state, playlist_type);

    if current_index + 1 < results.len() {
        let next_index = current_index + 1;
        state.set_active_playback_index(next_index);
        Ok(Some(results[next_index].clone()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn play_previous(
    state: tauri::State<'_, AppState>,
) -> Result<Option<Video>, String> {
    // Read from active_playback (new model)
    let active_playback = state.active_playback.read();
    let Some(playback) = active_playback.as_ref() else {
        return Ok(None);
    };
    let playlist_type = PlaylistType::from(&playback.list_id);
    let current_index = playback.current_index;
    drop(active_playback);

    if current_index > 0 {
        let previous_index = current_index - 1;
        state.set_active_playback_index(previous_index);
        let results = playlist_results_for_type(&state, playlist_type);
        Ok(Some(results[previous_index].clone()))
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
pub fn get_storage_info(
    app: tauri::AppHandle,
) -> Result<StorageInfo, String> {
    use crate::database::{get_data_dir, get_db_path};

    let data_dir = get_data_dir(&app);
    let db_path = get_db_path(&app);
    let database_size_kb = std::fs::metadata(db_path)
        .ok()
        .map(|metadata| metadata.len() / 1024);

    Ok(StorageInfo {
        data_directory: data_dir.display().to_string(),
        database_size_kb,
    })
}

#[tauri::command]
pub async fn get_sync_preflight_estimate(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<SyncPreflightEstimate, String> {
    let config = state.config.read().clone();
    let estimated_video_count = crate::scraper_preflight::estimate_video_count(&config).await;

    use crate::database::{get_data_dir, get_db_path};
    let data_dir = get_data_dir(&app);
    let db_path = get_db_path(&app);
    let current_database_size_kb = std::fs::metadata(db_path)
        .ok()
        .map(|metadata| metadata.len() / 1024);
    let current_total_videos = state.db.get_total_videos().unwrap_or(0);

    let estimated_database_size_kb = crate::scraper_preflight::estimate_database_size_kb(
        estimated_video_count,
        current_database_size_kb,
        current_total_videos,
    );

    let free_space_kb = crate::scraper_preflight::lookup_free_space_kb(&data_dir);

    Ok(SyncPreflightEstimate {
        estimated_video_count,
        estimated_database_size_kb,
        free_space_kb,
    })
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


// ===== Watch Later Commands =====

#[tauri::command]
pub async fn add_to_watch_later(
    state: tauri::State<'_, AppState>,
    app: AppHandle,
    video_id: String,
    title: String,
    thumbnail_url: Option<String>,
) -> Result<(), String> {
    state.db.add_to_watch_later(&video_id, &title, thumbnail_url.as_deref())
        .map_err(|e| e.to_string())?;
    app.emit("watch-later-changed", video_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_from_watch_later(
    state: tauri::State<'_, AppState>,
    app: AppHandle,
    video_id: String,
) -> Result<(), String> {
    state.db.remove_from_watch_later(&video_id).map_err(|e| e.to_string())?;
    app.emit("watch-later-changed", video_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn is_in_watch_later(
    state: tauri::State<'_, AppState>,
    video_id: String,
) -> Result<bool, String> {
    state.db.is_in_watch_later(&video_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_watch_later(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    page: usize,
    page_size: usize,
    sort_direction: Option<String>,
) -> Result<WatchLaterResponse, String> {
    let total = state.db.get_watch_later_count().map_err(|e| e.to_string())?;
    let entries = state.db.get_watch_later(page, page_size, sort_direction.as_deref()).map_err(|e| e.to_string())?;
    
    let results_for_state: Vec<Video> = entries.iter().map(|entry| Video {
        id: entry.video_id.clone(),
        title: entry.title.clone(),
        thumbnail_url: entry.thumbnail_url.clone(),
        watch_url: None,
        view_count: 0,
        comment_count: 0,
        mylist_count: 0,
        like_count: 0,
        start_time: Some(entry.added_at.clone()),
        tags: vec![],
        duration: None,
        uploader_id: None,
        uploader_name: None,
        description: None,
        is_watched: false,
    }).collect();
    
    let has_next = (page * page_size) < total;
    let requested_sort: SortDirection = sort_direction
        .as_deref()
        .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
        .unwrap_or(SortDirection::Desc);
    
    // Update WatchLater list_context
    if page == 1 {
        let sort_config = crate::models::SortConfig {
            by: SortField::AddedAt,
            direction: requested_sort,
            weights: None,
        };
        
        let mut contexts = state.list_contexts.write();
        let context = contexts.entry(ListContextId::WatchLater).or_default();
        
        // Check if sort changed to bump version
        let should_bump = context.sort.as_ref().map(|s| s.direction != requested_sort).unwrap_or(true);
        if should_bump {
            context.version += 1;
            // Clear active playback if WatchLater was playing
            drop(contexts);
            let playback_cleared = state.clear_active_playback_for_list(&ListContextId::WatchLater);
            if playback_cleared {
                emit_active_playback_cleared(&app, &ListContextId::WatchLater)?;
            }
            contexts = state.list_contexts.write();
        }
        
        let context = contexts.get_mut(&ListContextId::WatchLater).unwrap();
        context.id = ListContextId::WatchLater;
        context.items = results_for_state.clone();
        context.page = page;
        context.page_size = page_size;
        context.has_next = has_next;
        context.total_count = total;
        context.sort = Some(sort_config);
        let current_version = context.version;
        
        // Also update legacy watch_later_state
        let mut watch_later_state = state.watch_later_state.write();
        watch_later_state.sort_direction = requested_sort.into();
        watch_later_state.page = page;
        watch_later_state.page_size = page_size;
        watch_later_state.total_count = total;
        watch_later_state.version = current_version;
    } else {
        // Append items for pagination
        let mut contexts = state.list_contexts.write();
        if let Some(context) = contexts.get_mut(&ListContextId::WatchLater) {
            context.items.extend(results_for_state.clone());
            context.page = page;
            context.has_next = has_next;
        }
    }

    Ok(WatchLaterResponse {
        total,
        page,
        page_size,
        has_next,
        results: entries,
    })
}

#[tauri::command]
pub async fn get_watch_later_count(
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    state.db.get_watch_later_count().map_err(|e| e.to_string())
}
// ===== State Management Commands =====

#[tauri::command]
pub async fn get_history_state(
    state: tauri::State<'_, AppState>,
) -> Result<HistoryState, String> {
    let mut history_state = state.history_state.read().clone();
    // Sync all relevant fields from list_context (the authoritative source)
    if let Some(list_context) = state.get_list_context(&ListContextId::History) {
        history_state.version = list_context.version;
        history_state.page = list_context.page;
        history_state.has_next = list_context.has_next;
        history_state.total_count = list_context.total_count;
        if let Some(ref sort) = list_context.sort {
            history_state.sort_direction = sort.direction.into();
        }
    }
    Ok(history_state)
}

#[tauri::command]
pub async fn set_history_state(
    app: AppHandle,
    history_state: HistoryState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut current = state.history_state.write();
        let version = current.version;
        *current = HistoryState { version, ..history_state };
    }
    app.emit("history-state-changed", &state.history_state.read().clone()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_watch_later_state(
    state: tauri::State<'_, AppState>,
) -> Result<WatchLaterState, String> {
    let mut watch_later_state = state.watch_later_state.read().clone();
    // Sync all relevant fields from list_context (the authoritative source)
    if let Some(list_context) = state.get_list_context(&ListContextId::WatchLater) {
        watch_later_state.version = list_context.version;
        watch_later_state.page = list_context.page;
        watch_later_state.has_next = list_context.has_next;
        watch_later_state.total_count = list_context.total_count;
        if let Some(ref sort) = list_context.sort {
            watch_later_state.sort_direction = sort.direction.into();
        }
    }
    Ok(watch_later_state)
}

#[tauri::command]
pub async fn set_watch_later_state(
    app: AppHandle,
    watch_later_state: WatchLaterState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut current = state.watch_later_state.write();
        let version = current.version;
        *current = WatchLaterState { version, ..watch_later_state };
    }
    app.emit("watch-later-state-changed", &state.watch_later_state.read().clone()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_playlist_type(
    state: tauri::State<'_, AppState>,
    playlist_type: PlaylistType,
) -> Result<(), String> {
    // Set the browsing list (creates/updates active_playback)
    let list_id = ListContextId::from(playlist_type);
    state.set_browsing_list(list_id);
    Ok(())
}

#[tauri::command]
pub async fn reset_playback_for_sync_route_entry(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.invalidate_search_playback_snapshot();

    if let Some(cleared_list_id) = state.clear_active_playback_with_list() {
        emit_active_playback_cleared(&app, &cleared_list_id)?;
    }

    Ok(())
}


// ===== Video Info Fetching =====

fn parse_thumbinfo_xml(xml: &str, video_id: &str) -> Result<ThumbInfo, String> {
    if let Some(error_code) = extract_xml_tag(xml, "code") {
        let error_description = extract_xml_tag(xml, "description")
            .unwrap_or_else(|| "Video not found or deleted".to_string());
        return Err(format!("getthumbinfo error ({}): {}", error_code, error_description));
    }

    let thumb_start = xml.find("<thumb>").ok_or_else(|| {
        let preview = xml.chars().take(200).collect::<String>();
        format!("Invalid getthumbinfo response for {}: {}", video_id, preview)
    })?;
    let thumb_end = xml.find("</thumb>").ok_or_else(|| {
        let preview = xml.chars().take(200).collect::<String>();
        format!("Invalid getthumbinfo response for {}: {}", video_id, preview)
    })?;
    let thumb_xml = &xml[thumb_start..thumb_end + "</thumb>".len()];

    let title = extract_xml_tag(thumb_xml, "title").ok_or_else(|| "Video not found or deleted".to_string())?;

    Ok(ThumbInfo {
        id: video_id.to_string(),
        title,
        description: extract_xml_tag(thumb_xml, "description"),
        thumbnail_url: extract_xml_tag(thumb_xml, "thumbnail_url"),
        first_retrieve: extract_xml_tag(thumb_xml, "first_retrieve"),
        length: extract_xml_tag(thumb_xml, "length"),
        view_counter: extract_xml_tag(thumb_xml, "view_counter").and_then(|s| s.parse().ok()),
        comment_num: extract_xml_tag(thumb_xml, "comment_num").and_then(|s| s.parse().ok()),
        mylist_counter: extract_xml_tag(thumb_xml, "mylist_counter").and_then(|s| s.parse().ok()),
        tags: Some(extract_xml_tags(thumb_xml)),
        user_id: extract_xml_tag(thumb_xml, "user_id"),
        user_nickname: extract_xml_tag(thumb_xml, "user_nickname"),
    })
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let start = xml.find(&start_tag)? + start_tag.len();
    let end = xml[start..].find(&end_tag)? + start;
    let value = &xml[start..end];
    quick_xml::escape::unescape(value).ok().map(|v| v.into_owned())
}

fn extract_xml_tags(xml: &str) -> Vec<String> {
    let tags_start = match xml.find("<tags") {
        Some(i) => i,
        None => return Vec::new(),
    };
    let tags_open_end = match xml[tags_start..].find('>') {
        Some(i) => tags_start + i + 1,
        None => return Vec::new(),
    };
    let tags_end = match xml[tags_open_end..].find("</tags>") {
        Some(i) => tags_open_end + i,
        None => return Vec::new(),
    };
    let body = &xml[tags_open_end..tags_end];
    let mut out = Vec::new();
    let mut rest = body;

    while let Some(start) = rest.find("<tag") {
        let after_open = match rest[start..].find('>') {
            Some(i) => start + i + 1,
            None => break,
        };
        let close = match rest[after_open..].find("</tag>") {
            Some(i) => after_open + i,
            None => break,
        };
        let value = &rest[after_open..close];
        if let Ok(unescaped) = quick_xml::escape::unescape(value) {
            out.push(unescaped.into_owned());
        }
        rest = &rest[close + "</tag>".len()..];
    }

    out
}

struct ThumbInfo {
    id: String,
    title: String,
    description: Option<String>,
    thumbnail_url: Option<String>,
    first_retrieve: Option<String>,
    length: Option<String>,
    view_counter: Option<i64>,
    comment_num: Option<i64>,
    mylist_counter: Option<i64>,
    tags: Option<Vec<String>>,
    user_id: Option<String>,
    user_nickname: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_default_sort() -> SortConfig {
        SortConfig {
            by: SortField::View,
            direction: SortDirection::Desc,
            weights: None,
        }
    }

    #[test]
    fn build_query_without_filters() {
        let request = SearchRequest {
            query: String::new(),
            page: 1,
            page_size: 50,
            exclude_watched: false,
            filters: None,
            sort: Some(make_default_sort()),
            formula_filter: None,
        };
        
        let (sql, params, count_sql) = build_search_query(&request, &[]);
        
        assert!(sql.contains("SELECT v.id"));
        assert!(sql.contains("FROM videos v"));
        assert!(!sql.contains("WHERE"));
        assert!(params.is_empty());
        assert!(count_sql.contains("COUNT(*)"));
    }

    #[test]
    fn build_query_with_search_query() {
        let request = SearchRequest {
            query: "VOCALOID".to_string(),
            page: 1,
            page_size: 50,
            exclude_watched: false,
            filters: None,
            sort: Some(make_default_sort()),
            formula_filter: None,
        };
        
        let (sql, params, _count_sql) = build_search_query(&request, &[]);
        
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("v.title LIKE ? OR v.tags LIKE ?"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn build_query_with_view_filter() {
        let request = SearchRequest {
            query: String::new(),
            page: 1,
            page_size: 50,
            exclude_watched: false,
            filters: Some(Filters {
                view: Some(NumericFilter { gte: Some(1000.0), lte: None }),
                ..Default::default()
            }),
            sort: Some(make_default_sort()),
            formula_filter: None,
        };
        
        let (sql, params, _count_sql) = build_search_query(&request, &[]);
        
        assert!(sql.contains("v.view_count >= ?"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn build_query_with_date_filter() {
        let request = SearchRequest {
            query: String::new(),
            page: 1,
            page_size: 50,
            exclude_watched: false,
            filters: Some(Filters {
                start_time: Some(DateFilter { 
                    gte: Some("2024-01-01".to_string()), 
                    lte: None 
                }),
                ..Default::default()
            }),
            sort: Some(make_default_sort()),
            formula_filter: None,
        };
        
        let (sql, params, _count_sql) = build_search_query(&request, &[]);
        
        assert!(sql.contains("v.start_time >= ?"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn build_query_with_sort_by_mylist() {
        let request = SearchRequest {
            query: String::new(),
            page: 1,
            page_size: 50,
            exclude_watched: false,
            filters: None,
            sort: Some(SortConfig {
                by: SortField::Mylist,
                direction: SortDirection::Asc,
                weights: None,
            }),
            formula_filter: None,
        };
        
        let (sql, _params, _count_sql) = build_search_query(&request, &[]);
        
        assert!(sql.contains("ORDER BY v.mylist_count ASC"));
    }

    #[test]
    fn build_query_with_custom_sort_with_weights() {
        let request = SearchRequest {
            query: String::new(),
            page: 1,
            page_size: 50,
            exclude_watched: false,
            filters: None,
            sort: Some(SortConfig {
                by: SortField::Custom,
                direction: SortDirection::Desc,
                weights: Some(SortWeights {
                    view: 0.5,
                    mylist: 0.3,
                    comment: 0.1,
                    like: 0.1,
                }),
            }),
            formula_filter: None,
        };
        
        let (sql, _params, _count_sql) = build_search_query(&request, &[]);
        
        assert!(sql.contains("ORDER BY"));
        assert!(sql.contains("v.view_count"));
        assert!(sql.contains("v.mylist_count"));
    }

    #[test]
    fn build_query_with_exclude_watched() {
        let request = SearchRequest {
            query: String::new(),
            page: 1,
            page_size: 50,
            exclude_watched: true,
            filters: None,
            sort: Some(make_default_sort()),
            formula_filter: None,
        };
        
        let watched_ids = vec!["sm123".to_string(), "sm456".to_string()];
        let (sql, params, _count_sql) = build_search_query(&request, &watched_ids);
        
        assert!(sql.contains("v.id NOT IN"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn build_query_with_stable_tie_breaker_for_like_sort() {
        let request = SearchRequest {
            query: String::new(),
            page: 1,
            page_size: 50,
            exclude_watched: false,
            filters: None,
            sort: Some(SortConfig {
                by: SortField::Like,
                direction: SortDirection::Desc,
                weights: None,
            }),
            formula_filter: None,
        };
        
        let (sql, _params, _count_sql) = build_search_query(&request, &[]);
        
        assert!(sql.contains("ORDER BY v.like_count DESC, v.id DESC"), 
            "ORDER BY should include deterministic tie-breaker: {}", sql);
    }

    #[test]
    fn build_query_with_stable_tie_breaker_for_view_sort_asc() {
        let request = SearchRequest {
            query: String::new(),
            page: 1,
            page_size: 50,
            exclude_watched: false,
            filters: None,
            sort: Some(SortConfig {
                by: SortField::View,
                direction: SortDirection::Asc,
                weights: None,
            }),
            formula_filter: None,
        };
        
        let (sql, _params, _count_sql) = build_search_query(&request, &[]);
        
        assert!(sql.contains("ORDER BY v.view_count ASC, v.id ASC"), 
            "ORDER BY should include deterministic tie-breaker with same direction: {}", sql);
    }

    #[test]
    fn build_query_with_stable_tie_breaker_for_custom_sort() {
        let request = SearchRequest {
            query: String::new(),
            page: 1,
            page_size: 50,
            exclude_watched: false,
            filters: None,
            sort: Some(SortConfig {
                by: SortField::Custom,
                direction: SortDirection::Desc,
                weights: Some(SortWeights {
                    view: 5.0,
                    mylist: 3.0,
                    comment: 2.0,
                    like: 1.0,
                }),
            }),
            formula_filter: None,
        };

        let (sql, _params, _count_sql) = build_search_query(&request, &[]);

        assert!(sql.contains("v.id DESC"),
            "ORDER BY for custom sort should include v.id DESC tie-breaker: {}", sql);
    }

    #[test]
    fn applies_playback_metadata_update_and_returns_payload_for_active_playback() {
        let test = TestAppState::new();
        let history_items = vec![sample_video("sm1"), sample_video("sm2"), sample_video("sm9")];
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
        test.state
            .set_active_playback(ListContextId::History, history_version, 2);
        let updated_video = Video {
            title: "updated title".to_string(),
            ..sample_video("sm9")
        };

        let payload = apply_playback_metadata_update(
            &test.state,
            &ListContextId::History,
            history_version,
            2,
            updated_video.clone(),
        );

        assert_eq!(
            test.state.get_list_context_items(&ListContextId::History)[2].title,
            "updated title"
        );
        assert_eq!(
            payload,
            Some(PlaybackVideoUpdatedPayload::new(
                ListContextId::History,
                history_version,
                2,
                updated_video,
            ))
        );
    }

    #[test]
    fn applies_playback_metadata_update_without_payload_for_stale_playback() {
        let test = TestAppState::new();
        let history_items = vec![sample_video("sm1"), sample_video("sm2"), sample_video("sm9")];
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
        test.state
            .set_active_playback(ListContextId::History, history_version, 2);
        let updated_video = Video {
            title: "updated title".to_string(),
            ..sample_video("sm9")
        };

        let payload = apply_playback_metadata_update(
            &test.state,
            &ListContextId::History,
            history_version,
            1,
            updated_video,
        );

        assert_eq!(
            test.state.get_list_context_items(&ListContextId::History)[1].title,
            "updated title"
        );
        assert!(payload.is_none());
    }

    #[test]
    fn skips_playback_metadata_update_payload_for_stale_identity() {
        let test = TestAppState::new();
        let history_items = vec![sample_video("sm1"), sample_video("sm2"), sample_video("sm9")];
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
        test.state
            .set_active_playback(ListContextId::History, history_version, 2);

        let payload = build_playback_metadata_update_payload(
            &test.state,
            &ListContextId::History,
            history_version + 1,
            2,
            sample_video("sm9"),
        );

        assert!(payload.is_none());
    }

    #[test]
    fn search_selection_uses_user_info_enrichment_strategy() {
        let request = build_playback_enrichment_request(
            ListContextId::Search,
            4,
            1,
            sample_video("sm9"),
        );

        assert_eq!(request.kind, PlaybackEnrichmentKind::FetchUserInfo);
        assert_eq!(request.playlist_version, 4);
        assert_eq!(request.index, 1);
        assert_eq!(request.video.id, "sm9");
    }

    #[test]
    fn history_selection_uses_full_video_enrichment_strategy() {
        let request = build_playback_enrichment_request(
            ListContextId::History,
            7,
            2,
            sample_video("sm9"),
        );

        assert_eq!(request.kind, PlaybackEnrichmentKind::FetchFullVideoInfo);
        assert_eq!(request.playlist_version, 7);
        assert_eq!(request.index, 2);
        assert_eq!(request.video.id, "sm9");
    }

    #[test]
    fn merge_user_info_into_video_updates_uploader_fields_only() {
        let merged = merge_user_info_into_video(
            sample_video("sm9"),
            Some(UserInfo {
                user_id: Some("42".to_string()),
                user_nickname: Some("MikuP".to_string()),
                user_icon_url: Some("https://example.com/icon.jpg".to_string()),
            }),
        );

        assert_eq!(merged.uploader_id.as_deref(), Some("42"));
        assert_eq!(merged.uploader_name.as_deref(), Some("MikuP"));
        assert_eq!(merged.title, "title-sm9");
        assert_eq!(merged.like_count, 4);
    }

    #[test]
    fn merge_user_info_into_video_keeps_existing_fields_when_user_info_missing() {
        let original = sample_video("sm9");
        let merged = merge_user_info_into_video(original.clone(), None);

        assert_eq!(merged, original);
    }

    #[test]
    fn resolve_search_enrichment_video_merges_user_info_without_snapshot_refetch() {
        let request = build_playback_enrichment_request(
            ListContextId::Search,
            4,
            1,
            sample_video("sm9"),
        );

        let resolved = resolve_playback_enrichment_video(
            &request,
            Some(Video {
                title: "should not win".to_string(),
                ..sample_video("sm9")
            }),
            Some(UserInfo {
                user_id: Some("42".to_string()),
                user_nickname: Some("MikuP".to_string()),
                user_icon_url: Some("https://example.com/icon.jpg".to_string()),
            }),
        );

        assert_eq!(resolved.title, "title-sm9");
        assert_eq!(resolved.uploader_id.as_deref(), Some("42"));
        assert_eq!(resolved.uploader_name.as_deref(), Some("MikuP"));
    }

    #[test]
    fn resolve_non_search_enrichment_video_prefers_full_video_and_falls_back_to_placeholder() {
        let request = build_playback_enrichment_request(
            ListContextId::History,
            7,
            2,
            sample_video("sm9"),
        );
        let full_video = Video {
            title: "full info title".to_string(),
            like_count: 99,
            ..sample_video("sm9")
        };

        let resolved = resolve_playback_enrichment_video(&request, Some(full_video.clone()), None);
        let fallback = resolve_playback_enrichment_video(&request, None, None);

        assert_eq!(resolved, full_video);
        assert_eq!(fallback, request.video);
    }

    #[test]
    fn build_full_video_from_thumbinfo_prefers_thumbinfo_base_and_snapshot_like_count() {
        let video = build_full_video_from_thumbinfo(
            "sm9",
            sample_thumb_info(),
            Some(SnapshotVideo {
                contentId: "sm9".to_string(),
                title: "snapshot title should not win".to_string(),
                thumbnailUrl: serde_json::json!("https://example.com/snapshot.jpg"),
                viewCounter: Some(999),
                commentCounter: Some(888),
                mylistCounter: Some(777),
                likeCounter: Some(66),
                startTime: Some("2024-02-02T00:00:00+09:00".to_string()),
                tags: Some(serde_json::json!(["snapshot"])),
                lengthSeconds: Some(456),
                genre: None,
                description: Some("snapshot desc".to_string()),
                userId: Some("snapshot-user".to_string()),
            }),
        );

        assert_eq!(video.title, "thumb title");
        assert_eq!(video.like_count, 66);
        assert_eq!(video.view_count, 123);
        assert_eq!(video.comment_count, 45);
        assert_eq!(video.mylist_count, 6);
        assert_eq!(video.uploader_id.as_deref(), Some("42"));
        assert_eq!(video.uploader_name.as_deref(), Some("MikuP"));
        assert_eq!(video.duration, Some(83));
    }

    #[test]
    fn build_full_video_from_thumbinfo_falls_back_when_snapshot_is_missing() {
        let video = build_full_video_from_thumbinfo("sm9", sample_thumb_info(), None);

        assert_eq!(video.title, "thumb title");
        assert_eq!(video.like_count, 0);
        assert_eq!(video.view_count, 123);
        assert_eq!(video.comment_count, 45);
        assert_eq!(video.mylist_count, 6);
        assert_eq!(video.duration, Some(83));
    }

    #[tokio::test]
    async fn fetch_non_search_enrichment_video_runs_source_fetches_in_parallel() {
        let started = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let thumb_started = started.clone();
        let snapshot_started = started.clone();
        let started_for_thumb_assert = started.clone();
        let started_for_snapshot_assert = started.clone();

        let now = std::time::Instant::now();
        let video = fetch_non_search_enrichment_video(
            "sm9",
            move || async move {
                thumb_started.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                assert_eq!(
                    started_for_thumb_assert.load(std::sync::atomic::Ordering::SeqCst),
                    2
                );
                Ok(sample_thumb_info())
            },
            move || async move {
                snapshot_started.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                assert_eq!(
                    started_for_snapshot_assert.load(std::sync::atomic::Ordering::SeqCst),
                    2
                );
                Some(SnapshotVideo {
                    contentId: "sm9".to_string(),
                    title: "snapshot".to_string(),
                    thumbnailUrl: serde_json::json!("https://example.com/snapshot.jpg"),
                    viewCounter: Some(999),
                    commentCounter: Some(888),
                    mylistCounter: Some(777),
                    likeCounter: Some(66),
                    startTime: Some("2024-02-02T00:00:00+09:00".to_string()),
                    tags: Some(serde_json::json!(["snapshot"])),
                    lengthSeconds: Some(456),
                    genre: None,
                    description: Some("snapshot desc".to_string()),
                    userId: Some("snapshot-user".to_string()),
                })
            },
        )
        .await
        .unwrap();

        assert!(now.elapsed() < std::time::Duration::from_millis(70));
        assert_eq!(video.title, "thumb title");
        assert_eq!(video.like_count, 66);
    }

    #[tokio::test]
    async fn fetch_non_search_enrichment_video_keeps_thumbinfo_when_snapshot_missing() {
        let video = fetch_non_search_enrichment_video(
            "sm9",
            || async move { Ok(sample_thumb_info()) },
            || async move { None },
        )
        .await
        .unwrap();

        assert_eq!(video.title, "thumb title");
        assert_eq!(video.like_count, 0);
        assert_eq!(video.uploader_name.as_deref(), Some("MikuP"));
    }

    struct TestAppState {
        state: AppState,
        _root: std::path::PathBuf,
    }

    impl TestAppState {
        fn new() -> Self {
            use std::time::{SystemTime, UNIX_EPOCH};

            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("commands-test-{}", unique));
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

    fn sample_thumb_info() -> ThumbInfo {
        ThumbInfo {
            id: "sm9".to_string(),
            title: "thumb title".to_string(),
            description: Some("thumb desc".to_string()),
            thumbnail_url: Some("https://example.com/thumb.jpg".to_string()),
            first_retrieve: Some("2024-01-01T00:00:00+09:00".to_string()),
            length: Some("01:23".to_string()),
            view_counter: Some(123),
            comment_num: Some(45),
            mylist_counter: Some(6),
            tags: Some(vec!["vocaloid".to_string(), "miku".to_string()]),
            user_id: Some("42".to_string()),
            user_nickname: Some("MikuP".to_string()),
        }
    }
}
