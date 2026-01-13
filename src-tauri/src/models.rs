use serde::{Deserialize, Serialize};

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub access_token: Option<String>,
    pub selected_repos: Vec<String>,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_minutes: u32,
}

fn default_refresh_interval() -> u32 {
    60 // Default to 1 hour
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            access_token: None,
            selected_repos: vec![],
            refresh_interval_minutes: 60, // Default to 1 hour
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertsResponse {
    pub total_alerts: usize,
    pub repos: Vec<RepoAlerts>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoAlerts {
    pub name: String,
    pub alerts: usize,
    pub dependabot_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAlert {
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub full_name: String,
    pub name: String,
    pub owner: GitHubOwner,
    pub private: bool,
    pub permissions: Option<GitHubPermissions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubPermissions {
    pub admin: Option<bool>,
    pub push: Option<bool>,
    pub pull: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoInfo {
    pub full_name: String,
    pub name: String,
    pub owner: String,
    pub selected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubOrg {
    pub login: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwnerInfo {
    pub name: String,
    pub is_user: bool,
}