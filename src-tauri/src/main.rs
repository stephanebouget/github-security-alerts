#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use tauri::{
  Manager,
  tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
  menu::{Menu, MenuItem},
  image::Image,
  PhysicalPosition,
};
use std::sync::Mutex;

// Global state to store the number of alerts
struct AppState {
  alert_count: Mutex<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AlertsResponse {
  total_alerts: usize,
  repos: Vec<RepoAlerts>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RepoAlerts {
  name: String,
  alerts: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubAlert {
  state: String,
}

#[tauri::command]
async fn hello_world_command(_app: tauri::AppHandle) -> Result<String, String> {
  println!("I was invoked from JS!");
  Ok("Hello world from Tauri!".into())
}

#[tauri::command]
async fn get_github_security_alerts(
  github_token: String,
) -> Result<AlertsResponse, String> {
  let token = github_token;
  let repos = vec![
    "KhiopsML/khiops-visualization",
    "KhiopsML/khiops-visualization-desktop",
    "stephanebouget/github-security-alerts",
    "KhiopsML/kv-electron",
    "stephanebouget/powo-cli",
    "KhiopsML/ngx-khiops-histogram"
  ];

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
        match response.json::<Vec<GitHubAlert>>().await {
          Ok(alerts) => {
            let open_alerts = alerts.iter()
              .filter(|a| a.state == "open")
              .count();
            total_alerts += open_alerts;
            repo_alerts.push(RepoAlerts {
              name: repo.to_string(),
              alerts: open_alerts,
            });
          }
          Err(e) => {
            eprintln!("Failed to parse alerts for {}: {}", repo, e);
            repo_alerts.push(RepoAlerts {
              name: repo.to_string(),
              alerts: 0,
            });
          }
        }
      }
      Err(e) => {
        eprintln!("Failed to fetch alerts for {}: {}", repo, e);
        repo_alerts.push(RepoAlerts {
          name: repo.to_string(),
          alerts: 0,
        });
      }
    }
  }

  Ok(AlertsResponse {
    total_alerts,
    repos: repo_alerts,
  })
}

// Generates an icon with a number (badge)
fn generate_tray_icon(count: usize) -> Vec<u8> {
  use image::{Rgba, RgbaImage, ImageEncoder};
  
  let size = 32u32;
  let mut img = RgbaImage::new(size, size);
  
  // Background color: green if 0, red otherwise
  let bg_color = if count == 0 {
    Rgba([34u8, 197u8, 94u8, 255u8]) // Green
  } else {
    Rgba([239u8, 68u8, 68u8, 255u8]) // Red
  };
  
  let text_color = Rgba([255u8, 255u8, 255u8, 255u8]); // White
  
  // Fill the background with the color
  for y in 0..size {
    for x in 0..size {
      img.put_pixel(x, y, bg_color);
    }
  }
  
  // Draw the number in white at the center
  if count > 0 {
    let display_count = if count > 99 { 99 } else { count };
    draw_number(&mut img, display_count, text_color);
  } else {
    // Draw a checkmark for 0 alerts
    draw_checkmark(&mut img, text_color);
  }
  
  // Convert to PNG bytes
  let mut png_data = Vec::new();
  let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
  encoder.write_image(
    img.as_raw(),
    size,
    size,
    image::ColorType::Rgba8
  ).unwrap();
  
  png_data
}

// Draws a checkmark (✓) on the image
fn draw_checkmark(img: &mut image::RgbaImage, color: image::Rgba<u8>) {
  // Simple checkmark
  let points = [
    (8, 16), (9, 17), (10, 18), (11, 19), (12, 20),
    (13, 19), (14, 18), (15, 17), (16, 16), (17, 15),
    (18, 14), (19, 13), (20, 12), (21, 11), (22, 10),
    (23, 9),
    // Thickness
    (8, 17), (9, 18), (10, 19), (11, 20), (12, 21),
    (13, 20), (14, 19), (15, 18), (16, 17), (17, 16),
    (18, 15), (19, 14), (20, 13), (21, 12), (22, 11),
    (23, 10),
  ];
  
  for (x, y) in points {
    if x < 32 && y < 32 {
      img.put_pixel(x, y, color);
    }
  }
}

// Draws a number on the image
fn draw_number(img: &mut image::RgbaImage, num: usize, color: image::Rgba<u8>) {
  let digits: Vec<u8> = if num == 0 {
    vec![0]
  } else {
    let mut n = num;
    let mut d = Vec::new();
    while n > 0 {
      d.push((n % 10) as u8);
      n /= 10;
    }
    d.reverse();
    d
  };
  
  let digit_width = 10;
  let total_width = digits.len() as u32 * digit_width;
  let start_x = (32 - total_width) / 2;
  let start_y = 6u32;
  
  for (i, digit) in digits.iter().enumerate() {
    let x_offset = start_x + (i as u32 * digit_width);
    draw_digit(img, *digit, x_offset, start_y, color);
  }
}

// Draws an individual digit (0-9) with a simple bitmap font
fn draw_digit(img: &mut image::RgbaImage, digit: u8, x: u32, y: u32, color: image::Rgba<u8>) {
  // Bitmap font 8x12 for each digit
  let patterns: [&[u16]; 10] = [
    // 0
    &[
      0b01111110,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b01111110,
    ],
    // 1
    &[
      0b00011000,
      0b00111000,
      0b01111000,
      0b00011000,
      0b00011000,
      0b00011000,
      0b00011000,
      0b00011000,
      0b00011000,
      0b00011000,
      0b00011000,
      0b01111110,
    ],
    // 2
    &[
      0b01111110,
      0b11000011,
      0b00000011,
      0b00000011,
      0b00000110,
      0b00001100,
      0b00011000,
      0b00110000,
      0b01100000,
      0b11000000,
      0b11000000,
      0b11111111,
    ],
    // 3
    &[
      0b01111110,
      0b11000011,
      0b00000011,
      0b00000011,
      0b00000110,
      0b00111100,
      0b00000110,
      0b00000011,
      0b00000011,
      0b00000011,
      0b11000011,
      0b01111110,
    ],
    // 4
    &[
      0b00000110,
      0b00001110,
      0b00011110,
      0b00110110,
      0b01100110,
      0b11000110,
      0b11111111,
      0b00000110,
      0b00000110,
      0b00000110,
      0b00000110,
      0b00000110,
    ],
    // 5
    &[
      0b11111111,
      0b11000000,
      0b11000000,
      0b11000000,
      0b11111110,
      0b00000011,
      0b00000011,
      0b00000011,
      0b00000011,
      0b00000011,
      0b11000011,
      0b01111110,
    ],
    // 6
    &[
      0b00111110,
      0b01100000,
      0b11000000,
      0b11000000,
      0b11111110,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b01111110,
    ],
    // 7
    &[
      0b11111111,
      0b00000011,
      0b00000011,
      0b00000110,
      0b00001100,
      0b00011000,
      0b00110000,
      0b00110000,
      0b00110000,
      0b00110000,
      0b00110000,
      0b00110000,
    ],
    // 8
    &[
      0b01111110,
      0b11000011,
      0b11000011,
      0b11000011,
      0b01111110,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b01111110,
    ],
    // 9
    &[
      0b01111110,
      0b11000011,
      0b11000011,
      0b11000011,
      0b11000011,
      0b01111111,
      0b00000011,
      0b00000011,
      0b00000011,
      0b00000011,
      0b00000110,
      0b01111100,
    ],
  ];
  
  let pattern = patterns[digit as usize];
  
  for (row, &bits) in pattern.iter().enumerate() {
    for col in 0..8 {
      if (bits >> (7 - col)) & 1 == 1 {
        let px = x + col;
        let py = y + row as u32;
        if px < 32 && py < 32 {
          img.put_pixel(px, py, color);
        }
      }
    }
  }
}

#[tauri::command]
async fn update_tray_icon(
  app: tauri::AppHandle,
  alertCount: usize,
) -> Result<(), String> {
  // Update the state
  if let Some(state) = app.try_state::<AppState>() {
    let mut count = state.alert_count.lock().unwrap();
    *count = alertCount;
  }
  
  // Generate the new icon
  let icon_data = generate_tray_icon(alertCount);
  
  // Update the tray icon
  if let Some(tray) = app.tray_by_id("main-tray") {
    let icon = Image::from_bytes(&icon_data).map_err(|e| e.to_string())?;
    tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;
    
    // Update the tooltip
    let tooltip = if alertCount == 0 {
      "GitHub Security Alerts - No alerts".to_string()
    } else {
      format!("GitHub Security Alerts - {} alert(s)!", alertCount)
    };
    tray.set_tooltip(Some(&tooltip)).map_err(|e| e.to_string())?;
  }
  
  // Update the window title as well
  if let Some(window) = app.get_webview_window("main") {
    let title = if alertCount > 0 {
      format!("GitHub Alerts - {} alert(s)", alertCount)
    } else {
      "GitHub Alerts".to_string()
    };
    let _ = window.set_title(&title);
  }

  Ok(())
}

fn main() {
  tauri::Builder::default()
    .manage(AppState {
      alert_count: Mutex::new(0),
    })
    .setup(|app| {
      // Create the tray menu
      let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
      let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&show, &hide, &quit])?;
      
      // Generate the initial icon (green, 0 alerts)
      let icon_data = generate_tray_icon(0);
      let icon = Image::from_bytes(&icon_data)?;
      
      // Hide the main window at startup
      if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
      }
      
      // Create the tray icon
      let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("GitHub Security Alerts")
        .on_menu_event(|app, event| {
          match event.id.as_ref() {
            "quit" => {
              app.exit(0);
            }
            "show" => {
              if let Some(window) = app.get_webview_window("main") {
                position_window_near_tray(&window);
                let _ = window.show();
                let _ = window.set_focus();
              }
            }
            "hide" => {
              if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
              }
            }
            _ => {}
          }
        })
        .on_tray_icon_event(|tray, event| {
          if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
              // Toggle visibility
              if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
              } else {
                position_window_near_tray(&window);
                let _ = window.show();
                let _ = window.set_focus();
              }
            }
          }
        })
        .build(app)?;
      
      Ok(())
    })
    .on_window_event(|window, event| {
      // Intercept window close to hide it instead of quitting
      if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        let _ = window.hide();
        api.prevent_close();
      }
      // Hide the window if it loses focus
      if let tauri::WindowEvent::Focused(focused) = event {
        if !focused {
          let _ = window.hide();
        }
      }
    })
    .invoke_handler(tauri::generate_handler![
      hello_world_command,
      get_github_security_alerts,
      update_tray_icon
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

// Position the window at the bottom right of the screen, near the system tray
fn position_window_near_tray(window: &tauri::WebviewWindow) {
  if let Ok(Some(monitor)) = window.primary_monitor() {
    let monitor_size = monitor.size();
    let monitor_position = monitor.position();
    
    // Window size
    let window_width = 420i32;
    let window_height = 500i32;
    
    // Margin from the edge (to avoid sticking to the screen edge)
    let margin = 10i32;
    
    // Position at the bottom right
    // On Windows, the taskbar is usually at the bottom with a height of about 48px
    let taskbar_height = 48i32;
    
    let x = monitor_position.x + monitor_size.width as i32 - window_width - margin;
    let y = monitor_position.y + monitor_size.height as i32 - window_height - taskbar_height - margin;
    
    let _ = window.set_position(PhysicalPosition::new(x, y));
  }
}
