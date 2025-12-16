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
                if response.status() == 422 {
                    // 422 Unprocessable Entity usually means Dependabot is not enabled
                    eprintln!("Dependabot not enabled for {}", repo);
                    repo_alerts.push(RepoAlerts {
                        name: repo,
                        alerts: 0,
                        dependabot_enabled: false,
                    });
                } else {
                    match response.json::<Vec<GitHubAlert>>().await {
                        Ok(alerts) => {
                            let open_alerts = alerts.iter()
                                .filter(|a| a.state == "open")
                                .count();
                            total_alerts += open_alerts;
                            repo_alerts.push(RepoAlerts {
                                name: repo,
                                alerts: open_alerts,
                                dependabot_enabled: true,
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to parse alerts for {}: {}", repo, e);
                            // If parsing fails, assume Dependabot is not enabled
                            repo_alerts.push(RepoAlerts {
                                name: repo,
                                alerts: 0,
                                dependabot_enabled: false,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch alerts for {}: {}", repo, e);
                repo_alerts.push(RepoAlerts {
                    name: repo,
                    alerts: 0,
                    dependabot_enabled: false,
                });
            }
        }
    }

    Ok(AlertsResponse {
        total_alerts,
        repos: repo_alerts,
    })
}