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
mod system;
mod updater;

use config::load_config;
use state::AppState;
use tray::generate_tray_icon;
use window::{position_window_near_tray, handle_window_focus_lost, handle_window_show};

fn main() {
    // Load environment variables from .env file
    if dotenv::dotenv().is_err() {
        let _ = dotenv::from_filename("../.env");
    }
    
    let config = load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                position_window_near_tray(&window);
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .manage(AppState {
            alert_count: Mutex::new(0),
            last_shown: Mutex::new(None),
            last_focus_lost: Mutex::new(None),
            auto_hide_paused: Mutex::new(false),
            config: Mutex::new(config),
        })
        .setup(|app| {
            // Enable autostart on first run
            use tauri_plugin_autostart::ManagerExt;
            let autostart_manager = app.autolaunch();
            if !autostart_manager.is_enabled().unwrap_or(false) {
                let _ = autostart_manager.enable();
                println!("Autostart enabled");
            }

            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
            let restart = MenuItem::with_id(app, "restart", "Restart", true, None::<&str>)?;
            let show_dev_tools = MenuItem::with_id(app, "show_dev_tools", "Show Dev Tools", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &restart, &show_dev_tools, &quit])?;

            // Check if repos are configured
            let has_repos = {
                let state = app.state::<AppState>();
                let config = state.config.lock().unwrap();
                !config.selected_repos.is_empty()
            };

            let icon_data = generate_tray_icon(None, has_repos);
            let icon = Image::from_bytes(&icon_data)?;

            // Check if user is authenticated
            let is_authenticated = {
                let state = app.state::<AppState>();
                let config = state.config.lock().unwrap();
                config.access_token.as_ref()
                    .map(|t| !t.trim().is_empty())
                    .unwrap_or(false)
            };

            // Show window if not authenticated, hide if already logged in
            if let Some(window) = app.get_webview_window("main") {
                if is_authenticated {
                    let _ = window.hide();
                } else {
                    // First time - show window for login
                    handle_window_show(app.handle());
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
                                let _ = window.unminimize();
                            }
                        }
                        "hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        "restart" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.eval("window.location.reload()");
                            }
                        }
                        "show_dev_tools" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.open_devtools();
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
                                let _ = window.unminimize();
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
                } else {
                    // Window regained focus - clear the focus lost timestamp
                    if let Some(state) = window.app_handle().try_state::<AppState>() {
                        let mut last_focus_lost = state.last_focus_lost.lock().unwrap();
                        *last_focus_lost = None;
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            auth::set_token,
            auth::get_auth_status,
            auth::get_token,
            auth::logout,
            auth::start_oauth_flow,
            auth::complete_oauth_flow,
            repos::get_owners,
            repos::get_repos_for_owner,
            repos::set_selected_repos,
            repos::get_selected_repos,
            alerts::get_github_security_alerts,
            tray::update_tray_icon,
            system::open_taskbar_settings,
            window::pause_auto_hide,
            window::resume_auto_hide,
            updater::check_for_updates,
            updater::install_update,
            updater::get_current_version,
            config::get_refresh_interval,
            config::set_refresh_interval
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
