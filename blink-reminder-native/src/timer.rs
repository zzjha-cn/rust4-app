use std::collections::VecDeque;
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
        let mut window_history: VecDeque<bool> = VecDeque::new();
        let mut current_window_active = false;
        let mut elapsed_secs_in_window = 0;
        let mut secs_since_last_blink = 0;
        let mut waiting_for_idle_to_blink = false;

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let cfg = blink_config.read().unwrap().clone();
            if !cfg.enable_blink || blink_state.is_paused.load(Ordering::Relaxed) {
                continue;
            }

            // If we are waiting for the user to stop typing before showing the animation
            if waiting_for_idle_to_blink {
                if get_idle_time_secs() >= 1.0 {
                    if is_within_work_hours(&cfg) {
                        on_blink();
                    }
                    waiting_for_idle_to_blink = false;
                    // Reset history and timer after a successful blink
                    window_history.clear();
                    current_window_active = false;
                    elapsed_secs_in_window = 0;
                    secs_since_last_blink = 0;
                }
                continue;
            }

            secs_since_last_blink += 1;

            // Check activity in the current second
            if get_idle_time_secs() < 1.0 {
                current_window_active = true;
            }

            elapsed_secs_in_window += 1;

            // End of a time window (e.g., 3 seconds)
            if elapsed_secs_in_window >= cfg.time_window_sec {
                // Push the result of this window into the history
                window_history.push_back(current_window_active);

                // Calculate how many windows make up a full check cycle
                let max_windows_in_cycle = (cfg.blink_interval_sec / cfg.time_window_sec) as usize;

                // Maintain the sliding window size
                while window_history.len() > max_windows_in_cycle {
                    window_history.pop_front();
                }

                // Check if we have enough active windows in the current sliding history
                let active_windows_count = window_history.iter().filter(|&&active| active).count() as u64;

                // Only trigger if we've waited at least `blink_interval_sec` since the last blink
                // AND the user has been focused enough in the recent sliding window
                if secs_since_last_blink >= cfg.blink_interval_sec && active_windows_count >= cfg.active_window_threshold {
                    waiting_for_idle_to_blink = true;
                }

                // Reset for the next small window
                elapsed_secs_in_window = 0;
                current_window_active = false;
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
    use chrono::Timelike;
    chrono::Local::now().hour() as u8
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
