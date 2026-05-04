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

use tauri::{PhysicalPosition, LogicalSize};

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

// ============================================================================
// macOS Window Configuration
// ============================================================================

/// Configure the window for macOS tray-app behavior:
/// - Visible on all Spaces (never swept away by swipe gestures)
/// - Does not auto-hide when the app loses focus
#[cfg(target_os = "macos")]
pub fn set_macos_window_level(window: &tauri::WebviewWindow) {
    use objc2_app_kit::NSWindow;

    // Visible on all Spaces via Tauri native API
    // (sets NSWindowCollectionBehaviorCanJoinAllSpaces under the hood)
    let _ = window.set_visible_on_all_workspaces(true);

    // setHidesOnDeactivate is not exposed by Tauri — call via objc2-app-kit.
    // Prevents macOS from auto-hiding the window when the app loses focus.
    unsafe {
        let ns_window: &NSWindow = &*window
            .ns_window()
            .expect("Failed to get NSWindow handle")
            .cast();
        ns_window.setHidesOnDeactivate(false);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_macos_window_level(_window: &tauri::WebviewWindow) {
    // No-op on non-macOS platforms
}
