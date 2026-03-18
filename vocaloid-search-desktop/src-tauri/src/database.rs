use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

/// Schema for videos.db - contains only video metadata cache (rebuildable)
const VIDEOS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS videos (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    thumbnail_url TEXT,
    watch_url TEXT,
    view_count INTEGER DEFAULT 0,
    comment_count INTEGER DEFAULT 0,
    mylist_count INTEGER DEFAULT 0,
    like_count INTEGER DEFAULT 0,
    start_time TEXT,
    tags TEXT,
    duration INTEGER,
    category TEXT,
    description TEXT,
    uploader_id TEXT,
    uploader_name TEXT,
    last_update_time TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_view_count ON videos(view_count);
CREATE INDEX IF NOT EXISTS idx_mylist_count ON videos(mylist_count);
CREATE INDEX IF NOT EXISTS idx_comment_count ON videos(comment_count);
CREATE INDEX IF NOT EXISTS idx_like_count ON videos(like_count);
CREATE INDEX IF NOT EXISTS idx_start_time ON videos(start_time);

CREATE VIRTUAL TABLE IF NOT EXISTS video_fts USING fts5(
    title,
    tags,
    content='videos',
    content_rowid='rowid',
    tokenize='unicode61'
);

CREATE TRIGGER IF NOT EXISTS videos_ai AFTER INSERT ON videos BEGIN
    INSERT INTO video_fts(rowid, title, tags) 
    VALUES (new.rowid, new.title, COALESCE(new.tags, ''));
END;

CREATE TRIGGER IF NOT EXISTS videos_ad AFTER DELETE ON videos BEGIN
    INSERT INTO video_fts(video_fts, rowid, title, tags) 
    VALUES('delete', old.rowid, old.title, COALESCE(old.tags, ''));
END;

CREATE TRIGGER IF NOT EXISTS videos_au AFTER UPDATE ON videos BEGIN
    INSERT INTO video_fts(video_fts, rowid, title, tags) 
    VALUES('delete', old.rowid, old.title, COALESCE(old.tags, ''));
    INSERT INTO video_fts(rowid, title, tags) 
    VALUES (new.rowid, new.title, COALESCE(new.tags, ''));
END;
"#;

/// Schema for user_data.db - contains user-generated data (history, watch_later, config)
const USER_DATA_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS history (
    video_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    thumbnail_url TEXT,
    watched_at TEXT DEFAULT CURRENT_TIMESTAMP,
    first_watched_seq INTEGER,
    first_watched_at TEXT
);

CREATE TABLE IF NOT EXISTS watch_later (
    video_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    thumbnail_url TEXT,
    added_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT
);
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConfig {
    pub query: String,
    pub max_age_days: Option<i64>,
    pub targets: String,
    pub category_filter: Option<String>,
    pub auto_play: bool,
    pub auto_skip: bool,
    pub skip_threshold: u32,
}

impl Default for StoredConfig {
    fn default() -> Self {
        Self {
            query: "VOCALOID".to_string(),
            max_age_days: Some(365),
            targets: "tags".to_string(),
            category_filter: Some("MUSIC".to_string()),
            auto_play: true,
            auto_skip: false,
            skip_threshold: 30,
        }
    }
}

pub fn get_data_dir(app: &tauri::AppHandle) -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let portable_data = exe_dir.join("data");

    if portable_data.exists() && portable_data.is_dir() {
        portable_data
    } else {
        app.path()
            .app_data_dir()
            .expect("Failed to get app data dir")
    }
}

pub fn get_videos_db_path(app: &tauri::AppHandle) -> PathBuf {
    get_data_dir(app).join("videos.db")
}

pub fn get_user_data_db_path(app: &tauri::AppHandle) -> PathBuf {
    get_data_dir(app).join("user_data.db")
}

/// Legacy function for backward compatibility - returns videos.db path
pub fn get_db_path(app: &tauri::AppHandle) -> PathBuf {
    get_videos_db_path(app)
}

pub fn get_config_path(app: &tauri::AppHandle) -> PathBuf {
    get_data_dir(app).join("config.json")
}

pub fn init_db(videos_path: &PathBuf, user_data_path: &PathBuf) -> Result<(), rusqlite::Error> {
    // Initialize videos.db
    let videos_conn = Connection::open(videos_path)?;
    videos_conn.execute_batch(VIDEOS_SCHEMA)?;
    videos_conn.pragma_update(None, "journal_mode", "WAL")?;
    videos_conn.pragma_update(None, "synchronous", "NORMAL")?;
    videos_conn.pragma_update(None, "cache_size", -64000)?;

    // Initialize user_data.db
    let user_data_conn = Connection::open(user_data_path)?;
    user_data_conn.execute_batch(USER_DATA_SCHEMA)?;
    migrate_user_data_schema(&user_data_conn)?;
    user_data_conn.pragma_update(None, "journal_mode", "WAL")?;
    user_data_conn.pragma_update(None, "synchronous", "NORMAL")?;
    user_data_conn.pragma_update(None, "cache_size", -64000)?;

    Ok(())
}

fn migrate_user_data_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    add_history_column_if_missing(conn, "first_watched_seq", "INTEGER")?;
    add_history_column_if_missing(conn, "first_watched_at", "TEXT")?;
    backfill_history_first_watch_metadata(conn)?;
    Ok(())
}

fn add_history_column_if_missing(
    conn: &Connection,
    column_name: &str,
    column_definition: &str,
) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare("PRAGMA table_info(history)")?;
    let existing_columns: Vec<String> = stmt
        .query_map([], |row| row.get(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if existing_columns.iter().any(|existing| existing == column_name) {
        return Ok(());
    }

    conn.execute(
        &format!("ALTER TABLE history ADD COLUMN {} {}", column_name, column_definition),
        [],
    )?;
    Ok(())
}

fn backfill_history_first_watch_metadata(conn: &Connection) -> Result<(), rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "UPDATE history SET first_watched_at = watched_at WHERE first_watched_at IS NULL",
        [],
    )?;

    let max_seq: i64 = tx.query_row(
        "SELECT COALESCE(MAX(first_watched_seq), 0) FROM history",
        [],
        |row| row.get(0),
    )?;

    let mut next_seq = max_seq;
    let mut stmt = tx.prepare(
        "SELECT video_id FROM history WHERE first_watched_seq IS NULL ORDER BY first_watched_at ASC, watched_at ASC, video_id ASC",
    )?;
    let pending_ids: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    for video_id in pending_ids {
        next_seq += 1;
        tx.execute(
            "UPDATE history SET first_watched_seq = ? WHERE video_id = ?",
            params![next_seq, video_id],
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub struct Database {
    videos_path: Arc<PathBuf>,
    user_data_path: Arc<PathBuf>,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            videos_path: Arc::clone(&self.videos_path),
            user_data_path: Arc::clone(&self.user_data_path),
        }
    }
}

impl Database {
    pub fn new(videos_path: PathBuf, user_data_path: PathBuf) -> Self {
        Self {
            videos_path: Arc::new(videos_path),
            user_data_path: Arc::new(user_data_path),
        }
    }

    /// Connect to videos.db for video metadata operations
    pub fn connect_videos(&self) -> Result<Connection, rusqlite::Error> {
        let conn = Connection::open(&*self.videos_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", -64000)?;
        conn.pragma_update(None, "mmap_size", 268435456)?;
        Ok(conn)
    }
    /// Connect to user_data.db for user data operations (history, watch_later, config)
    pub fn connect_user_data(&self) -> Result<Connection, rusqlite::Error> {
        let conn = Connection::open(&*self.user_data_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", -64000)?;
        Ok(conn)
    }
    /// Legacy connect method - returns videos.db connection for backward compatibility
    pub fn connect(&self) -> Result<Connection, rusqlite::Error> {
        self.connect_videos()
    }

    pub fn get_total_videos(&self) -> Result<usize, rusqlite::Error> {
        let conn = self.connect()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM videos", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_last_update(&self) -> Result<Option<String>, rusqlite::Error> {
        let conn = self.connect()?;
        let result: Option<String> = conn
            .query_row("SELECT MAX(last_update_time) FROM videos", [], |row| {
                row.get(0)
            })
            .ok();
        Ok(result)
    }

    pub fn is_video_watched(&self, video_id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        let exists: i64 = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM history WHERE video_id = ?)",
            [video_id],
            |row| row.get(0),
        )?;
        Ok(exists == 1)
    }

    pub fn get_all_watched_video_ids(&self) -> Result<Vec<String>, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        let mut stmt = conn.prepare("SELECT video_id FROM history")?;
        let ids: Vec<String> = stmt.query_map([], |row| row.get(0))?.collect::<Result<Vec<_>, _>>()?;
        Ok(ids)
    }

    /// Get all watched video IDs with first_watched_seq <= boundary_seq
    /// Used for frozen watched boundary in Search playback
    pub fn get_watched_ids_up_to_boundary(&self, boundary_seq: i64) -> Result<Vec<String>, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        migrate_user_data_schema(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT video_id FROM history WHERE first_watched_seq <= ?"
        )?;
        let ids: Vec<String> = stmt
            .query_map([boundary_seq], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ids)
    }

    /// Get the maximum first_watched_seq from history
    /// Used to determine the frozen boundary when creating Search playback snapshot
    pub fn get_max_first_watched_seq(&self) -> Result<i64, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        migrate_user_data_schema(&conn)?;
        let max_seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(first_watched_seq), 0) FROM history",
                [],
                |row| row.get(0),
            )?;
        Ok(max_seq)
    }

    pub fn mark_watched(
        &self,
        video_id: &str,
        title: &str,
        thumbnail_url: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let mut user_conn = self.connect_user_data()?;
        migrate_user_data_schema(&user_conn)?;
        let videos_conn = self.connect_videos()?;

        let is_bad_title = title.trim().is_empty() || title == "ニコニコ動画";
        let existing: Option<(String, Option<String>)> = user_conn
            .query_row(
                "SELECT title, thumbnail_url FROM history WHERE video_id = ?",
                [video_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();
        let from_videos: Option<(String, Option<String>)> = videos_conn
            .query_row(
                "SELECT title, thumbnail_url FROM videos WHERE id = ?",
                [video_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        let final_title = if !is_bad_title {
            title.to_string()
        } else if let Some((db_title, _)) = &from_videos {
            db_title.clone()
        } else if let Some((existing_title, _)) = &existing {
            existing_title.clone()
        } else {
            title.to_string()
        };

        let final_thumbnail = thumbnail_url
            .map(|s| s.to_string())
            .or_else(|| from_videos.as_ref().and_then(|(_, thumb)| thumb.clone()))
            .or_else(|| existing.as_ref().and_then(|(_, thumb)| thumb.clone()));

        let tx = user_conn.transaction()?;
        let current_seq: Option<i64> = tx
            .query_row(
                "SELECT first_watched_seq FROM history WHERE video_id = ?",
                [video_id],
                |row| row.get(0),
            )
            .ok();
        let first_watched_seq = match current_seq {
            Some(seq) => seq,
            None => tx.query_row(
                "SELECT COALESCE(MAX(first_watched_seq), 0) + 1 FROM history",
                [],
                |row| row.get(0),
            )?,
        };

        tx.execute(
            "INSERT INTO history (
                video_id,
                title,
                thumbnail_url,
                watched_at,
                first_watched_seq,
                first_watched_at
            ) VALUES (?, ?, ?, datetime('now', '+9 hours'), ?, datetime('now', '+9 hours'))
            ON CONFLICT(video_id) DO UPDATE SET
                title = excluded.title,
                thumbnail_url = excluded.thumbnail_url,
                watched_at = excluded.watched_at,
                first_watched_seq = history.first_watched_seq,
                first_watched_at = history.first_watched_at",
            params![video_id, final_title, final_thumbnail, first_watched_seq],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_history(
        &self,
        page: usize,
        page_size: usize,
        sort_direction: Option<&str>,
    ) -> Result<Vec<crate::models::HistoryEntry>, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        let videos_conn = self.connect_videos()?;
        let offset = (page - 1) * page_size;

        let order = match sort_direction {
            Some("asc") => "ASC",
            _ => "DESC",
        };
        let sql = format!(
            "SELECT video_id, title, thumbnail_url, watched_at FROM history ORDER BY watched_at {} LIMIT ? OFFSET ?",
            order
        );

        let mut stmt = conn.prepare(&sql)?;

        let entries: Vec<crate::models::HistoryEntry> = stmt
            .query_map([page_size as i64, offset as i64], |row| {
                let video_id: String = row.get(0)?;
                let stored_title: String = row.get(1)?;
                let stored_thumbnail: Option<String> = row.get(2)?;
                let watched_at: String = row.get(3)?;
                
                let from_videos: Option<(String, Option<String>)> = videos_conn
                    .query_row(
                        "SELECT title, thumbnail_url FROM videos WHERE id = ?",
                        [&video_id],
                        |video_row| Ok((video_row.get(0)?, video_row.get(1)?)),
                    )
                    .ok();
                
                let title = if stored_title.trim().is_empty() || stored_title == "ニコニコ動画" {
                    from_videos.as_ref().map(|(t, _)| t.clone()).unwrap_or(stored_title)
                } else {
                    stored_title
                };
                let thumbnail_url = stored_thumbnail
                    .or_else(|| from_videos.as_ref().and_then(|(_, thumb)| thumb.clone()));
                
                Ok(crate::models::HistoryEntry {
                    video_id,
                    title,
                    thumbnail_url,
                    watched_at,
                })
            })?
            .filter_map(|e| e.ok())
            .collect();

        Ok(entries)
    }

    pub fn get_history_count(&self) -> Result<usize, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn clear_videos(&self) -> Result<(), rusqlite::Error> {
        let conn = self.connect_videos()?;
        conn.execute("DELETE FROM videos", [])?;
        Ok(())
    }

    pub fn insert_videos_batch(
        &self,
        videos: &[(
            String,
            String,
            Option<String>,
            Option<String>,
            i64,
            i64,
            i64,
            i64,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )],
    ) -> Result<(), rusqlite::Error> {
        let mut conn = self.connect_videos()?;
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO videos 
                (id, title, thumbnail_url, watch_url, view_count, comment_count, 
                 mylist_count, like_count, start_time, tags, duration, category, 
                 description, uploader_id, uploader_name, last_update_time)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '+9 hours'))",
            )?;

            for video in videos {
                stmt.execute(params![
                    video.0, video.1, video.2, video.3, video.4, video.5, video.6, video.7,
                    video.8, video.9, video.10, video.11, video.12, video.13, video.14
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_config(&self) -> Result<StoredConfig, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        let result: Option<String> = conn
            .query_row(
                "SELECT value FROM config WHERE key = 'scraper'",
                [],
                |row| row.get(0),
            )
            .ok();

        let config = match result {
            Some(json) => serde_json::from_str(&json).unwrap_or_else(|_| StoredConfig::default()),
            None => StoredConfig::default(),
        };

        Ok(config)
    }

    pub fn save_config(&self, config: &StoredConfig) -> Result<(), rusqlite::Error> {
        let conn = self.connect_user_data()?;
        let json = serde_json::to_string(config).unwrap_or_else(|_| "{}".to_string());
        conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES ('scraper', ?)",
            [&json],
        )?;
        Ok(())
    }

    // ===== Watch Later Methods =====

    pub fn add_to_watch_later(
        &self,
        video_id: &str,
        title: &str,
        thumbnail_url: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let user_conn = self.connect_user_data()?;
        let videos_conn = self.connect_videos()?;
        
        let is_bad_title = title.trim().is_empty() || title == "ニコニコ動画";
        let existing: Option<(String, Option<String>)> = user_conn
            .query_row(
                "SELECT title, thumbnail_url FROM watch_later WHERE video_id = ?",
                [video_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();
        let from_videos: Option<(String, Option<String>)> = videos_conn
            .query_row(
                "SELECT title, thumbnail_url FROM videos WHERE id = ?",
                [video_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();
        
        let final_title = if !is_bad_title {
            title.to_string()
        } else if let Some((db_title, _)) = &from_videos {
            db_title.clone()
        } else if let Some((existing_title, _)) = &existing {
            existing_title.clone()
        } else {
            title.to_string()
        };
        
        let final_thumbnail = thumbnail_url
            .map(|s| s.to_string())
            .or_else(|| from_videos.as_ref().and_then(|(_, thumb)| thumb.clone()))
            .or_else(|| existing.as_ref().and_then(|(_, thumb)| thumb.clone()));
        
        user_conn.execute(
            "INSERT OR REPLACE INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, datetime('now', '+9 hours'))",
            params![video_id, final_title, final_thumbnail],
        )?;
        Ok(())
    }

    pub fn remove_from_watch_later(&self, video_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.connect_user_data()?;
        conn.execute("DELETE FROM watch_later WHERE video_id = ?", [video_id])?;
        Ok(())
    }

    pub fn is_in_watch_later(&self, video_id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        let exists: i64 = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM watch_later WHERE video_id = ?)",
            [video_id],
            |row| row.get(0),
        )?;
        Ok(exists == 1)
    }

    pub fn get_watch_later(
        &self,
        page: usize,
        page_size: usize,
        sort_direction: Option<&str>,
    ) -> Result<Vec<crate::models::WatchLaterEntry>, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        let videos_conn = self.connect_videos()?;
        let offset = (page - 1) * page_size;

        let order = match sort_direction {
            Some("asc") => "ASC",
            _ => "DESC",
        };
        let sql = format!(
            "SELECT video_id, title, thumbnail_url, added_at FROM watch_later ORDER BY added_at {} LIMIT ? OFFSET ?",
            order
        );

        let mut stmt = conn.prepare(&sql)?;

        let entries: Vec<crate::models::WatchLaterEntry> = stmt
            .query_map([page_size as i64, offset as i64], |row| {
                let video_id: String = row.get(0)?;
                let stored_title: String = row.get(1)?;
                let stored_thumbnail: Option<String> = row.get(2)?;
                let added_at: String = row.get(3)?;
                
                let from_videos: Option<(String, Option<String>)> = videos_conn
                    .query_row(
                        "SELECT title, thumbnail_url FROM videos WHERE id = ?",
                        [&video_id],
                        |video_row| Ok((video_row.get(0)?, video_row.get(1)?)),
                    )
                    .ok();
                
                let title = if stored_title.trim().is_empty() || stored_title == "ニコニコ動画" {
                    from_videos.as_ref().map(|(t, _)| t.clone()).unwrap_or(stored_title)
                } else {
                    stored_title
                };
                let thumbnail_url = stored_thumbnail
                    .or_else(|| from_videos.as_ref().and_then(|(_, thumb)| thumb.clone()));
                
                Ok(crate::models::WatchLaterEntry {
                    video_id,
                    title,
                    thumbnail_url,
                    added_at,
                })
            })?
            .filter_map(|e| e.ok())
            .collect();

        Ok(entries)
    }

    pub fn get_watch_later_count(&self) -> Result<usize, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM watch_later", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

pub fn get_window_state_path(app: &tauri::AppHandle) -> PathBuf {
    get_data_dir(app).join("window_state.json")
}

pub fn save_window_state(
    app: &tauri::AppHandle,
    state: &crate::models::WindowState,
) -> Result<(), String> {
    let path = get_window_state_path(app);
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    let mut file = fs::File::create(&path)
        .map_err(|e| format!("Failed to create window state file: {}", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write window state: {}", e))?;
    Ok(())
}

pub fn load_window_state(app: &tauri::AppHandle) -> Option<crate::models::WindowState> {
    let path = get_window_state_path(app);
    if !path.exists() {
        return None;
    }
    let mut file = fs::File::open(&path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    serde_json::from_str(&contents).ok()
}

pub fn get_pip_window_state_path(app: &tauri::AppHandle) -> PathBuf {
    get_data_dir(app).join("pip_window_state.json")
}

pub fn save_pip_window_state(
    app: &tauri::AppHandle,
    state: &crate::models::PipWindowState,
) -> Result<(), String> {
    let path = get_pip_window_state_path(app);
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    let mut file = fs::File::create(&path)
        .map_err(|e| format!("Failed to create pip window state file: {}", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write pip window state: {}", e))?;
    Ok(())
}

pub fn load_pip_window_state(app: &tauri::AppHandle) -> Option<crate::models::PipWindowState> {
    let path = get_pip_window_state_path(app);
    if !path.exists() {
        return None;
    }
    let mut file = fs::File::open(&path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    serde_json::from_str(&contents).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDbPaths {
        root: PathBuf,
        videos: PathBuf,
        user_data: PathBuf,
    }

    impl TestDbPaths {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("{}-{}", prefix, unique));
            fs::create_dir_all(&root).unwrap();
            let videos = root.join("videos.db");
            let user_data = root.join("user_data.db");
            Self {
                root,
                videos,
                user_data,
            }
        }

        fn database(&self) -> Database {
            Database::new(self.videos.clone(), self.user_data.clone())
        }
    }

    impl Drop for TestDbPaths {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn history_schema_adds_immutable_first_watch_columns() {
        let paths = TestDbPaths::new("history-schema-columns");
        init_db(&paths.videos, &paths.user_data).unwrap();

        let conn = Connection::open(&paths.user_data).unwrap();
        let mut stmt = conn.prepare("PRAGMA table_info(history)").unwrap();
        let column_names: Vec<String> = stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(
            column_names.contains(&"first_watched_seq".to_string()),
            "history table should include immutable first_watched_seq column: {:?}",
            column_names
        );
        assert!(
            column_names.contains(&"first_watched_at".to_string()),
            "history table should include immutable first_watched_at column: {:?}",
            column_names
        );
    }

    #[test]
    fn init_db_backfills_missing_first_watch_sequence_for_existing_history_rows() {
        let paths = TestDbPaths::new("history-backfill");

        let videos_conn = Connection::open(&paths.videos).unwrap();
        videos_conn.execute_batch(VIDEOS_SCHEMA).unwrap();

        let user_conn = Connection::open(&paths.user_data).unwrap();
        user_conn
            .execute_batch(
                r#"
                CREATE TABLE history (
                    video_id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    thumbnail_url TEXT,
                    watched_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE watch_later (
                    video_id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    thumbnail_url TEXT,
                    added_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE config (
                    key TEXT PRIMARY KEY,
                    value TEXT
                );
                "#,
            )
            .unwrap();
        user_conn
            .execute(
                "INSERT INTO history (video_id, title, thumbnail_url, watched_at) VALUES (?, ?, ?, ?)",
                params!["sm1", "First", Option::<String>::None, "2026-03-01 00:00:00"],
            )
            .unwrap();
        user_conn
            .execute(
                "INSERT INTO history (video_id, title, thumbnail_url, watched_at) VALUES (?, ?, ?, ?)",
                params!["sm2", "Second", Option::<String>::None, "2026-03-02 00:00:00"],
            )
            .unwrap();
        drop(user_conn);

        init_db(&paths.videos, &paths.user_data).unwrap();

        let conn = Connection::open(&paths.user_data).unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT video_id, first_watched_seq, first_watched_at FROM history ORDER BY watched_at ASC, video_id ASC",
            )
            .unwrap();
        let rows: Vec<(String, i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], ("sm1".to_string(), 1, "2026-03-01 00:00:00".to_string()));
        assert_eq!(rows[1], ("sm2".to_string(), 2, "2026-03-02 00:00:00".to_string()));
    }

    #[test]
    fn rewatch_keeps_immutable_first_watch_sequence_while_refreshing_mutable_fields() {
        let paths = TestDbPaths::new("history-rewatch");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        db.mark_watched("sm9", "Original Title", Some("https://thumb/1"))
            .unwrap();

        let conn = db.connect_user_data().unwrap();
        let first_state: (i64, String) = conn
            .query_row(
                "SELECT first_watched_seq, watched_at FROM history WHERE video_id = ?",
                ["sm9"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        conn.execute(
            "UPDATE history SET watched_at = ? WHERE video_id = ?",
            params!["2000-01-01 00:00:00", "sm9"],
        )
        .unwrap();
        drop(conn);

        db.mark_watched("sm9", "Updated Title", None).unwrap();

        let conn = db.connect_user_data().unwrap();
        let second_state: (String, Option<String>, String, i64, String) = conn
            .query_row(
                "SELECT title, thumbnail_url, watched_at, first_watched_seq, first_watched_at FROM history WHERE video_id = ?",
                ["sm9"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();

        assert_eq!(second_state.0, "Updated Title");
        assert_eq!(second_state.1, Some("https://thumb/1".to_string()));
        assert_ne!(second_state.2, "2000-01-01 00:00:00");
        assert_eq!(second_state.3, first_state.0);
        assert_eq!(second_state.4, first_state.1);
    }
}
