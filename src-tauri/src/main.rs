#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{
  Manager,
  tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
  menu::{Menu, MenuItem},
  image::Image,
};
use std::sync::Mutex;

// Module declarations
mod models;
mod config;
mod state;
mod auth;
mod repos;
mod alerts;
mod tray;
mod window;

use config::load_config;
use state::AppState;
use tray::generate_tray_icon;
use window::{position_window_near_tray, handle_window_focus_lost, handle_window_show};



// ============================================================================
// Main Application
// ============================================================================

fn main() {
    let config = load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            alert_count: Mutex::new(0),
            last_shown: Mutex::new(None),
            config: Mutex::new(config),
            dev_tools_open: Mutex::new(false),
        })
        .setup(|app| {
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

            // Check if repos are configured
            let has_repos = {
                if let Some(state) = app.try_state::<AppState>() {
                    let config = state.config.lock().unwrap();
                    !config.selected_repos.is_empty()
                } else {
                    false
                }
            };

            let icon_data = generate_tray_icon(None, has_repos);
            let icon = Image::from_bytes(&icon_data)?;

            // Check if user is authenticated
            let is_authenticated = {
                if let Some(state) = app.try_state::<AppState>() {
                    let config = state.config.lock().unwrap();
                    config.access_token.is_some()
                } else {
                    false
                }
            };

            // Show window if not authenticated, hide if already logged in
            if let Some(window) = app.get_webview_window("main") {
                if is_authenticated {
                    let _ = window.hide();
                } else {
                    // First time - show window for login
                    position_window_near_tray(&window);
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .tooltip("GitHub Security Alerts")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                position_window_near_tray(&window);
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { 
                        button: MouseButton::Left, 
                        button_state: MouseButtonState::Up, 
                        .. 
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                handle_window_show(&app);
                                position_window_near_tray(&window);
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
            if let tauri::WindowEvent::Focused(focused) = event {
                if !*focused {
                    handle_window_focus_lost(window);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            auth::set_token,
            auth::get_auth_status,
            auth::logout,
            repos::get_owners,
            repos::get_repos_for_owner,
            repos::set_selected_repos,
            repos::get_selected_repos,
            alerts::get_github_security_alerts,
            tray::update_tray_icon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
