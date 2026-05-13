/*
 * Software Name : GitHub Security Alerts
 * SPDX-FileCopyrightText: Copyright (c) Orange
 * SPDX-License-Identifier: MIT
 * 
 * This software is distributed under the MIT,
 * see the "LICENSE.txt" file for more details or https://opensource.org/license/mit
 * 
 * Software description: A modern desktop application that monitors security vulnerabilities across your GitHub repositories in real-time.
 */

use tauri::Manager;
use crate::models::{AlertsResponse, RepoAlerts, GitHubAlert};
use crate::state::AppState;

// ============================================================================
// Security Alerts Commands
// ============================================================================

#[tauri::command]
pub async fn get_github_security_alerts(app: tauri::AppHandle) -> Result<AlertsResponse, String> {
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
        // Paginate through all pages (GitHub returns max 100 per page)
        let mut next_url: Option<String> = Some(format!(
            "https://api.github.com/repos/{}/dependabot/alerts?state=open&per_page=100",
            repo
        ));
        let mut all_alerts: Vec<GitHubAlert> = Vec::new();
        let mut fetch_error: Option<String> = None;
        let mut dependabot_enabled = true;

        'pagination: while let Some(url) = next_url.take() {
            match client
                .get(&url)
                .header("Accept", "application/vnd.github+json")
                .header("Authorization", format!("Bearer {}", token))
                .header("User-Agent", "github-security-alerts")
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();

                    if status == 422 {
                        eprintln!("[{}] Dependabot not enabled (HTTP 422)", repo);
                        dependabot_enabled = false;
                        break 'pagination;
                    } else if !status.is_success() {
                        let body = response.text().await.unwrap_or_default();
                        let msg = extract_error_message(status.as_u16(), &body);
                        eprintln!("[{}] GitHub API error: {}", repo, msg);
                        fetch_error = Some(msg);
                        break 'pagination;
                    } else {
                        // Extract next page URL from Link header before consuming body
                        let link_header = response
                            .headers()
                            .get("link")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string());

                        match response.bytes().await {
                            Ok(bytes) => {
                                match serde_json::from_slice::<Vec<GitHubAlert>>(&bytes) {
                                    Ok(page) => {
                                        all_alerts.extend(page);
                                        // Follow next page if present
                                        next_url = link_header.and_then(|h| parse_next_link(&h));
                                    }
                                    Err(e) => {
                                        let raw = String::from_utf8_lossy(&bytes);
                                        let msg = format!("JSON parse error: {} — body: {}", e, raw);
                                        eprintln!("[{}] {}", repo, msg);
                                        fetch_error = Some(format!("JSON parse error: {}", e));
                                        break 'pagination;
                                    }
                                }
                            }
                            Err(e) => {
                                let msg = format!("Error reading response body: {}", e);
                                eprintln!("[{}] {}", repo, msg);
                                fetch_error = Some(msg);
                                break 'pagination;
                            }
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Network error: {}", e);
                    eprintln!("[{}] {}", repo, msg);
                    fetch_error = Some(msg);
                    break 'pagination;
                }
            }
        }

        if !dependabot_enabled {
            repo_alerts.push(RepoAlerts {
                name: repo,
                alerts: 0,
                dev_alerts: 0,
                dependabot_enabled: false,
                error: None,
            });
        } else if let Some(msg) = fetch_error {
            repo_alerts.push(RepoAlerts {
                name: repo,
                alerts: 0,
                dev_alerts: 0,
                dependabot_enabled: false,
                error: Some(msg),
            });
        } else {
            let open_count = all_alerts.len();
            let dev_count = all_alerts.iter()
                .filter(|a| a.dependency.scope.as_deref() == Some("development"))
                .count();
            total_alerts += open_count;
            repo_alerts.push(RepoAlerts {
                name: repo,
                alerts: open_count,
                dev_alerts: dev_count,
                dependabot_enabled: true,
                error: None,
            });
        }
    }

    Ok(AlertsResponse {
        total_alerts,
        repos: repo_alerts,
    })
}

/// Parse the `Link` header from GitHub's API to extract the next page URL.
/// Example header: `<https://api.github.com/...?page=2>; rel="next", <...>; rel="last"`
fn parse_next_link(link_header: &str) -> Option<String> {
    link_header.split(',').find_map(|part| {
        let mut url = None;
        let mut is_next = false;
        for segment in part.split(';') {
            let segment = segment.trim();
            if segment.starts_with('<') && segment.ends_with('>') {
                url = Some(segment[1..segment.len() - 1].to_string());
            } else if segment == "rel=\"next\"" {
                is_next = true;
            }
        }
        if is_next { url } else { None }
    })
}

/// Extract a human-readable error message from a GitHub API error response.
/// Tries to parse the body as JSON and return the `message` field.
/// Falls back to `HTTP {status}` if parsing fails or the field is absent.
fn extract_error_message(status: u16, body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
            return msg.to_string();
        }
    }
    format!("HTTP {status}")
}
