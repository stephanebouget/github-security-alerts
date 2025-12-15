use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
pub async fn check_for_updates(app_handle: AppHandle) -> Result<bool, String> {
    match app_handle.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    println!("[UPDATE] Available: {}", update.version);
                    Ok(true)
                }
                Ok(None) => {
                    println!("[UPDATE] No update available");
                    Ok(false)
                }
                Err(e) => {
                    eprintln!("[UPDATE ERROR] Check failed: {}", e);
                    Err(e.to_string())
                }
            }
        }
        Err(e) => {
            eprintln!("[UPDATE ERROR] Updater not configured: {}", e);
            Err(format!("Updater not available: {}", e))
        }
    }
}

#[tauri::command]
pub async fn install_update(app_handle: AppHandle) -> Result<(), String> {
    match app_handle.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    println!("[UPDATE] Installing: {}", update.version);
                    
                    let bytes = update
                        .download(
                            |chunk_length, content_length| {
                                println!("[DOWNLOAD] {} / {:?}", chunk_length, content_length);
                            },
                            || {
                                println!("[DOWNLOAD] Finished");
                            },
                        )
                        .await
                        .map_err(|e| e.to_string())?;

                    update.install(&bytes).map_err(|e| e.to_string())?;
                    app_handle.restart()
                }
                Ok(None) => Err("No update available".to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(format!("Updater not available: {}", e)),
    }
}

#[tauri::command]
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
