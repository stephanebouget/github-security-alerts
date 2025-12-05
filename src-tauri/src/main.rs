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
};
use std::sync::Mutex;

// État global pour stocker le nombre d'alertes
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

// Génère une icône avec un nombre (badge)
fn generate_tray_icon(count: usize) -> Vec<u8> {
  use image::{Rgba, RgbaImage, ImageEncoder};
  
  let size = 32u32;
  let mut img = RgbaImage::new(size, size);
  
  // Couleur de fond: vert si 0, rouge sinon
  let bg_color = if count == 0 {
    Rgba([34u8, 197u8, 94u8, 255u8]) // Vert
  } else {
    Rgba([239u8, 68u8, 68u8, 255u8]) // Rouge
  };
  
  let text_color = Rgba([255u8, 255u8, 255u8, 255u8]); // Blanc
  
  // Remplir le fond avec la couleur
  for y in 0..size {
    for x in 0..size {
      img.put_pixel(x, y, bg_color);
    }
  }
  
  // Dessiner le chiffre en blanc au centre
  if count > 0 {
    let display_count = if count > 99 { 99 } else { count };
    draw_number(&mut img, display_count, text_color);
  } else {
    // Dessiner un checkmark pour 0 alertes
    draw_checkmark(&mut img, text_color);
  }
  
  // Convertir en PNG bytes
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

// Dessine un checkmark (✓) sur l'image
fn draw_checkmark(img: &mut image::RgbaImage, color: image::Rgba<u8>) {
  // Checkmark simple
  let points = [
    (8, 16), (9, 17), (10, 18), (11, 19), (12, 20),
    (13, 19), (14, 18), (15, 17), (16, 16), (17, 15),
    (18, 14), (19, 13), (20, 12), (21, 11), (22, 10),
    (23, 9),
    // Épaisseur
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

// Dessine un nombre sur l'image
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

// Dessine un chiffre individuel (0-9) avec une police bitmap simple
fn draw_digit(img: &mut image::RgbaImage, digit: u8, x: u32, y: u32, color: image::Rgba<u8>) {
  // Police bitmap 8x12 pour chaque chiffre
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
  // Mettre à jour l'état
  if let Some(state) = app.try_state::<AppState>() {
    let mut count = state.alert_count.lock().unwrap();
    *count = alertCount;
  }
  
  // Générer la nouvelle icône
  let icon_data = generate_tray_icon(alertCount);
  
  // Mettre à jour l'icône du tray
  if let Some(tray) = app.tray_by_id("main-tray") {
    let icon = Image::from_bytes(&icon_data).map_err(|e| e.to_string())?;
    tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;
    
    // Mettre à jour le tooltip
    let tooltip = if alertCount == 0 {
      "GitHub Security Alerts - No alerts".to_string()
    } else {
      format!("GitHub Security Alerts - {} alert(s)!", alertCount)
    };
    tray.set_tooltip(Some(&tooltip)).map_err(|e| e.to_string())?;
  }
  
  // Mettre à jour le titre de la fenêtre aussi
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
      // Créer le menu du tray
      let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&show, &quit])?;
      
      // Générer l'icône initiale (verte, 0 alertes)
      let icon_data = generate_tray_icon(0);
      let icon = Image::from_bytes(&icon_data)?;
      
      // Créer l'icône du tray
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
                let _ = window.show();
                let _ = window.set_focus();
              }
            }
            _ => {}
          }
        })
        .on_tray_icon_event(|tray, event| {
          if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
              let _ = window.show();
              let _ = window.set_focus();
            }
          }
        })
        .build(app)?;
      
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      hello_world_command,
      get_github_security_alerts,
      update_tray_icon
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
