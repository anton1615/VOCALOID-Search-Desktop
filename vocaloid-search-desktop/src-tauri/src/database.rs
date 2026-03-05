use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

const SCHEMA: &str = r#"
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

CREATE TABLE IF NOT EXISTS history (
    video_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    thumbnail_url TEXT,
    watched_at TEXT DEFAULT CURRENT_TIMESTAMP
);

DROP TABLE IF EXISTS watched;

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
}

impl Default for StoredConfig {
    fn default() -> Self {
        Self {
            query: "VOCALOID".to_string(),
            max_age_days: Some(365),
            targets: "tags".to_string(),
            category_filter: Some("MUSIC".to_string()),
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

pub fn get_db_path(app: &tauri::AppHandle) -> PathBuf {
    get_data_dir(app).join("data.db")
}

pub fn get_config_path(app: &tauri::AppHandle) -> PathBuf {
    get_data_dir(app).join("config.json")
}

pub fn init_db(path: &PathBuf) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "synchronous", &"NORMAL")?;
    conn.pragma_update(None, "cache_size", &-64000)?;
    Ok(())
}

pub struct Database {
    path: Arc<PathBuf>,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            path: Arc::clone(&self.path),
        }
    }
}

impl Database {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: Arc::new(path),
        }
    }

    pub fn connect(&self) -> Result<Connection, rusqlite::Error> {
        let conn = Connection::open(&*self.path)?;
        conn.pragma_update(None, "journal_mode", &"WAL")?;
        conn.pragma_update(None, "synchronous", &"NORMAL")?;
        conn.pragma_update(None, "cache_size", &-64000)?;
        conn.pragma_update(None, "mmap_size", &268435456)?;
        Ok(conn)
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
        let conn = self.connect()?;
        let exists: i64 = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM history WHERE video_id = ?)",
            [video_id],
            |row| row.get(0),
        )?;
        Ok(exists == 1)
    }

    pub fn mark_watched(
        &self,
        video_id: &str,
        title: &str,
        thumbnail_url: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT OR REPLACE INTO history (video_id, title, thumbnail_url, watched_at) VALUES (?, ?, ?, datetime('now', '+9 hours'))",
            params![video_id, title, thumbnail_url],
        )?;
        Ok(())
    }

    pub fn get_history(
        &self,
        page: usize,
        page_size: usize,
    ) -> Result<Vec<crate::models::HistoryEntry>, rusqlite::Error> {
        let conn = self.connect()?;
        let offset = (page - 1) * page_size;

        let mut stmt = conn.prepare(
            "SELECT video_id, title, thumbnail_url, watched_at FROM history ORDER BY watched_at DESC LIMIT ? OFFSET ?"
        )?;

        let entries: Vec<crate::models::HistoryEntry> = stmt
            .query_map([page_size as i64, offset as i64], |row| {
                Ok(crate::models::HistoryEntry {
                    video_id: row.get(0)?,
                    title: row.get(1)?,
                    thumbnail_url: row.get(2)?,
                    watched_at: row.get(3)?,
                })
            })?
            .filter_map(|e| e.ok())
            .collect();

        Ok(entries)
    }

    pub fn get_history_count(&self) -> Result<usize, rusqlite::Error> {
        let conn = self.connect()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn clear_videos(&self) -> Result<(), rusqlite::Error> {
        let conn = self.connect()?;
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
        let mut conn = self.connect()?;
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
        let conn = self.connect()?;
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
        let conn = self.connect()?;
        let json = serde_json::to_string(config).unwrap_or_else(|_| "{}".to_string());
        conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES ('scraper', ?)",
            [&json],
        )?;
        Ok(())
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
