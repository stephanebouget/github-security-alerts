use tauri::{PhysicalPosition, Manager, LogicalSize};
use std::time::Instant;
use crate::state::AppState;

// ============================================================================
// Window Management
// ============================================================================

pub fn position_window_near_tray(window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();
        let scale_factor = window.scale_factor().unwrap_or(1.0);

        // Logical size (will be scaled by DPI automatically by Tauri)
        let window_width = 420.0;
        let window_height = 600.0;
        let margin = 10i32;

        #[cfg(target_os = "macos")]
        {
            // On macOS, position window at top-right corner
            let x = monitor_position.x + monitor_size.width as i32 - (window_width * scale_factor) as i32 - margin;
            let y = monitor_position.y + margin;

            let _ = window.set_position(PhysicalPosition::new(x, y));
            let _ = window.set_size(LogicalSize::new(window_width, window_height));
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On Windows and Linux, position window at bottom-right corner
            let taskbar_height = 48i32;
            let x = monitor_position.x + monitor_size.width as i32 - (window_width * scale_factor) as i32 - margin;
            let y = monitor_position.y + monitor_size.height as i32 - (window_height * scale_factor) as i32 - taskbar_height - margin;

            let _ = window.set_position(PhysicalPosition::new(x, y));
            let _ = window.set_size(LogicalSize::new(window_width, window_height));
        }
    }
}

pub fn handle_window_focus_lost(window: &tauri::Window) {
    let app = window.app_handle();
    let should_hide = if let Some(state) = app.try_state::<AppState>() {
        let last_shown = state.last_shown.lock().unwrap();
        if let Some(instant) = *last_shown {
            instant.elapsed().as_millis() > 500
        } else {
            true
        }
    } else {
        true
    };

    if should_hide {
        let _ = window.hide();
    }
}

pub fn handle_window_show(app: &tauri::AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let mut last_shown = state.last_shown.lock().unwrap();
        *last_shown = Some(Instant::now());
    }
}