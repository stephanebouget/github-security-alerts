use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use std::time::Duration;

#[tauri::command]
pub async fn check_for_updates(app_handle: AppHandle) -> Result<bool, String> {
    match app_handle.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    println!("✅ Update available from GitHub: {}", update.version);
                    Ok(true)
                }
                Ok(None) => {
                    println!("📦 No update available from GitHub");
                    Ok(false)
                }
                Err(e) => {
                    eprintln!("❌ Failed to check GitHub for updates: {}", e);
                    // If signature verification fails, provide helpful error
                    if e.to_string().contains("signature") || e.to_string().contains("public key") {
                        return Err("Signature verification failed. For development, you can disable signing in tauri.conf.json".to_string());
                    }
                    Err(e.to_string())
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️ Updater not configured: {}", e);
            Err(format!("Updater not available: {} (Check tauri.conf.json configuration)", e))
        }
    }
}

#[tauri::command]
pub async fn install_update(app_handle: AppHandle) -> Result<(), String> {
    match app_handle.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    println!("Installing update: {}", update.version);
                    
                    // Download the update
                    let mut downloaded = 0;
                    let bytes = update
                        .download(
                            |chunk_length, content_length| {
                                downloaded += chunk_length;
                                println!("downloaded {downloaded} from {content_length:?}");
                            },
                            || {
                                println!("download finished");
                            },
                        )
                        .await
                        .map_err(|e| e.to_string())?;

                    println!("Update downloaded successfully");
                    
                    // Install the update
                    update.install(&bytes).map_err(|e| e.to_string())?;
                    
                    // Restart the application
                    app_handle.restart();
                }
                Ok(None) => {
                    Err("No update available to install".to_string())
                }
                Err(e) => {
                    eprintln!("Failed to check for updates: {}", e);
                    Err(e.to_string())
                }
            }
        }
        Err(e) => {
            eprintln!("Updater not available: {}", e);
            Err(format!("Updater not available: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn start_background_update_checker(app_handle: AppHandle) {
    let handle = app_handle.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Check every hour
        
        loop {
            interval.tick().await;
            
            // Check for updates silently from GitHub releases
            if let Ok(update_available) = check_for_updates(handle.clone()).await {
                if update_available {
                    println!("Silent update check: GitHub release update available, installing automatically...");
                    
                    // Wait a bit to avoid potential conflicts
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    
                    // Install update silently
                    match install_update(handle.clone()).await {
                        Ok(_) => {
                            println!("Silent update from GitHub completed successfully");
                        }
                        Err(e) => {
                            eprintln!("Failed to install silent update from GitHub: {}", e);
                        }
                    }
                }
            }
        }
    });
}