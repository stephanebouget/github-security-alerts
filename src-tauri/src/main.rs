#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use tauri::{
  Manager,
  tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
  menu::{Menu, MenuItem},
  image::Image,
  PhysicalPosition,
};
use std::sync::Mutex;
use std::time::Instant;

// Global state to store the number of alerts and last window show time
struct AppState {
  alert_count: Mutex<usize>,
  last_shown: Mutex<Option<Instant>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AlertsResponse {
  total_alerts: usize,
  repos: Vec<RepoAlerts>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RepoAlerts {
  name: String,
  alerts: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubAlert {
  state: String,
}

#[tauri::command]
async fn hello_world_command(_app: tauri::AppHandle) -> Result<String, String> {
  println!("I was invoked from JS!");
  Ok("Hello world from Tauri!".into())
}

#[tauri::command]
async fn get_github_security_alerts(
  github_token: String,
) -> Result<AlertsResponse, String> {
  let token = github_token;
  let repos = vec![
    "KhiopsML/khiops-visualization",
    "KhiopsML/khiops-visualization-desktop",
    "stephanebouget/github-security-alerts",
    "KhiopsML/kv-electron",
    "stephanebouget/powo-cli",
    "KhiopsML/ngx-khiops-histogram"
  ];

  let mut total_alerts = 0;
  let mut repo_alerts = Vec::new();

  let client = reqwest::Client::new();

  for repo in repos {
    let url = format!(
      "https://api.github.com/repos/{}/dependabot/alerts",
      repo
    );

    match client
      .get(&url)
      .header("Accept", "application/vnd.github+json")
      .header("Authorization", format!("Bearer {}", token))
      .header("User-Agent", "github-security-alerts")
      .send()
      .await
    {
      Ok(response) => {
        match response.json::<Vec<GitHubAlert>>().await {
          Ok(alerts) => {
            let open_alerts = alerts.iter()
              .filter(|a| a.state == "open")
              .count();
            total_alerts += open_alerts;
            repo_alerts.push(RepoAlerts {
              name: repo.to_string(),
              alerts: open_alerts,
            });
          }
          Err(e) => {
            eprintln!("Failed to parse alerts for {}: {}", repo, e);
            repo_alerts.push(RepoAlerts {
              name: repo.to_string(),
              alerts: 0,
            });
          }
        }
      }
      Err(e) => {
        eprintln!("Failed to fetch alerts for {}: {}", repo, e);
        repo_alerts.push(RepoAlerts {
          name: repo.to_string(),
          alerts: 0,
        });
      }
    }
  }

  Ok(AlertsResponse {
    total_alerts,
    repos: repo_alerts,
  })
}

// Embedded tray icons - replace these files with your own designs
// Icons should be 32x32 or 64x64 PNG with transparency
const ICON_GRAY: &[u8] = include_bytes!("../icons/tray/icon-gray.png");
const ICON_GREEN: &[u8] = include_bytes!("../icons/tray/icon-green.png");
const ICON_RED: &[u8] = include_bytes!("../icons/tray/icon-red.png");

// Returns the appropriate tray icon based on state
// None = loading (gray), Some(0) = green/ok, Some(n) = red/alert
fn generate_tray_icon(count: Option<usize>) -> Vec<u8> {
  match count {
    None => ICON_GRAY.to_vec(),
    Some(0) => ICON_GREEN.to_vec(),
    Some(_) => ICON_RED.to_vec(),
  }
}

#[tauri::command]
async fn update_tray_icon(
  app: tauri::AppHandle,
  alert_count: usize,
) -> Result<(), String> {
  // Update the state
  if let Some(state) = app.try_state::<AppState>() {
    let mut count = state.alert_count.lock().unwrap();
    *count = alert_count;
  }
  
  // Generate the new icon with Some(count)
  let icon_data = generate_tray_icon(Some(alert_count));
  
  // Update the tray icon
  if let Some(tray) = app.tray_by_id("main-tray") {
    let icon = Image::from_bytes(&icon_data).map_err(|e| e.to_string())?;
    tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;
    
    // Update the tooltip
    let tooltip = if alert_count == 0 {
      "GitHub Security Alerts - No alerts".to_string()
    } else {
      format!("GitHub Security Alerts - {} alert(s)!", alert_count)
    };
    tray.set_tooltip(Some(&tooltip)).map_err(|e| e.to_string())?;
  }
  
  // Update the window title as well
  if let Some(window) = app.get_webview_window("main") {
    let title = if alert_count > 0 {
      format!("GitHub Alerts - {} alert(s)", alert_count)
    } else {
      "GitHub Alerts".to_string()
    };
    let _ = window.set_title(&title);
  }

  Ok(())
}

fn main() {
  tauri::Builder::default()
    .manage(AppState {
      alert_count: Mutex::new(0),
      last_shown: Mutex::new(None),
    })
    .setup(|app| {
      // Create the tray menu
      let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
      let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&show, &hide, &quit])?;
      
      // Generate the initial icon (gray, loading state)
      let icon_data = generate_tray_icon(None);
      let icon = Image::from_bytes(&icon_data)?;
      
      // Hide the main window at startup
      if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
      }
      
      // Create the tray icon
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
          if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
              // Toggle visibility
              if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
              } else {
                // Update last_shown timestamp
                if let Some(state) = app.try_state::<AppState>() {
                  let mut last_shown = state.last_shown.lock().unwrap();
                  *last_shown = Some(Instant::now());
                }
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
      // Intercept window close to hide it instead of quitting
      if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        let _ = window.hide();
        api.prevent_close();
      }
      // Hide the window if it loses focus (but ignore if just opened)
      if let tauri::WindowEvent::Focused(focused) = event {
        if !focused {
          let app = window.app_handle();
          let should_hide = if let Some(state) = app.try_state::<AppState>() {
            let last_shown = state.last_shown.lock().unwrap();
            if let Some(instant) = *last_shown {
              // Ignore focus loss within 500ms of opening
              instant.elapsed().as_millis() > 500
            } else {
              true
            }
          } else {
            true
          };
          
          if should_hide {
            let _ = window.hide();
          }
        }
      }
    })
    .invoke_handler(tauri::generate_handler![
      hello_world_command,
      get_github_security_alerts,
      update_tray_icon
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

// Position the window at the bottom right of the screen, near the system tray
fn position_window_near_tray(window: &tauri::WebviewWindow) {
  if let Ok(Some(monitor)) = window.primary_monitor() {
    let monitor_size = monitor.size();
    let monitor_position = monitor.position();
    
    // Window size
    let window_width = 420i32;
    let window_height = 500i32;
    
    // Margin from the edge (to avoid sticking to the screen edge)
    let margin = 10i32;
    
    // Position at the bottom right
    // On Windows, the taskbar is usually at the bottom with a height of about 48px
    let taskbar_height = 48i32;
    
    let x = monitor_position.x + monitor_size.width as i32 - window_width - margin;
    let y = monitor_position.y + monitor_size.height as i32 - window_height - taskbar_height - margin;
    
    let _ = window.set_position(PhysicalPosition::new(x, y));
  }
}
