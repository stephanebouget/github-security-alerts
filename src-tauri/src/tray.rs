use tauri::{image::Image, Manager};

// ============================================================================
// Tray Icons
// ============================================================================

const ICON_GRAY: &[u8] = include_bytes!("../icons/tray/icon-gray.png");
const ICON_GREEN: &[u8] = include_bytes!("../icons/tray/icon-green.png");
const ICON_RED: &[u8] = include_bytes!("../icons/tray/icon-red.png");

pub fn generate_tray_icon(count: Option<usize>) -> Vec<u8> {
    match count {
        None => ICON_GRAY.to_vec(),
        Some(0) => ICON_GREEN.to_vec(),
        Some(_) => ICON_RED.to_vec(),
    }
}

#[tauri::command]
pub async fn update_tray_icon(
    app: tauri::AppHandle,
    alert_count: usize,
) -> Result<(), String> {
    if let Some(state) = app.try_state::<crate::state::AppState>() {
        let mut count = state.alert_count.lock().unwrap();
        *count = alert_count;
    }

    let icon_data = generate_tray_icon(Some(alert_count));

    if let Some(tray) = app.tray_by_id("main-tray") {
        let icon = Image::from_bytes(&icon_data).map_err(|e| e.to_string())?;
        tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;

        let tooltip = if alert_count == 0 {
            "GitHub Security Alerts - No alerts".to_string()
        } else {
            format!("GitHub Security Alerts - {} alert(s)!", alert_count)
        };
        tray.set_tooltip(Some(&tooltip)).map_err(|e| e.to_string())?;
    }

    if let Some(window) = app.get_webview_window("main") {
        let title = if alert_count > 0 {
            format!("GitHub Alerts - {} alert(s)", alert_count)
        } else {
            "GitHub Alerts".to_string()
        };
        let _ = window.set_title(&title);
    }

    Ok(())
}