use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
pub async fn check_for_updates(app_handle: AppHandle) -> Result<bool, String> {
    println!("[UPDATE] Starting update check...");
    
    match app_handle.updater() {
        Ok(updater) => {
            println!("[UPDATE] Updater initialized successfully");
            match updater.check().await {
                Ok(Some(update)) => {
                    println!("[UPDATE] Update available: v{}", update.version);
                    println!("[UPDATE] Notes: {:?}", update.body);
                    println!("[UPDATE] Date: {:?}", update.date);
                    Ok(true)
                }
                Ok(None) => {
                    println!("[UPDATE] No update available (running latest version)");
                    Ok(false)
                }
                Err(e) => {
                    eprintln!("[UPDATE ERROR] Check failed with error:");
                    eprintln!("[UPDATE ERROR] Message: {}", e);
                    
                    let error_str = e.to_string();
                    if error_str.contains("Could not fetch") {
                        eprintln!("[UPDATE ERROR] Network issue: Check if latest.json exists in GitHub release");
                    } else if error_str.contains("Invalid") || error_str.contains("JSON") {
                        eprintln!("[UPDATE ERROR] Invalid JSON format in latest.json");
                    } else if error_str.contains("signature") {
                        eprintln!("[UPDATE ERROR] Signature verification failed");
                    } else if error_str.contains("invalid updater binary format") || error_str.contains("binary format") {
                        eprintln!("[UPDATE ERROR] Binary format mismatch: Check if correct platform-specific binaries are in GitHub release");
                        eprintln!("[UPDATE ERROR] Linux systems need .AppImage, .deb, or .tar.gz, not Windows .exe or .msi files");
                    }
                    
                    Err(e.to_string())
                }
            }
        }
        Err(e) => {
            eprintln!("[UPDATE ERROR] Updater initialization failed: {}", e);
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

                    update.install(&bytes).map_err(|e| {
                        let error_str = e.to_string();
                        if error_str.contains("invalid updater binary format") || error_str.contains("binary format") {
                            format!("Binary format error: {}\nThis usually means the wrong binary type was downloaded for your platform. Check GitHub release assets.", error_str)
                        } else {
                            error_str
                        }
                    })?;
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
