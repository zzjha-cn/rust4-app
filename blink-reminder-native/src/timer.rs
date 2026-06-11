use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::config::AppConfig;

pub struct TimerState {
    pub is_paused: AtomicBool,
}

pub fn setup_timer(config: Arc<RwLock<AppConfig>>, state: Arc<TimerState>, on_blink: impl Fn() + Send + Sync + 'static, on_rest: impl Fn() + Send + Sync + 'static) {
    let blink_state = state.clone();
    let blink_config = config.clone();
    tokio::spawn(async move {
        let mut elapsed_secs = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let cfg = blink_config.read().unwrap().clone();
            if !cfg.enable_blink || blink_state.is_paused.load(Ordering::Relaxed) {
                continue;
            }

            elapsed_secs += 1;
            if elapsed_secs >= cfg.blink_interval_sec {
                if get_idle_time_secs() >= 1.0 {
                    elapsed_secs = 0;
                    if is_within_work_hours(&cfg) {
                        on_blink();
                    }
                } else {
                    // User is active, delay the blink by not resetting elapsed_secs
                    // It will check again in the next second
                }
            }
        }
    });

    let rest_state = state.clone();
    let rest_config = config.clone();
    tokio::spawn(async move {
        let mut elapsed_secs = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let cfg = rest_config.read().unwrap().clone();
            if rest_state.is_paused.load(Ordering::Relaxed) {
                continue;
            }

            elapsed_secs += 1;
            if elapsed_secs >= cfg.rest_interval_min * 60 {
                if get_idle_time_secs() >= 1.0 {
                    elapsed_secs = 0;
                    if is_within_work_hours(&cfg) {
                        on_rest();
                    }
                } else {
                    // User is active, delay the rest by not resetting elapsed_secs
                }
            }
        }
    });
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
    let local_offset_secs = local_offset_seconds();
    let local_secs = total_secs.saturating_add_signed(local_offset_secs as i64);
    ((local_secs / 3600) % 24) as u8
}

fn local_offset_seconds() -> i64 {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            static timezone: i64;
            fn tzset();
        }
        unsafe {
            tzset();
            -timezone
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        0i64
    }
}

fn get_idle_time_secs() -> f64 {
    #[cfg(target_os = "macos")]
    {
        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGEventSourceSecondsSinceLastEventType(sourceState: u32, eventType: u32) -> f64;
        }
        unsafe {
            // kCGEventSourceStateHIDSystemState = 1
            // kCGAnyInputEventType = 0xFFFFFFFF
            CGEventSourceSecondsSinceLastEventType(1, 0xFFFFFFFF)
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        // TODO: Implement for Windows using GetLastInputInfo
        1.0
    }
}
