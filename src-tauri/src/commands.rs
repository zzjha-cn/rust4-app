use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

use crate::config::{AppConfig, AppConfigState};
use crate::timer::TimerState;
use crate::window;

#[tauri::command]
pub fn show_reminder(app: AppHandle, reminder_type: String) -> Result<(), String> {
    window::create_or_show_reminder(&app, &reminder_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config(config: State<'_, AppConfigState>) -> Result<AppConfig, String> {
    Ok(config.get())
}

#[tauri::command]
pub fn update_config(
    new_config: AppConfig,
    config_state: State<'_, AppConfigState>,
) -> Result<(), String> {
    new_config.save()?;
    config_state.set(new_config);
    Ok(())
}

#[tauri::command]
pub fn pause_reminders(state: State<'_, TimerState>) -> Result<(), String> {
    state.is_paused.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn resume_reminders(state: State<'_, TimerState>) -> Result<(), String> {
    state.is_paused.store(false, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn hide_reminder_window(app: AppHandle) -> Result<(), String> {
    window::hide_reminder(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_settings_window(app: AppHandle) -> Result<(), String> {
    window::open_settings(&app).map_err(|e| e.to_string())
}
