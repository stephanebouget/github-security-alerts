use tauri::{PhysicalPosition, Manager, PhysicalSize};
use std::time::Instant;
use crate::state::AppState;

// ============================================================================
// Window Management
// ============================================================================

pub fn position_window_near_tray(window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();

        let window_width = 420u32;
        let window_height = 600u32;
        let margin = 10i32;

        #[cfg(target_os = "macos")]
        {
            // On macOS, position window at top-right corner
            let x = monitor_position.x + monitor_size.width as i32 - window_width as i32 - margin;
            let y = monitor_position.y + margin;

            let _ = window.set_position(PhysicalPosition::new(x, y));
            let _ = window.set_size(PhysicalSize::new(window_width, window_height));
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On Windows and Linux, position window at bottom-right corner
            let taskbar_height = 48i32;
            let x = monitor_position.x + monitor_size.width as i32 - window_width as i32 - margin;
            let y = monitor_position.y + monitor_size.height as i32 - window_height as i32 - taskbar_height - margin;

            let _ = window.set_position(PhysicalPosition::new(x, y));
            let _ = window.set_size(PhysicalSize::new(window_width, window_height));
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