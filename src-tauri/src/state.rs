use std::sync::Mutex;
use std::time::Instant;
use crate::models::AppConfig;

pub struct AppState {
    pub alert_count: Mutex<usize>,
    pub last_shown: Mutex<Option<Instant>>,
    pub config: Mutex<AppConfig>,
    pub dev_tools_open: Mutex<bool>,
}