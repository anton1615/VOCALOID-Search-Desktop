pub mod commands;
pub mod database;
pub mod models;
pub mod playback_settings_config;
pub mod scraper;
pub mod scraper_preflight;
pub mod state;

use tauri::Manager;
use models::WindowState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle();
            
            let data_dir = database::get_data_dir(app_handle);
            std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
            
            let videos_db_path = database::get_videos_db_path(app_handle);
            let user_data_db_path = database::get_user_data_db_path(app_handle);
            
            // Check for migration from single data.db to split databases
            let old_db_path = data_dir.join("data.db");
            if old_db_path.exists() && !videos_db_path.exists() {
                // Migration: rename old data.db to videos.db, create new user_data.db
                let _ = std::fs::rename(&old_db_path, &videos_db_path);
            }
            
            database::init_db(&videos_db_path, &user_data_db_path).expect("Failed to initialize database");
            
            let state = state::AppState::new(videos_db_path, user_data_db_path);
            app.manage(state);
            
            let window_state = database::load_window_state(app_handle);
            
            #[cfg(target_os = "windows")]
            {
                let data_dir = app_handle.path().app_data_dir().expect("Failed to get app data dir");
                let webview_data_dir = data_dir.join("webview_data");
                std::fs::create_dir_all(&webview_data_dir).expect("Failed to create webview data directory");
                
                let mut builder = tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into())
                )
                .title("VOCALOID Search Desktop")
                .min_inner_size(800.0, 600.0)
                .resizable(true)
                .data_directory(webview_data_dir);
                
                if let Some(ref ws) = window_state {
                    if ws.maximized {
                        builder = builder.maximized(true);
                    } else {
                        builder = builder
                            .inner_size(ws.width as f64, ws.height as f64)
                            .position(ws.x as f64, ws.y as f64);
                    }
                } else {
                    builder = builder.inner_size(1200.0, 800.0);
                }
                
                let window = builder.build().expect("Failed to create main window");
                
                let window_clone = window.clone();
                let app_handle_clone = app_handle.clone();
                tauri::WebviewWindow::on_window_event(&window, move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        // Close PiP window if open
                        if let Some(pip_window) = app_handle_clone.get_webview_window("pip") {
                            let _ = pip_window.close();
                        }
                        
                        if let Ok(pos) = window_clone.outer_position() {
                            if let Ok(size) = window_clone.inner_size() {
                                let maximized = window_clone.is_maximized().unwrap_or(false);
                                let state = WindowState {
                                    x: pos.x,
                                    y: pos.y,
                                    width: size.width,
                                    height: size.height,
                                    maximized,
                                };
                                let _ = database::save_window_state(&app_handle_clone, &state);
                            }
                        }
                    }
                });
            }
            
            #[cfg(not(target_os = "windows"))]
            {
                let mut builder = tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into())
                )
                .title("VOCALOID Search Desktop")
                .min_inner_size(800.0, 600.0)
                .resizable(true);
                
                if let Some(ref ws) = window_state {
                    if ws.maximized {
                        builder = builder.maximized(true);
                    } else {
                        builder = builder
                            .inner_size(ws.width as f64, ws.height as f64)
                            .position(ws.x as f64, ws.y as f64);
                    }
                } else {
                    builder = builder.inner_size(1200.0, 800.0);
                }
                
                let window = builder.build().expect("Failed to create main window");
                
                let window_clone = window.clone();
                let app_handle_clone = app_handle.clone();
                tauri::WebviewWindow::on_window_event(&window, move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        // Close PiP window if open
                        if let Some(pip_window) = app_handle_clone.get_webview_window("pip") {
                            let _ = pip_window.close();
                        }
                        
                        if let Ok(pos) = window_clone.outer_position() {
                            if let Ok(size) = window_clone.inner_size() {
                                let maximized = window_clone.is_maximized().unwrap_or(false);
                                let state = WindowState {
                                    x: pos.x,
                                    y: pos.y,
                                    width: size.width,
                                    height: size.height,
                                    maximized,
                                };
                                let _ = database::save_window_state(&app_handle_clone, &state);
                            }
                        }
                    }
                });
            }
            
            Ok(())
        })
.invoke_handler(tauri::generate_handler![
        commands::search,
        commands::get_video,
        commands::get_user_info,
        commands::fetch_video_metadata,
        commands::mark_watched,
        commands::get_watched,
        commands::get_history,
        commands::get_scraper_config,
        commands::save_scraper_config,
        commands::run_scraper,
        commands::get_scraper_progress,
        commands::cancel_scraper,
        commands::get_database_stats,
        commands::check_database_freshness,
        commands::open_pip_window,
        commands::close_pip_window,
        commands::notify_pip_closing,
        commands::reenter_active_playback_metadata,
        commands::select_video,
        commands::play_next,
        commands::play_previous,
        commands::get_database_path,
        commands::get_storage_info,
        commands::get_sync_preflight_estimate,
        commands::save_window_state,
        commands::load_window_state,
        commands::get_playlist_state,
        commands::set_playlist_index,
        commands::update_playlist_video,
        commands::get_playback_settings,
        commands::set_playback_settings,
        commands::get_search_state,
        commands::set_search_state,
        commands::set_search_loading,
        commands::load_more,
        commands::save_pip_window_state,
        commands::load_pip_window_state,
        // Watch Later commands
        commands::add_to_watch_later,
        commands::remove_from_watch_later,
        commands::is_in_watch_later,
        commands::get_watch_later,
        commands::get_watch_later_count,
        // History/WatchLater state commands
        commands::get_history_state,
        commands::set_history_state,
        commands::get_watch_later_state,
        commands::set_watch_later_state,
        commands::set_playlist_type,
        // Video info fetching
        commands::fetch_full_video_info,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
