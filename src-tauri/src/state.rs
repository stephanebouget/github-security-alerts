use std::sync::Mutex;
use std::time::Instant;
use crate::models::AppConfig;

pub struct AppState {
    pub alert_count: Mutex<usize>,
    pub last_shown: Mutex<Option<Instant>>,
    pub last_focus_lost: Mutex<Option<Instant>>,
    pub auto_hide_paused: Mutex<bool>,
    pub config: Mutex<AppConfig>,
}