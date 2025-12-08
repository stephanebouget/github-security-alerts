use tauri::Manager;
use crate::models::{GitHubOrg, GitHubUser, GitHubRepo, OwnerInfo, RepoInfo};
use crate::state::AppState;
use crate::config::save_config;

// ============================================================================
// Repository Management Commands
// ============================================================================

/// Get list of owners (user + organizations)
#[tauri::command]
pub async fn get_owners(app: tauri::AppHandle) -> Result<Vec<OwnerInfo>, String> {
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
pub async fn get_repos_for_owner(app: tauri::AppHandle, owner: String, is_user: bool) -> Result<Vec<RepoInfo>, String> {
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
pub async fn set_selected_repos(app: tauri::AppHandle, repos: Vec<String>) -> Result<(), String> {
    if let Some(state) = app.try_state::<AppState>() {
        let mut config = state.config.lock().unwrap();
        config.selected_repos = repos;
        save_config(&config)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_selected_repos(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let state = app.try_state::<AppState>().ok_or("No state")?;
    let config = state.config.lock().unwrap();
    Ok(config.selected_repos.clone())
}