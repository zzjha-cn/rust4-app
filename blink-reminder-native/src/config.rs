use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub blink_interval_sec: u64,
    pub rest_interval_min: u64,
    pub blink_animation_duration_sec: f64,
    pub rest_animation_duration_sec: f64,
    pub ripple_color: String,
    pub work_start_hour: u8,
    pub work_end_hour: u8,
    pub enable_work_hours: bool,
    pub enable_sound: bool,
    pub enable_blink: bool,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            blink_interval_sec: 40,
            rest_interval_min: 40,
            blink_animation_duration_sec: 1.8,
            rest_animation_duration_sec: 5.0,
            ripple_color: "#4FC3F7".to_string(),
            work_start_hour: 9,
            work_end_hour: 18,
            enable_work_hours: true,
            enable_sound: false,
            enable_blink: true,
            theme: "light".to_string(),
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".blink-reminder")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}
