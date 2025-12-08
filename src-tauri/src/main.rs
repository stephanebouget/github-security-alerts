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
use std::fs;
use std::path::PathBuf;


// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
  access_token: Option<String>,
  selected_repos: Vec<String>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      access_token: None,
      selected_repos: vec![],
    }
  }
}

struct AppState {
  alert_count: Mutex<usize>,
  last_shown: Mutex<Option<Instant>>,
  config: Mutex<AppConfig>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepo {
  full_name: String,
  name: String,
  owner: GitHubOwner,
  private: bool,
  permissions: Option<GitHubPermissions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubOwner {
  login: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubPermissions {
  admin: Option<bool>,
  push: Option<bool>,
  pull: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoInfo {
  full_name: String,
  name: String,
  owner: String,
  selected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubUser {
  login: String,
  name: Option<String>,
  avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthStatus {
  authenticated: bool,
  username: Option<String>,
}

// ============================================================================
// Config File Management
// ============================================================================

fn get_config_path() -> PathBuf {
  let config_dir = dirs::config_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("github-security-alerts");
  
  fs::create_dir_all(&config_dir).ok();
  config_dir.join("config.json")
}

fn load_config() -> AppConfig {
  let path = get_config_path();
  if path.exists() {
    if let Ok(content) = fs::read_to_string(&path) {
      if let Ok(config) = serde_json::from_str(&content) {
        return config;
      }
    }
  }
  AppConfig::default()
}

fn save_config(config: &AppConfig) -> Result<(), String> {
  let path = get_config_path();
  let content = serde_json::to_string_pretty(config)
    .map_err(|e| e.to_string())?;
  fs::write(path, content).map_err(|e| e.to_string())?;
  Ok(())
}

// ============================================================================
// Tray Icons
// ============================================================================

const ICON_GRAY: &[u8] = include_bytes!("../icons/tray/icon-gray.png");
const ICON_GREEN: &[u8] = include_bytes!("../icons/tray/icon-green.png");
const ICON_RED: &[u8] = include_bytes!("../icons/tray/icon-red.png");

fn generate_tray_icon(count: Option<usize>) -> Vec<u8> {
  match count {
    None => ICON_GRAY.to_vec(),
    Some(0) => ICON_GREEN.to_vec(),
    Some(_) => ICON_RED.to_vec(),
  }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Set a Personal Access Token directly
#[tauri::command]
async fn set_token(app: tauri::AppHandle, token: String) -> Result<(), String> {
  // Verify token by fetching user info
  let client = reqwest::Client::new();
  let response = client
    .get("https://api.github.com/user")
    .header("Authorization", format!("Bearer {}", token))
    .header("User-Agent", "github-security-alerts")
    .send()
    .await
    .map_err(|e| format!("Failed to verify token: {}", e))?;
  
  if !response.status().is_success() {
    return Err("Invalid token".to_string());
  }
  
  // Token is valid, save it
  if let Some(state) = app.try_state::<AppState>() {
    let mut config = state.config.lock().unwrap();
    config.access_token = Some(token);
    save_config(&config)?;
  }
  
  Ok(())
}

#[tauri::command]
async fn get_auth_status(app: tauri::AppHandle) -> Result<AuthStatus, String> {
  let token = {
    let state = app.try_state::<AppState>().ok_or("No state")?;
    let config = state.config.lock().unwrap();
    config.access_token.clone()
  };
  
  match token {
    Some(token) => {
      // Verify token by fetching user info
      let client = reqwest::Client::new();
      let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "github-security-alerts")
        .send()
        .await;
      
      match response {
        Ok(resp) if resp.status().is_success() => {
          let user: GitHubUser = resp.json().await.map_err(|e| e.to_string())?;
          Ok(AuthStatus {
            authenticated: true,
            username: Some(user.login),
          })
        }
        _ => {
          // Token is invalid, clear it
          if let Some(state) = app.try_state::<AppState>() {
            let mut config = state.config.lock().unwrap();
            config.access_token = None;
            let _ = save_config(&config);
          }
          Ok(AuthStatus {
            authenticated: false,
            username: None,
          })
        }
      }
    }
    None => Ok(AuthStatus {
      authenticated: false,
      username: None,
    }),
  }
}

#[tauri::command]
async fn logout(app: tauri::AppHandle) -> Result<(), String> {
  if let Some(state) = app.try_state::<AppState>() {
    let mut config = state.config.lock().unwrap();
    config.access_token = None;
    config.selected_repos = vec![];
    save_config(&config)?;
  }
  Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GitHubOrg {
  login: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OwnerInfo {
  name: String,
  is_user: bool,
}

/// Get list of owners (user + organizations)
#[tauri::command]
async fn get_owners(app: tauri::AppHandle) -> Result<Vec<OwnerInfo>, String> {
  let token = {
    let state = app.try_state::<AppState>().ok_or("No state")?;
    let config = state.config.lock().unwrap();
    config.access_token.clone().ok_or("Not authenticated")?
  };
  
  let client = reqwest::Client::new();
  let mut owners = Vec::new();
  
  // Get current user
  let user_response = client
    .get("https://api.github.com/user")
    .header("Authorization", format!("Bearer {}", token))
    .header("User-Agent", "github-security-alerts")
    .send()
    .await
    .map_err(|e| format!("Failed to fetch user: {}", e))?;
  
  if user_response.status().is_success() {
    let user: GitHubUser = user_response.json().await.map_err(|e| e.to_string())?;
    owners.push(OwnerInfo {
      name: user.login,
      is_user: true,
    });
  }
  
  // Get organizations
  let orgs_response = client
    .get("https://api.github.com/user/orgs")
    .header("Authorization", format!("Bearer {}", token))
    .header("User-Agent", "github-security-alerts")
    .send()
    .await
    .map_err(|e| format!("Failed to fetch orgs: {}", e))?;
  
  if orgs_response.status().is_success() {
    let orgs: Vec<GitHubOrg> = orgs_response.json().await.unwrap_or_default();
    for org in orgs {
      owners.push(OwnerInfo {
        name: org.login,
        is_user: false,
      });
    }
  }
  
  Ok(owners)
}

/// Get repos for a specific owner (user or organization)
#[tauri::command]
async fn get_repos_for_owner(app: tauri::AppHandle, owner: String, is_user: bool) -> Result<Vec<RepoInfo>, String> {
  let (token, selected_repos) = {
    let state = app.try_state::<AppState>().ok_or("No state")?;
    let config = state.config.lock().unwrap();
    (
      config.access_token.clone().ok_or("Not authenticated")?,
      config.selected_repos.clone(),
    )
  };
  
  let client = reqwest::Client::new();
  let mut all_repos = Vec::new();
  let mut page = 1;
  
  println!("Fetching repos for owner: {} (is_user: {})", owner, is_user);
  
  loop {
    let url = if is_user {
      "https://api.github.com/user/repos".to_string()
    } else {
      format!("https://api.github.com/orgs/{}/repos", owner)
    };
    
    let mut request = client.get(&url)
      .query(&[
        ("per_page", "100"),
        ("page", &page.to_string()),
        ("sort", "full_name"),
        ("direction", "asc"),
      ])
      .header("Authorization", format!("Bearer {}", token))
      .header("User-Agent", "github-security-alerts");
    
    // For user repos, filter by affiliation to only get owned repos
    if is_user {
      request = request.query(&[("affiliation", "owner")]);
    } else {
      request = request.query(&[("type", "all")]);
    }
    
    let response = request
      .send()
      .await
      .map_err(|e| format!("Failed to fetch repos: {}", e))?;
    
    if !response.status().is_success() {
      let text = response.text().await.unwrap_or_default();
      println!("Error response: {}", text);
      break;
    }
    
    let repos: Vec<GitHubRepo> = response.json().await.unwrap_or_default();
    
    println!("Page {} returned {} repos", page, repos.len());
    
    if repos.is_empty() {
      break;
    }
    
    // Filter repos that belong to this owner (for user repos)
    let filtered_repos: Vec<GitHubRepo> = if is_user {
      repos.into_iter().filter(|r| r.owner.login.to_lowercase() == owner.to_lowercase()).collect()
    } else {
      repos
    };
    
    all_repos.extend(filtered_repos);
    page += 1;
    
    if page > 20 {
      break;
    }
  }
  
  // Sort by name
  all_repos.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
  
  let repo_infos: Vec<RepoInfo> = all_repos
    .into_iter()
    .map(|r| RepoInfo {
      full_name: r.full_name.clone(),
      name: r.name,
      owner: r.owner.login,
      selected: selected_repos.contains(&r.full_name),
    })
    .collect();
  
  println!("Returning {} repos for {}", repo_infos.len(), owner);
  
  Ok(repo_infos)
}

#[tauri::command]
async fn set_selected_repos(app: tauri::AppHandle, repos: Vec<String>) -> Result<(), String> {
  if let Some(state) = app.try_state::<AppState>() {
    let mut config = state.config.lock().unwrap();
    config.selected_repos = repos;
    save_config(&config)?;
  }
  Ok(())
}

#[tauri::command]
async fn get_selected_repos(app: tauri::AppHandle) -> Result<Vec<String>, String> {
  let state = app.try_state::<AppState>().ok_or("No state")?;
  let config = state.config.lock().unwrap();
  Ok(config.selected_repos.clone())
}

#[tauri::command]
async fn get_github_security_alerts(app: tauri::AppHandle) -> Result<AlertsResponse, String> {
  let (token, repos) = {
    let state = app.try_state::<AppState>().ok_or("No state")?;
    let config = state.config.lock().unwrap();
    (
      config.access_token.clone().ok_or("Not authenticated")?,
      config.selected_repos.clone(),
    )
  };
  
  if repos.is_empty() {
    return Ok(AlertsResponse {
      total_alerts: 0,
      repos: vec![],
    });
  }
  
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
              name: repo,
              alerts: open_alerts,
            });
          }
          Err(e) => {
            eprintln!("Failed to parse alerts for {}: {}", repo, e);
            repo_alerts.push(RepoAlerts {
              name: repo,
              alerts: 0,
            });
          }
        }
      }
      Err(e) => {
        eprintln!("Failed to fetch alerts for {}: {}", repo, e);
        repo_alerts.push(RepoAlerts {
          name: repo,
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

#[tauri::command]
async fn update_tray_icon(
  app: tauri::AppHandle,
  alert_count: usize,
) -> Result<(), String> {
  if let Some(state) = app.try_state::<AppState>() {
    let mut count = state.alert_count.lock().unwrap();
    *count = alert_count;
  }
  
  let icon_data = generate_tray_icon(Some(alert_count));
  
  if let Some(tray) = app.tray_by_id("main-tray") {
    let icon = Image::from_bytes(&icon_data).map_err(|e| e.to_string())?;
    tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;
    
    let tooltip = if alert_count == 0 {
      "GitHub Security Alerts - No alerts".to_string()
    } else {
      format!("GitHub Security Alerts - {} alert(s)!", alert_count)
    };
    tray.set_tooltip(Some(&tooltip)).map_err(|e| e.to_string())?;
  }
  
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

// ============================================================================
// Main Application
// ============================================================================

fn main() {
  let config = load_config();
  
  tauri::Builder::default()
    .manage(AppState {
      alert_count: Mutex::new(0),
      last_shown: Mutex::new(None),
      config: Mutex::new(config),
    })
    .setup(|app| {
      let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
      let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&show, &hide, &quit])?;
      
      let icon_data = generate_tray_icon(None);
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
          if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
              if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
              } else {
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
      if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        let _ = window.hide();
        api.prevent_close();
      }
      if let tauri::WindowEvent::Focused(focused) = event {
        if !focused {
          let app = window.app_handle();
          let should_hide = if let Some(state) = app.try_state::<AppState>() {
            let last_shown = state.last_shown.lock().unwrap();
            if let Some(instant) = *last_shown {
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
      set_token,
      get_auth_status,
      logout,
      get_owners,
      get_repos_for_owner,
      set_selected_repos,
      get_selected_repos,
      get_github_security_alerts,
      update_tray_icon
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn position_window_near_tray(window: &tauri::WebviewWindow) {
  if let Ok(Some(monitor)) = window.primary_monitor() {
    let monitor_size = monitor.size();
    let monitor_position = monitor.position();
    
    let window_width = 420i32;
    let window_height = 500i32;
    let margin = 10i32;
    let taskbar_height = 48i32;
    
    let x = monitor_position.x + monitor_size.width as i32 - window_width - margin;
    let y = monitor_position.y + monitor_size.height as i32 - window_height - taskbar_height - margin;
    
    let _ = window.set_position(PhysicalPosition::new(x, y));
  }
}
