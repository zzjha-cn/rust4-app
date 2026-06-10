use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{App, Emitter, Manager};
use tokio::time;

use crate::config::{AppConfig, AppConfigState};

pub struct TimerState {
    pub is_paused: AtomicBool,
}

pub fn setup_timer(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    // Spawn blink timer
    let h = handle.clone();
    tauri::async_runtime::spawn(async move {
        run_blink_timer(h).await;
    });

    // Spawn rest timer
    tauri::async_runtime::spawn(async move {
        run_rest_timer(handle).await;
    });

    Ok(())
}

async fn run_blink_timer(app: tauri::AppHandle) {
    loop {
        let config = app.state::<AppConfigState>().get();
        time::sleep(Duration::from_secs(config.blink_interval_sec.max(1))).await;
        if app.state::<TimerState>().is_paused.load(Ordering::Relaxed) {
            continue;
        }
        if is_within_work_hours(&config) {
            let _ = app.emit("reminder", "blink");
        }
    }
}

async fn run_rest_timer(app: tauri::AppHandle) {
    loop {
        let config = app.state::<AppConfigState>().get();
        time::sleep(Duration::from_secs((config.rest_interval_min * 60).max(1))).await;
        if app.state::<TimerState>().is_paused.load(Ordering::Relaxed) {
            continue;
        }
        if is_within_work_hours(&config) {
            let _ = app.emit("reminder", "rest");
        }
    }
}

fn is_within_work_hours(config: &AppConfig) -> bool {
    if !config.enable_work_hours {
        return true;
    }
    let now = current_hour();
    now >= config.work_start_hour && now < config.work_end_hour
}

fn current_hour() -> u8 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = since_epoch.as_secs();

    // Get local time offset by inspecting the difference between local time and UTC
    // This is a simple approach without external crates
    let local_offset_secs = local_offset_seconds();
    let local_secs = total_secs.saturating_add_signed(local_offset_secs as i64);
    ((local_secs / 3600) % 24) as u8
}

/// Get the local timezone offset in seconds using libc
fn local_offset_seconds() -> i64 {
    #[cfg(target_os = "macos")]
    {
        // Best effort: use libc timezone
        extern "C" {
            static timezone: i64;
            fn tzset();
        }
        unsafe {
            tzset();
            -timezone // timezone is seconds WEST of GMT, so negate
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        // On Windows, use UTC only as fallback
        0i64
    }
}
