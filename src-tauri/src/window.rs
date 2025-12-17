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
    
    // Check if auto-hide is paused (for dropdown interactions)
    let is_paused = if let Some(state) = app.try_state::<AppState>() {
        let auto_hide_paused = state.auto_hide_paused.lock().unwrap();
        *auto_hide_paused
    } else {
        false
    };

    if is_paused {
        println!("[WINDOW] Auto-hide paused - ignoring focus loss");
        return;
    }
    
    // On Linux, use a delayed hide approach to handle dropdown interactions
    #[cfg(target_os = "linux")]
    {
        let should_hide = if let Some(state) = app.try_state::<AppState>() {
            let last_shown = state.last_shown.lock().unwrap();
            if let Some(instant) = *last_shown {
                instant.elapsed().as_millis() > 1000 // 1 second minimum on Linux
            } else {
                true
            }
        } else {
            true
        };

        if should_hide {
            let window_clone = window.clone();
            let app_clone = app.clone();
            
            // Store the focus lost time
            if let Some(state) = app.try_state::<AppState>() {
                let mut last_focus_lost = state.last_focus_lost.lock().unwrap();
                *last_focus_lost = Some(Instant::now());
            }
            
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(300)); // Wait 300ms
                
                // Check if auto-hide is still not paused and focus wasn't regained
                let should_still_hide = if let Some(state) = app_clone.try_state::<AppState>() {
                    let auto_hide_paused = state.auto_hide_paused.lock().unwrap();
                    if *auto_hide_paused {
                        return; // Auto-hide was paused during the delay
                    }
                    
                    let last_focus_lost = state.last_focus_lost.lock().unwrap();
                    if let Some(focus_lost_time) = *last_focus_lost {
                        // If more than 300ms have passed since focus lost and focus wasn't regained, hide
                        focus_lost_time.elapsed().as_millis() >= 300
                    } else {
                        false // Focus was regained
                    }
                } else {
                    true
                };
                
                if should_still_hide {
                    if let Ok(is_focused) = window_clone.is_focused() {
                        if !is_focused {
                            let _ = window_clone.hide();
                        }
                    }
                }
            });
        }
    }
    
    // On other platforms, use the original logic
    #[cfg(not(target_os = "linux"))]
    {
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
}

pub fn handle_window_show(app: &tauri::AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let mut last_shown = state.last_shown.lock().unwrap();
        *last_shown = Some(Instant::now());
    }
}

// ============================================================================
// Focus Management Commands (Linux dropdown fix)
// ============================================================================

#[tauri::command]
pub fn pause_auto_hide(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(state) = app.try_state::<AppState>() {
        let mut auto_hide_paused = state.auto_hide_paused.lock().unwrap();
        *auto_hide_paused = true;
        println!("[WINDOW] Auto-hide paused");
    }
    Ok(())
}

#[tauri::command]
pub fn resume_auto_hide(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(state) = app.try_state::<AppState>() {
        let mut auto_hide_paused = state.auto_hide_paused.lock().unwrap();
        *auto_hide_paused = false;
        println!("[WINDOW] Auto-hide resumed");
    }
    Ok(())
}