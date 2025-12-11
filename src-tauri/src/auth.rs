use tauri::Manager;
use crate::models::{AuthStatus, GitHubUser};
use crate::state::AppState;
use crate::config::save_config;

// ============================================================================
// Authentication Commands
// ============================================================================

/// Set a Personal Access Token directly
#[tauri::command]
pub async fn set_token(app: tauri::AppHandle, token: String) -> Result<(), String> {
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
pub async fn get_auth_status(app: tauri::AppHandle) -> Result<AuthStatus, String> {
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
pub async fn get_token(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let state = app.try_state::<AppState>().ok_or("No state")?;
    let config = state.config.lock().unwrap();
    Ok(config.access_token.clone())
}

#[tauri::command]
pub async fn logout(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(state) = app.try_state::<AppState>() {
        let mut config = state.config.lock().unwrap();
        config.access_token = None;
        config.selected_repos = vec![];
        save_config(&config)?;
    }
    Ok(())
}