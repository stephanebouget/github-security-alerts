use tauri::command;
use std::process::Command;

// ============================================================================
// System Settings
// ============================================================================

#[command]
pub fn open_taskbar_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/C", "start", "ms-settings:taskbar"])
            .spawn()
            .map_err(|e| format!("Failed to open taskbar settings: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Taskbar settings are only available on Windows".to_string())
    }
}
