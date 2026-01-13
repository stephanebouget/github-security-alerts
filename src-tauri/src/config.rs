use std::fs;
use std::path::PathBuf;
use crate::models::AppConfig;
use tauri::Manager;

// ============================================================================
// Config File Management
// ============================================================================

pub fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("github-security-alerts");

    fs::create_dir_all(&config_dir).ok();
    config_dir.join("config.json")
}

pub fn load_config() -> AppConfig {
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

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// Refresh Interval Management
// ============================================================================

#[tauri::command]
pub fn get_refresh_interval(app: tauri::AppHandle) -> Result<u32, String> {
    use crate::state::AppState;
    
    let state = app.try_state::<AppState>().ok_or("No state")?;
    let config = state.config.lock().unwrap();
    Ok(config.refresh_interval_minutes)
}

#[tauri::command]
pub fn set_refresh_interval(app: tauri::AppHandle, minutes: u32) -> Result<(), String> {
    use crate::state::AppState;
    
    let state = app.try_state::<AppState>().ok_or("No state")?;
    let mut config = state.config.lock().unwrap();
    config.refresh_interval_minutes = minutes;
    save_config(&config)?;
    Ok(())
}