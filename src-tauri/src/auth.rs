use tauri::Manager;
use crate::models::{AuthStatus, GitHubUser};
use crate::state::AppState;
use crate::config::save_config;
use std::collections::HashMap;
use tiny_http::{Server, Response, Header};
use url::Url;
use serde::{Deserialize, Serialize};

// ============================================================================
// OAuth Configuration
// ============================================================================

const DEFAULT_CLIENT_ID: &str = "REPLACE_WITH_YOUR_CLIENT_ID";
const DEFAULT_CLIENT_SECRET: &str = "REPLACE_WITH_YOUR_CLIENT_SECRET";
const REDIRECT_URI: &str = "http://localhost:8080/callback";
const OAUTH_SCOPE: &str = "repo read:user read:org";

fn get_client_id() -> String {
    // Try environment variable first (from build time)
    if let Some(id) = option_env!("GITHUB_CLIENT_ID") {
        if id != DEFAULT_CLIENT_ID {
            return id.to_string();
        }
    }
    
    // Try runtime environment variable
    std::env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string())
}

fn get_client_secret() -> String {
    // Try environment variable first (from build time)
    if let Some(secret) = option_env!("GITHUB_CLIENT_SECRET") {
        if secret != DEFAULT_CLIENT_SECRET {
            return secret.to_string();
        }
    }
    
    // Try runtime environment variable
    std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_else(|_| DEFAULT_CLIENT_SECRET.to_string())
}

#[derive(Serialize, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
}

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

/// Start OAuth flow
#[tauri::command]
pub async fn start_oauth_flow() -> Result<String, String> {
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}",
        get_client_id(),
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(OAUTH_SCOPE)
    );
    
    Ok(auth_url)
}

/// Complete OAuth flow by listening for callback
#[tauri::command]
pub async fn complete_oauth_flow(app: tauri::AppHandle) -> Result<String, String> {
    let server = Server::http("127.0.0.1:8080")
        .map_err(|e| format!("Failed to start callback server: {}", e))?;
    
    println!("Listening on http://localhost:8080 for OAuth callback...");
    
    for request in server.incoming_requests() {
        let url_path = request.url();
        println!("Received request: {}", url_path);
        
        if url_path.starts_with("/callback") {
            // Parse the callback URL to get the authorization code
            let full_url = format!("http://localhost:8080{}", url_path);
            let parsed_url = Url::parse(&full_url)
                .map_err(|e| format!("Failed to parse callback URL: {}", e))?;
            
            let params: HashMap<_, _> = parsed_url.query_pairs().collect();
            
            if let Some(code) = params.get("code") {
                // Send success response to browser
                let success_html = r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>GitHub Authorization</title></head>
                    <body style="font-family: Arial, sans-serif; text-align: center; padding: 50px;">
                        <h1 style="color: #28a745;">Authorization Successful!</h1>
                        <p>You can now close this window.</p>
                        <script>setTimeout(() => window.close(), 2000);</script>
                    </body>
                    </html>
                "#;
                
                let response = Response::from_string(success_html)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                let _ = request.respond(response);
                
                // Exchange code for token
                let access_token = exchange_code_for_token(code).await?;
                
                // Save token
                if let Some(state) = app.try_state::<AppState>() {
                    let mut config = state.config.lock().unwrap();
                    config.access_token = Some(access_token.clone());
                    save_config(&config)?;
                }
                
                return Ok(access_token);
                
            } else if let Some(error) = params.get("error") {
                // Send error response to browser
                let error_html = format!(r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>GitHub Authorization</title></head>
                    <body style="font-family: Arial, sans-serif; text-align: center; padding: 50px;">
                        <h1 style="color: #dc3545;">✗ Authorization Failed</h1>
                        <p>Error: {}</p>
                        <p>You can close this window and try again.</p>
                        <script>setTimeout(() => window.close(), 3000);</script>
                    </body>
                    </html>
                "#, error);
                
                let response = Response::from_string(error_html)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                let _ = request.respond(response);
                
                return Err(format!("OAuth error: {}", error));
            }
        }
        
        // Send default response for other requests
        let response = Response::from_string("Waiting for GitHub callback...");
        let _ = request.respond(response);
    }
    
    Err("OAuth flow was interrupted".to_string())
}

/// Exchange authorization code for access token
async fn exchange_code_for_token(code: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let client_id = get_client_id();
    let client_secret = get_client_secret();
    
    let mut params = HashMap::new();
    params.insert("client_id", client_id.as_str());
    params.insert("client_secret", client_secret.as_str());
    params.insert("code", code);
    params.insert("redirect_uri", REDIRECT_URI);
    
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .header("User-Agent", "github-security-alerts")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to exchange code: {}", e))?;
    
    let status = response.status();
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed ({}): {}", status, error_text));
    }
    
    // Get the raw text to debug
    let response_text = response.text().await
        .map_err(|e| format!("Failed to get response text: {}", e))?;
    
    println!("GitHub OAuth response: {}", response_text);
    
    // Try to parse as JSON
    let token_response: OAuthTokenResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse token response: {} | Raw response: {}", e, response_text))?;
    
    Ok(token_response.access_token)
}