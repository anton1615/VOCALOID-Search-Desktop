use crate::models::{
    WatchDataImportCompleted, WatchDataImportConfirmedSummary, WatchDataImportCounts,
    WatchDataImportPreviewResponse,
};
use chrono::NaiveDateTime;
use rusqlite::{params, Connection, TransactionBehavior};
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::Manager;

/// Schema for videos.db - contains only video metadata cache (rebuildable)
const VIDEOS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS videos (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    thumbnail_url TEXT,
    view_count INTEGER DEFAULT 0,
    comment_count INTEGER DEFAULT 0,
    mylist_count INTEGER DEFAULT 0,
    like_count INTEGER DEFAULT 0,
    start_time TEXT,
    tags TEXT,
    duration INTEGER,
    uploader_id TEXT,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportedHistoryRow {
    video_id: String,
    title: String,
    thumbnail_url: Option<String>,
    watched_at: String,
    first_watched_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportedWatchLaterRow {
    video_id: String,
    title: String,
    thumbnail_url: Option<String>,
    added_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PersistedHistoryRow {
    video_id: String,
    title: String,
    thumbnail_url: Option<String>,
    watched_at: String,
    first_watched_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PersistedWatchLaterRow {
    video_id: String,
    title: String,
    thumbnail_url: Option<String>,
    added_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedWatchDataImport {
    fingerprint: String,
    history: Vec<ImportedHistoryRow>,
    watch_later: Vec<ImportedWatchLaterRow>,
}

fn build_watch_data_import_counts(
    current_ids: &HashSet<String>,
    imported_ids: &HashSet<String>,
) -> WatchDataImportCounts {
    let overwrite = current_ids.intersection(imported_ids).count();
    WatchDataImportCounts {
        imported: imported_ids.len(),
        preserve: current_ids.len().saturating_sub(overwrite),
        overwrite,
        add: imported_ids.len().saturating_sub(overwrite),
    }
}

fn build_confirmation_token(fingerprint: &str) -> String {
    format!("watch-data-import:{}", fingerprint)
}

fn import_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("watch-data-import.db")
        .to_string()
}

fn fingerprint_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{:x}", digest)
}

fn stage_import_snapshot(bytes: &[u8], source_name: &str) -> Result<PathBuf, String> {
    for attempt in 0..8 {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Failed to stage import database {} for validation: {}", source_name, e))?
            .as_nanos();
        let staged_path = std::env::temp_dir().join(format!(
            "watch-data-import-{}-{}-{}.db",
            std::process::id(),
            unique,
            attempt
        ));

        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged_path)
        {
            Ok(mut staged_file) => {
                staged_file.write_all(bytes).map_err(|e| {
                    format!("Failed to stage import database {} for validation: {}", source_name, e)
                })?;
                staged_file.sync_all().map_err(|e| {
                    format!("Failed to stage import database {} for validation: {}", source_name, e)
                })?;
                return Ok(staged_path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(format!(
                    "Failed to stage import database {} for validation: {}",
                    source_name, e
                ))
            }
        }
    }

    Err(format!(
        "Failed to stage import database {} for validation: could not allocate unique temp path",
        source_name
    ))
}

fn validate_timestamp(value: &str, field_name: &str, table_name: &str, video_id: &str) -> Result<(), String> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").map_err(|_| {
        format!(
            "Invalid {} timestamp in {} for video_id {}: {}",
            field_name, table_name, video_id, value
        )
    })?;
    Ok(())
}

fn require_table_columns(conn: &Connection, table_name: &str, required_columns: &[&str]) -> Result<(), String> {
    let columns = load_table_columns(conn, table_name)?;

    if columns.is_empty() {
        return Err(format!("Import database is missing required table {}", table_name));
    }

    for required_column in required_columns {
        if !columns.iter().any(|column| column == required_column) {
            return Err(format!(
                "Import database table {} is missing required column {}",
                table_name, required_column
            ));
        }
    }

    Ok(())
}

fn load_table_columns(conn: &Connection, table_name: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", table_name))
        .map_err(|e| format!("Failed to inspect {} schema: {}", table_name, e))?;
    let columns = stmt
        .query_map([], |row| row.get(1))
        .map_err(|e| format!("Failed to inspect {} schema: {}", table_name, e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to inspect {} schema: {}", table_name, e))?;
    Ok(columns)
}

fn reject_duplicate_video_ids(conn: &Connection, table_name: &str) -> Result<(), String> {
    let sql = format!(
        "SELECT video_id FROM {} GROUP BY video_id HAVING COUNT(*) > 1 LIMIT 1",
        table_name
    );
    let duplicate: Result<String, _> = conn.query_row(&sql, [], |row| row.get(0));
    match duplicate {
        Ok(video_id) => Err(format!(
            "Import database contains duplicate video_id {} in {}",
            video_id, table_name
        )),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(()),
        Err(error) => Err(format!("Failed to validate {} duplicates: {}", table_name, error)),
    }
}

fn load_validated_import_from_connection(
    conn: &Connection,
    fingerprint: String,
) -> Result<ValidatedWatchDataImport, String> {
    require_table_columns(
        conn,
        "history",
        &[
            "video_id",
            "title",
            "thumbnail_url",
            "watched_at",
            "first_watched_seq",
            "first_watched_at",
        ],
    )?;
    require_table_columns(
        conn,
        "watch_later",
        &["video_id", "title", "thumbnail_url", "added_at"],
    )?;
    reject_duplicate_video_ids(conn, "history")?;
    reject_duplicate_video_ids(conn, "watch_later")?;

    let mut history_stmt = conn
        .prepare(
            "SELECT video_id, title, thumbnail_url, watched_at, first_watched_at FROM history ORDER BY rowid ASC",
        )
        .map_err(|e| format!("Failed to read imported history rows: {}", e))?;
    let history = history_stmt
        .query_map([], |row| {
            Ok(ImportedHistoryRow {
                video_id: row.get(0)?,
                title: row.get(1)?,
                thumbnail_url: row.get(2)?,
                watched_at: row.get(3)?,
                first_watched_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("Failed to read imported history rows: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read imported history rows: {}", e))?;

    for row in &history {
        validate_timestamp(&row.watched_at, "watched_at", "history", &row.video_id)?;
        validate_timestamp(
            &row.first_watched_at,
            "first_watched_at",
            "history",
            &row.video_id,
        )?;
    }

    let mut watch_later_stmt = conn
        .prepare(
            "SELECT video_id, title, thumbnail_url, added_at FROM watch_later ORDER BY rowid ASC",
        )
        .map_err(|e| format!("Failed to read imported watch_later rows: {}", e))?;
    let watch_later = watch_later_stmt
        .query_map([], |row| {
            Ok(ImportedWatchLaterRow {
                video_id: row.get(0)?,
                title: row.get(1)?,
                thumbnail_url: row.get(2)?,
                added_at: row.get(3)?,
            })
        })
        .map_err(|e| format!("Failed to read imported watch_later rows: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read imported watch_later rows: {}", e))?;

    for row in &watch_later {
        validate_timestamp(&row.added_at, "added_at", "watch_later", &row.video_id)?;
    }

    Ok(ValidatedWatchDataImport {
        fingerprint,
        history,
        watch_later,
    })
}

fn load_validated_import_from_bytes(
    bytes: &[u8],
    source_name: &str,
) -> Result<ValidatedWatchDataImport, String> {
    let fingerprint = fingerprint_bytes(bytes);
    let staged_path = stage_import_snapshot(bytes, source_name)?;
    let result = match Connection::open(&staged_path) {
        Ok(conn) => {
            let loaded = load_validated_import_from_connection(&conn, fingerprint);
            drop(conn);
            loaded
        }
        Err(e) => Err(format!(
            "Failed to open import database {}: {}",
            source_name, e
        )),
    };
    let _ = fs::remove_file(&staged_path);
    result
}

fn load_validated_import(path: &Path) -> Result<ValidatedWatchDataImport, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("Failed to read import database {}: {}", path.display(), e))?;
    let source_name = path.display().to_string();
    load_validated_import_from_bytes(&bytes, &source_name)
}

fn load_persisted_history_rows(conn: &Connection) -> Result<Vec<PersistedHistoryRow>, String> {
    let columns = load_table_columns(conn, "history")?;
    let has_first_watched_at = columns.iter().any(|column| column == "first_watched_at");

    let sql = if has_first_watched_at {
        "SELECT video_id, title, thumbnail_url, watched_at, first_watched_at FROM history"
    } else {
        "SELECT video_id, title, thumbnail_url, watched_at FROM history"
    };

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to read current history rows: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            let watched_at: String = row.get(3)?;
            let first_watched_at = if has_first_watched_at {
                row.get(4)?
            } else {
                watched_at.clone()
            };

            Ok(PersistedHistoryRow {
                video_id: row.get(0)?,
                title: row.get(1)?,
                thumbnail_url: row.get(2)?,
                watched_at,
                first_watched_at,
            })
        })
        .map_err(|e| format!("Failed to read current history rows: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read current history rows: {}", e))?;
    Ok(rows)
}

fn load_persisted_watch_later_rows(conn: &Connection) -> Result<Vec<PersistedWatchLaterRow>, String> {
    let mut stmt = conn
        .prepare("SELECT video_id, title, thumbnail_url, added_at FROM watch_later")
        .map_err(|e| format!("Failed to read current watch_later rows: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PersistedWatchLaterRow {
                video_id: row.get(0)?,
                title: row.get(1)?,
                thumbnail_url: row.get(2)?,
                added_at: row.get(3)?,
            })
        })
        .map_err(|e| format!("Failed to read current watch_later rows: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read current watch_later rows: {}", e))?;
    Ok(rows)
}

fn start_watch_data_import_transaction(
    conn: &mut Connection,
 ) -> Result<rusqlite::Transaction<'_>, String> {
    conn.transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| format!("Failed to start import transaction: {}", e))
}


fn preview_from_current_rows(
    file_name: String,
    fingerprint: String,
    current_history: &[PersistedHistoryRow],
    current_watch_later: &[PersistedWatchLaterRow],
    imported: &ValidatedWatchDataImport,
) -> WatchDataImportPreviewResponse {
    let current_history_ids: HashSet<String> = current_history
        .iter()
        .map(|row| row.video_id.clone())
        .collect();
    let current_watch_later_ids: HashSet<String> = current_watch_later
        .iter()
        .map(|row| row.video_id.clone())
        .collect();
    let imported_history_ids: HashSet<String> = imported
        .history
        .iter()
        .map(|row| row.video_id.clone())
        .collect();
    let imported_watch_later_ids: HashSet<String> = imported
        .watch_later
        .iter()
        .map(|row| row.video_id.clone())
        .collect();

    WatchDataImportPreviewResponse {
        file_name,
        confirmation_token: build_confirmation_token(&fingerprint),
        fingerprint,
        history: build_watch_data_import_counts(&current_history_ids, &imported_history_ids),
        watch_later: build_watch_data_import_counts(
            &current_watch_later_ids,
            &imported_watch_later_ids,
        ),
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
    migrate_videos_schema(&videos_conn)?;
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

fn migrate_videos_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare("PRAGMA table_info(videos)")?;
    let existing_columns: Vec<String> = stmt
        .query_map([], |row| row.get(1))?
        .collect::<Result<Vec<_>, _>>()?;

    let needs_rebuild = ["watch_url", "category", "description", "uploader_name"]
        .iter()
        .any(|column| existing_columns.iter().any(|existing| existing == column));

    if !needs_rebuild {
        return Ok(());
    }

    conn.execute_batch(
        r#"
        BEGIN IMMEDIATE;
        ALTER TABLE videos RENAME TO videos_old;
        CREATE TABLE videos (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            thumbnail_url TEXT,
            view_count INTEGER DEFAULT 0,
            comment_count INTEGER DEFAULT 0,
            mylist_count INTEGER DEFAULT 0,
            like_count INTEGER DEFAULT 0,
            start_time TEXT,
            tags TEXT,
            duration INTEGER,
            uploader_id TEXT,
            last_update_time TEXT DEFAULT CURRENT_TIMESTAMP
        );
        INSERT INTO videos (
            id,
            title,
            thumbnail_url,
            view_count,
            comment_count,
            mylist_count,
            like_count,
            start_time,
            tags,
            duration,
            uploader_id,
            last_update_time
        )
        SELECT
            id,
            title,
            thumbnail_url,
            view_count,
            comment_count,
            mylist_count,
            like_count,
            start_time,
            tags,
            duration,
            uploader_id,
            last_update_time
        FROM videos_old;
        DROP TABLE videos_old;
        CREATE INDEX IF NOT EXISTS idx_view_count ON videos(view_count);
        CREATE INDEX IF NOT EXISTS idx_mylist_count ON videos(mylist_count);
        CREATE INDEX IF NOT EXISTS idx_comment_count ON videos(comment_count);
        CREATE INDEX IF NOT EXISTS idx_like_count ON videos(like_count);
        CREATE INDEX IF NOT EXISTS idx_start_time ON videos(start_time);
        DROP TABLE IF EXISTS video_fts;
        CREATE VIRTUAL TABLE video_fts USING fts5(
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
        INSERT INTO video_fts(rowid, title, tags)
        SELECT rowid, title, COALESCE(tags, '') FROM videos;
        COMMIT;
        "#,
    )?;

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

    if existing_columns
        .iter()
        .any(|existing| existing == column_name)
    {
        return Ok(());
    }

    conn.execute(
        &format!(
            "ALTER TABLE history ADD COLUMN {} {}",
            column_name, column_definition
        ),
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

    pub fn preview_watch_data_import(
        &self,
        import_path: &PathBuf,
    ) -> Result<WatchDataImportPreviewResponse, String> {
        let imported = load_validated_import(import_path.as_path())?;
        let conn = self
            .connect_user_data()
            .map_err(|e| format!("Failed to open current user_data.db: {}", e))?;
        let current_history = load_persisted_history_rows(&conn)?;
        let current_watch_later = load_persisted_watch_later_rows(&conn)?;

        Ok(preview_from_current_rows(
            import_file_name(import_path.as_path()),
            imported.fingerprint.clone(),
            &current_history,
            &current_watch_later,
            &imported,
        ))
    }

    pub fn execute_watch_data_import(
        &self,
        import_path: &PathBuf,
        fingerprint: &str,
        confirmation_token: &str,
        confirmed_summary: &WatchDataImportConfirmedSummary,
    ) -> Result<WatchDataImportCompleted, String> {
        let imported = load_validated_import(import_path.as_path())?;
        if imported.fingerprint != fingerprint
            || build_confirmation_token(&imported.fingerprint) != confirmation_token
        {
            return Err("Import database changed since preview confirmation".to_string());
        }

        let mut conn = self
            .connect_user_data()
            .map_err(|e| format!("Failed to open current user_data.db: {}", e))?;
        migrate_user_data_schema(&conn)
            .map_err(|e| format!("Failed to prepare current user_data.db: {}", e))?;
        let tx = start_watch_data_import_transaction(&mut conn)?;
        let current_history = load_persisted_history_rows(&tx)?;
        let current_watch_later = load_persisted_watch_later_rows(&tx)?;
        let backend_preview = preview_from_current_rows(
            import_file_name(import_path.as_path()),
            imported.fingerprint.clone(),
            &current_history,
            &current_watch_later,
            &imported,
        );
        if !confirmed_summary.matches_preview(&backend_preview) {
            return Err(
                "Import preview no longer matches current data; run a fresh preview before importing"
                    .to_string(),
            );
        }
        let mut merged_history: HashMap<String, PersistedHistoryRow> = current_history
            .into_iter()
            .map(|row| (row.video_id.clone(), row))
            .collect();
        for row in &imported.history {
            merged_history.insert(
                row.video_id.clone(),
                PersistedHistoryRow {
                    video_id: row.video_id.clone(),
                    title: row.title.clone(),
                    thumbnail_url: row.thumbnail_url.clone(),
                    watched_at: row.watched_at.clone(),
                    first_watched_at: row.first_watched_at.clone(),
                },
            );
        }
        let mut merged_history_rows: Vec<PersistedHistoryRow> = merged_history.into_values().collect();
        merged_history_rows.sort_by(|left, right| {
            left.first_watched_at
                .cmp(&right.first_watched_at)
                .then_with(|| left.watched_at.cmp(&right.watched_at))
                .then_with(|| left.video_id.cmp(&right.video_id))
        });

        let mut merged_watch_later: HashMap<String, PersistedWatchLaterRow> = current_watch_later
            .into_iter()
            .map(|row| (row.video_id.clone(), row))
            .collect();
        for row in &imported.watch_later {
            merged_watch_later.insert(
                row.video_id.clone(),
                PersistedWatchLaterRow {
                    video_id: row.video_id.clone(),
                    title: row.title.clone(),
                    thumbnail_url: row.thumbnail_url.clone(),
                    added_at: row.added_at.clone(),
                },
            );
        }
        let mut merged_watch_later_rows: Vec<PersistedWatchLaterRow> =
            merged_watch_later.into_values().collect();
        merged_watch_later_rows.sort_by(|left, right| left.video_id.cmp(&right.video_id));

        tx.execute("DELETE FROM history", [])
            .map_err(|e| format!("Failed to replace history rows: {}", e))?;
        for (index, row) in merged_history_rows.iter().enumerate() {
            tx.execute(
                "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    row.video_id,
                    row.title,
                    row.thumbnail_url,
                    row.watched_at,
                    (index + 1) as i64,
                    row.first_watched_at
                ],
            )
            .map_err(|e| format!("Failed to write imported history rows: {}", e))?;
        }

        tx.execute("DELETE FROM watch_later", [])
            .map_err(|e| format!("Failed to replace watch_later rows: {}", e))?;
        for row in &merged_watch_later_rows {
            tx.execute(
                "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
                params![row.video_id, row.title, row.thumbnail_url, row.added_at],
            )
            .map_err(|e| format!("Failed to write imported watch_later rows: {}", e))?;
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit watch-data import: {}", e))?;

        Ok(WatchDataImportCompleted {
            file_name: confirmed_summary.file_name.clone(),
            history: confirmed_summary.history.clone(),
            watch_later: confirmed_summary.watch_later.clone(),
        })
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
        let ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ids)
    }

    /// Get all watched video IDs with first_watched_seq <= boundary_seq
    /// Used for frozen watched boundary in Search playback
    pub fn get_watched_ids_up_to_boundary(
        &self,
        boundary_seq: i64,
    ) -> Result<Vec<String>, rusqlite::Error> {
        let conn = self.connect_user_data()?;
        migrate_user_data_schema(&conn)?;
        let mut stmt = conn.prepare("SELECT video_id FROM history WHERE first_watched_seq <= ?")?;
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
        let max_seq: i64 = conn.query_row(
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

                let title = if stored_title.trim().is_empty() || stored_title == "ニコニコ動画"
                {
                    from_videos
                        .as_ref()
                        .map(|(t, _)| t.clone())
                        .unwrap_or(stored_title)
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
            i64,
            i64,
            i64,
            i64,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<String>,
        )],
    ) -> Result<(), rusqlite::Error> {
        let mut conn = self.connect_videos()?;
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO videos 
                (id, title, thumbnail_url, view_count, comment_count, 
                 mylist_count, like_count, start_time, tags, duration, uploader_id, last_update_time)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '+9 hours'))",
            )?;

            for video in videos {
                stmt.execute(params![
                    video.0, video.1, video.2, video.3, video.4, video.5, video.6, video.7,
                    video.8, video.9, video.10
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

                let title = if stored_title.trim().is_empty() || stored_title == "ニコニコ動画"
                {
                    from_videos
                        .as_ref()
                        .map(|(t, _)| t.clone())
                        .unwrap_or(stored_title)
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
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM watch_later", [], |row| row.get(0))?;
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
        assert_eq!(
            rows[0],
            ("sm1".to_string(), 1, "2026-03-01 00:00:00".to_string())
        );
        assert_eq!(
            rows[1],
            ("sm2".to_string(), 2, "2026-03-02 00:00:00".to_string())
        );
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

    #[test]
    fn videos_schema_omits_trimmed_playback_only_columns() {
        let paths = TestDbPaths::new("videos-schema-trimmed");
        init_db(&paths.videos, &paths.user_data).unwrap();

        let conn = Connection::open(&paths.videos).unwrap();
        let mut stmt = conn.prepare("PRAGMA table_info(videos)").unwrap();
        let column_names: Vec<String> = stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(column_names.contains(&"thumbnail_url".to_string()));
        assert!(column_names.contains(&"duration".to_string()));
        assert!(column_names.contains(&"uploader_id".to_string()));
        assert!(!column_names.contains(&"watch_url".to_string()));
        assert!(!column_names.contains(&"description".to_string()));
        assert!(!column_names.contains(&"uploader_name".to_string()));
        assert!(!column_names.contains(&"category".to_string()));
    }

    #[test]
    fn existing_videos_schema_migrates_without_removed_columns() {
        let paths = TestDbPaths::new("videos-schema-migrate");

        let videos_conn = Connection::open(&paths.videos).unwrap();
        videos_conn
            .execute_batch(
                r#"
                CREATE TABLE videos (
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
                "#,
            )
            .unwrap();
        videos_conn
            .execute(
                "INSERT INTO videos (id, title, thumbnail_url, watch_url, view_count, comment_count, mylist_count, like_count, start_time, tags, duration, category, description, uploader_id, uploader_name, last_update_time) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    "sm9",
                    "Title",
                    "https://thumb",
                    "https://www.nicovideo.jp/watch/sm9",
                    1,
                    2,
                    3,
                    4,
                    "2026-01-01T00:00:00+09:00",
                    "miku vocaloid",
                    123,
                    "music",
                    "desc",
                    "42",
                    "MikuP",
                    "2026-01-02 00:00:00"
                ],
            )
            .unwrap();
        drop(videos_conn);

        let user_conn = Connection::open(&paths.user_data).unwrap();
        user_conn.execute_batch(USER_DATA_SCHEMA).unwrap();
        drop(user_conn);

        init_db(&paths.videos, &paths.user_data).unwrap();

        let conn = Connection::open(&paths.videos).unwrap();
        let mut stmt = conn.prepare("PRAGMA table_info(videos)").unwrap();
        let column_names: Vec<String> = stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(!column_names.contains(&"watch_url".to_string()));
        assert!(!column_names.contains(&"description".to_string()));
        assert!(!column_names.contains(&"uploader_name".to_string()));
        assert!(!column_names.contains(&"category".to_string()));

        let row: (String, Option<String>, i64, Option<i64>, Option<String>) = conn
            .query_row(
                "SELECT title, thumbnail_url, like_count, duration, uploader_id FROM videos WHERE id = ?",
                ["sm9"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();

        assert_eq!(row.0, "Title");
        assert_eq!(row.1.as_deref(), Some("https://thumb"));
        assert_eq!(row.2, 4);
        assert_eq!(row.3, Some(123));
        assert_eq!(row.4.as_deref(), Some("42"));
    }

    #[test]
    fn history_entries_stay_self_contained_when_video_cache_row_is_missing() {
        let paths = TestDbPaths::new("history-self-contained");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let conn = db.connect_user_data().unwrap();
        conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                "sm9",
                "Stored History Title",
                "https://history-thumb",
                "2026-03-01 00:00:00",
                1,
                "2026-03-01 00:00:00"
            ],
        )
        .unwrap();
        drop(conn);

        let entries = db.get_history(1, 50, None).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Stored History Title");
        assert_eq!(
            entries[0].thumbnail_url.as_deref(),
            Some("https://history-thumb")
        );
    }

    #[test]
    fn watch_later_entries_stay_self_contained_when_video_cache_row_is_missing() {
        let paths = TestDbPaths::new("watch-later-self-contained");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let conn = db.connect_user_data().unwrap();
        conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params![
                "sm9",
                "Stored Watch Later Title",
                "https://watch-later-thumb",
                "2026-03-01 00:00:00"
            ],
        )
        .unwrap();
        drop(conn);

        let entries = db.get_watch_later(1, 50, None).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Stored Watch Later Title");
        assert_eq!(
            entries[0].thumbnail_url.as_deref(),
            Some("https://watch-later-thumb")
        );
    }

    #[test]
    fn watch_data_import_preview_summarizes_merge_and_ignores_extra_tables() {
        let paths = TestDbPaths::new("watch-data-import-preview");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let user_conn = db.connect_user_data().unwrap();
        user_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-current", "Current Only", Option::<String>::None, "2026-03-01 00:00:00", 9, "2026-03-01 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-overlap", "Current Overlap", Option::<String>::None, "2026-03-02 00:00:00", 10, "2026-03-02 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-current", "Current Watch Later", Option::<String>::None, "2026-03-03 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-overlap", "Current Watch Later Overlap", Option::<String>::None, "2026-03-04 00:00:00"],
        ).unwrap();
        drop(user_conn);

        let transfer_path = paths.root.join("transfer-preview.db");
        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER,
                first_watched_at TEXT NOT NULL
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            CREATE TABLE config (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            "#,
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-overlap", "Imported Overlap", Option::<String>::None, "2026-04-01 00:00:00", 1, "2026-02-01 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-new", "Imported New", Option::<String>::None, "2026-04-02 00:00:00", 2, "2026-01-15 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-overlap", "Imported Watch Later Overlap", Option::<String>::None, "2026-04-03 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-new", "Imported Watch Later New", Option::<String>::None, "2026-04-04 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO config (key, value) VALUES ('scraper', '{\"query\":\"ignored\"}')",
            [],
        ).unwrap();
        drop(transfer_conn);

        let preview = db.preview_watch_data_import(&transfer_path).unwrap();

        assert_eq!(preview.file_name, "transfer-preview.db");
        assert_eq!(preview.history.imported, 2);
        assert_eq!(preview.history.preserve, 1);
        assert_eq!(preview.history.overwrite, 1);
        assert_eq!(preview.history.add, 1);
        assert_eq!(preview.watch_later.imported, 2);
        assert_eq!(preview.watch_later.preserve, 1);
        assert_eq!(preview.watch_later.overwrite, 1);
        assert_eq!(preview.watch_later.add, 1);
        assert!(!preview.fingerprint.is_empty());
        assert!(!preview.confirmation_token.is_empty());
    }

    #[test]
    fn watch_data_import_preview_is_read_only_against_legacy_current_user_data() {
        let paths = TestDbPaths::new("watch-data-import-preview-read-only");

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
                params!["sm-existing", "Legacy Current", Option::<String>::None, "2026-03-01 00:00:00"],
            )
            .unwrap();
        drop(user_conn);

        let db = paths.database();
        let transfer_path = paths.root.join("transfer-preview-read-only.db");
        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER,
                first_watched_at TEXT NOT NULL
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            "#,
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-existing", "Imported Existing", Option::<String>::None, "2026-04-01 00:00:00", 1, "2026-03-01 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-new", "Imported New", Option::<String>::None, "2026-04-02 00:00:00", 2, "2026-04-02 00:00:00"],
        ).unwrap();
        drop(transfer_conn);

        let preview = db.preview_watch_data_import(&transfer_path).unwrap();

        assert_eq!(preview.history.imported, 2);
        assert_eq!(preview.history.preserve, 0);
        assert_eq!(preview.history.overwrite, 1);
        assert_eq!(preview.history.add, 1);

        let conn = Connection::open(&paths.user_data).unwrap();
        let mut stmt = conn.prepare("PRAGMA table_info(history)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            columns,
            vec![
                "video_id".to_string(),
                "title".to_string(),
                "thumbnail_url".to_string(),
                "watched_at".to_string(),
            ]
        );

        let row_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))
            .unwrap();
        assert_eq!(row_count, 1);
    }

    #[test]
    fn watch_data_import_preview_rejects_duplicate_history_rows() {
        let paths = TestDbPaths::new("watch-data-import-duplicate");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let transfer_path = paths.root.join("transfer-duplicate.db");
        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER,
                first_watched_at TEXT NOT NULL
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            "#,
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm9", "First", Option::<String>::None, "2026-04-01 00:00:00", 1, "2026-04-01 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm9", "Second", Option::<String>::None, "2026-04-02 00:00:00", 2, "2026-04-02 00:00:00"],
        ).unwrap();
        drop(transfer_conn);

        let error = db.preview_watch_data_import(&transfer_path).unwrap_err();

        assert!(error.contains("duplicate"));
        assert!(error.contains("history"));
    }

    #[test]
    fn watch_data_import_preview_rejects_invalid_timestamps_and_missing_columns() {
        let paths = TestDbPaths::new("watch-data-import-invalid");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let invalid_timestamp_path = paths.root.join("transfer-invalid-timestamp.db");
        let invalid_timestamp_conn = Connection::open(&invalid_timestamp_path).unwrap();
        invalid_timestamp_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER,
                first_watched_at TEXT NOT NULL
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            "#,
        ).unwrap();
        invalid_timestamp_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm9", "Bad Timestamp", Option::<String>::None, "not-a-timestamp", 1, "2026-04-01 00:00:00"],
        ).unwrap();
        drop(invalid_timestamp_conn);

        let invalid_timestamp_error = db.preview_watch_data_import(&invalid_timestamp_path).unwrap_err();
        assert!(invalid_timestamp_error.contains("watched_at"));

        let missing_column_path = paths.root.join("transfer-missing-column.db");
        let missing_column_conn = Connection::open(&missing_column_path).unwrap();
        missing_column_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            "#,
        ).unwrap();
        drop(missing_column_conn);

        let missing_column_error = db.preview_watch_data_import(&missing_column_path).unwrap_err();
        assert!(missing_column_error.contains("first_watched_at"));
    }

    #[test]
    fn fingerprint_bytes_uses_sha256_hex_and_confirmation_token_prefix() {
        let fingerprint = fingerprint_bytes(b"watch-data-import");

        assert_eq!(
            fingerprint,
            "c7b3ce186fd88f27903ba980f96980afed7d951bc9bb22b739a5dbe7ac5de238"
        );
        assert_eq!(
            build_confirmation_token(&fingerprint),
            "watch-data-import:c7b3ce186fd88f27903ba980f96980afed7d951bc9bb22b739a5dbe7ac5de238"
        );
    }


    #[test]
    fn load_validated_import_from_bytes_uses_captured_snapshot() {
        let paths = TestDbPaths::new("watch-data-import-snapshot-bytes");
        init_db(&paths.videos, &paths.user_data).unwrap();

        let transfer_path = paths.root.join("transfer-snapshot.db");
        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER,
                first_watched_at TEXT NOT NULL
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            "#,
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm9", "Before Snapshot", Option::<String>::None, "2026-04-01 00:00:00", 1, "2026-04-01 00:00:00"],
        ).unwrap();
        drop(transfer_conn);

        let snapshot_bytes = fs::read(&transfer_path).unwrap();

        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute(
            "UPDATE history SET title = ? WHERE video_id = ?",
            params!["After Snapshot", "sm9"],
        ).unwrap();
        drop(transfer_conn);

        let imported = load_validated_import_from_bytes(&snapshot_bytes, "captured snapshot").unwrap();

        assert_eq!(imported.fingerprint, fingerprint_bytes(&snapshot_bytes));
        assert_eq!(imported.history.len(), 1);
        assert_eq!(imported.history[0].title, "Before Snapshot");
    }

    #[test]
    fn start_watch_data_import_transaction_blocks_concurrent_writers() {
        let paths = TestDbPaths::new("watch-data-import-immediate-tx");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let mut first_conn = db.connect_user_data().unwrap();
        let second_conn = db.connect_user_data().unwrap();
        second_conn
            .busy_timeout(std::time::Duration::from_millis(0))
            .unwrap();

        let tx = start_watch_data_import_transaction(&mut first_conn).unwrap();
        let error = second_conn
            .execute(
                "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
                params!["sm-blocked", "Blocked Writer", Option::<String>::None, "2026-04-02 00:00:00", 1, "2026-04-02 00:00:00"],
            )
            .unwrap_err();

        assert!(matches!(error, rusqlite::Error::SqliteFailure(_, _) | rusqlite::Error::SqliteSingleThreadedMode));
        assert!(error.to_string().contains("database is locked") || error.to_string().contains("database table is locked"));
        drop(tx);
    }


    #[test]
    fn watch_data_import_execute_rejects_changed_contents_after_preview() {
        let paths = TestDbPaths::new("watch-data-import-changed");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let transfer_path = paths.root.join("transfer-changed.db");
        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER,
                first_watched_at TEXT NOT NULL
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            "#,
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm9", "Before Change", Option::<String>::None, "2026-04-01 00:00:00", 1, "2026-04-01 00:00:00"],
        ).unwrap();
        drop(transfer_conn);

        let preview = db.preview_watch_data_import(&transfer_path).unwrap();

        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute(
            "UPDATE history SET title = ? WHERE video_id = ?",
            params!["After Change", "sm9"],
        ).unwrap();
        drop(transfer_conn);

        let error = db.execute_watch_data_import(
            &transfer_path,
            &preview.fingerprint,
            &preview.confirmation_token,
            &WatchDataImportConfirmedSummary {
                file_name: preview.file_name.clone(),
                history: preview.history.clone(),
                watch_later: preview.watch_later.clone(),
            },
        ).unwrap_err();

        assert!(error.contains("changed"));
        assert_eq!(db.get_history_count().unwrap(), 0);
        assert_eq!(db.get_watch_later_count().unwrap(), 0);
    }

    #[test]
    fn watch_data_import_execute_rejects_stale_confirmed_summary_when_current_rows_change() {
        let paths = TestDbPaths::new("watch-data-import-stale-summary");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let user_conn = db.connect_user_data().unwrap();
        user_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-current", "Current Only", Option::<String>::None, "2026-03-03 00:00:00", 9, "2026-03-01 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-overlap", "Current Overlap", Option::<String>::None, "2026-03-04 00:00:00", 10, "2026-03-02 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-current", "Current Watch Later", Option::<String>::None, "2026-03-05 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-overlap", "Current Watch Later Overlap", Option::<String>::None, "2026-03-06 00:00:00"],
        ).unwrap();
        drop(user_conn);

        let transfer_path = paths.root.join("transfer-stale-summary.db");
        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER,
                first_watched_at TEXT NOT NULL
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            "#,
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-overlap", "Imported Overlap", Option::<String>::None, "2026-04-02 00:00:00", 1, "2026-02-01 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-new", "Imported New", Option::<String>::None, "2026-04-01 00:00:00", 2, "2026-01-15 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-overlap", "Imported Watch Later Overlap", Option::<String>::None, "2026-04-03 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-new", "Imported Watch Later New", Option::<String>::None, "2026-04-04 00:00:00"],
        ).unwrap();
        drop(transfer_conn);

        let preview = db.preview_watch_data_import(&transfer_path).unwrap();
        assert_eq!(preview.history.preserve, 1);
        assert_eq!(preview.watch_later.preserve, 1);

        let user_conn = db.connect_user_data().unwrap();
        user_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-late", "Added After Preview", Option::<String>::None, "2026-03-07 00:00:00", 11, "2026-03-07 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-late", "Watch Later Added After Preview", Option::<String>::None, "2026-03-08 00:00:00"],
        ).unwrap();
        drop(user_conn);

        let error = db.execute_watch_data_import(
            &transfer_path,
            &preview.fingerprint,
            &preview.confirmation_token,
            &WatchDataImportConfirmedSummary {
                file_name: preview.file_name.clone(),
                history: preview.history.clone(),
                watch_later: preview.watch_later.clone(),
            },
        ).unwrap_err();

        assert!(error.contains("preview"));
        assert!(error.contains("fresh"));
        assert_eq!(db.get_history_count().unwrap(), 3);
        assert_eq!(db.get_watch_later_count().unwrap(), 3);
    }

    #[test]
    fn watch_data_import_execute_accepts_verified_confirmed_summary_after_fresh_preview() {
        let paths = TestDbPaths::new("watch-data-import-fresh-summary");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let user_conn = db.connect_user_data().unwrap();
        user_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-current", "Current Only", Option::<String>::None, "2026-03-03 00:00:00", 9, "2026-03-01 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-current", "Current Watch Later", Option::<String>::None, "2026-03-05 00:00:00"],
        ).unwrap();
        drop(user_conn);

        let transfer_path = paths.root.join("transfer-fresh-summary.db");
        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER,
                first_watched_at TEXT NOT NULL
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            "#,
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-new", "Imported New", Option::<String>::None, "2026-04-01 00:00:00", 1, "2026-01-15 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-new", "Imported Watch Later New", Option::<String>::None, "2026-04-04 00:00:00"],
        ).unwrap();
        drop(transfer_conn);

        let stale_preview = db.preview_watch_data_import(&transfer_path).unwrap();

        let user_conn = db.connect_user_data().unwrap();
        user_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-late", "Added After Preview", Option::<String>::None, "2026-03-07 00:00:00", 10, "2026-03-07 00:00:00"],
        ).unwrap();
        drop(user_conn);

        let stale_error = db.execute_watch_data_import(
            &transfer_path,
            &stale_preview.fingerprint,
            &stale_preview.confirmation_token,
            &WatchDataImportConfirmedSummary {
                file_name: stale_preview.file_name.clone(),
                history: stale_preview.history.clone(),
                watch_later: stale_preview.watch_later.clone(),
            },
        ).unwrap_err();
        assert!(stale_error.contains("fresh"));

        let fresh_preview = db.preview_watch_data_import(&transfer_path).unwrap();
        assert_eq!(fresh_preview.history.preserve, 2);
        assert_eq!(fresh_preview.history.add, 1);

        let completed = db.execute_watch_data_import(
            &transfer_path,
            &fresh_preview.fingerprint,
            &fresh_preview.confirmation_token,
            &WatchDataImportConfirmedSummary {
                file_name: fresh_preview.file_name.clone(),
                history: fresh_preview.history.clone(),
                watch_later: fresh_preview.watch_later.clone(),
            },
        ).unwrap();

        assert_eq!(completed.file_name, fresh_preview.file_name);
        assert_eq!(completed.history, fresh_preview.history);
        assert_eq!(completed.watch_later, fresh_preview.watch_later);
        assert_eq!(db.get_history_count().unwrap(), 3);
        assert_eq!(db.get_watch_later_count().unwrap(), 2);
    }


    #[test]
    fn watch_data_import_execute_merges_rows_recomputes_history_sequence_and_preserves_config() {
        let paths = TestDbPaths::new("watch-data-import-merge");
        init_db(&paths.videos, &paths.user_data).unwrap();
        let db = paths.database();

        let user_conn = db.connect_user_data().unwrap();
        user_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-current", "Current Only", Option::<String>::None, "2026-03-03 00:00:00", 9, "2026-03-01 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-overlap", "Current Overlap", Option::<String>::None, "2026-03-04 00:00:00", 10, "2026-03-02 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-current", "Current Watch Later", Option::<String>::None, "2026-03-05 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-overlap", "Current Watch Later Overlap", Option::<String>::None, "2026-03-06 00:00:00"],
        ).unwrap();
        user_conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES ('scraper', ?)",
            ["{\"query\":\"keep-me\"}"],
        ).unwrap();
        drop(user_conn);

        let transfer_path = paths.root.join("transfer-merge.db");
        let transfer_conn = Connection::open(&transfer_path).unwrap();
        transfer_conn.execute_batch(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                watched_at TEXT NOT NULL,
                first_watched_seq INTEGER,
                first_watched_at TEXT NOT NULL
            );
            CREATE TABLE watch_later (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                thumbnail_url TEXT,
                added_at TEXT NOT NULL
            );
            "#,
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-overlap", "Imported Overlap", Option::<String>::None, "2026-04-02 00:00:00", 1, "2026-02-01 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO history (video_id, title, thumbnail_url, watched_at, first_watched_seq, first_watched_at) VALUES (?, ?, ?, ?, ?, ?)",
            params!["sm-new", "Imported New", Option::<String>::None, "2026-04-01 00:00:00", 2, "2026-01-15 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-overlap", "Imported Watch Later Overlap", Option::<String>::None, "2026-04-03 00:00:00"],
        ).unwrap();
        transfer_conn.execute(
            "INSERT INTO watch_later (video_id, title, thumbnail_url, added_at) VALUES (?, ?, ?, ?)",
            params!["sm-wl-new", "Imported Watch Later New", Option::<String>::None, "2026-04-04 00:00:00"],
        ).unwrap();
        drop(transfer_conn);

        let preview = db.preview_watch_data_import(&transfer_path).unwrap();
        let completed = db.execute_watch_data_import(
            &transfer_path,
            &preview.fingerprint,
            &preview.confirmation_token,
            &WatchDataImportConfirmedSummary {
                file_name: preview.file_name.clone(),
                history: preview.history.clone(),
                watch_later: preview.watch_later.clone(),
            },
        ).unwrap();

        assert_eq!(completed.history.imported, 2);
        assert_eq!(completed.watch_later.imported, 2);

        let user_conn = db.connect_user_data().unwrap();
        let mut stmt = user_conn.prepare(
            "SELECT video_id, title, first_watched_seq, first_watched_at, watched_at FROM history ORDER BY first_watched_seq ASC",
        ).unwrap();
        let rows: Vec<(String, String, i64, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            rows,
            vec![
                ("sm-new".to_string(), "Imported New".to_string(), 1, "2026-01-15 00:00:00".to_string(), "2026-04-01 00:00:00".to_string()),
                ("sm-overlap".to_string(), "Imported Overlap".to_string(), 2, "2026-02-01 00:00:00".to_string(), "2026-04-02 00:00:00".to_string()),
                ("sm-current".to_string(), "Current Only".to_string(), 3, "2026-03-01 00:00:00".to_string(), "2026-03-03 00:00:00".to_string()),
            ],
        );

        let mut stmt = user_conn.prepare(
            "SELECT video_id, title FROM watch_later ORDER BY video_id ASC",
        ).unwrap();
        let watch_later_rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            watch_later_rows,
            vec![
                ("sm-wl-current".to_string(), "Current Watch Later".to_string()),
                ("sm-wl-new".to_string(), "Imported Watch Later New".to_string()),
                ("sm-wl-overlap".to_string(), "Imported Watch Later Overlap".to_string()),
            ],
        );

        let config_value: String = user_conn.query_row(
            "SELECT value FROM config WHERE key = 'scraper'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(config_value, "{\"query\":\"keep-me\"}");
    }
}
