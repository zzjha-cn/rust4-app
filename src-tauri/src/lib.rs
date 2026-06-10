mod commands;
mod config;
mod timer;
mod window;

use config::{AppConfig, AppConfigState};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Listener, Manager,
};
use timer::TimerState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load();

    tauri::Builder::default()
        .manage(AppConfigState::new(config))
        .manage(TimerState {
            is_paused: std::sync::atomic::AtomicBool::new(false),
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Setup system tray
            setup_tray(app)?;

            // Setup timers
            timer::setup_timer(app)?;

            // Listen for reminder events from timer
            let handle = app.handle().clone();
            app.listen("reminder", move |event| {
                let reminder_type = event.payload().trim_matches('"').to_string();
                let _ = window::create_or_show_reminder(&handle, &reminder_type);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::show_reminder,
            commands::get_config,
            commands::update_config,
            commands::pause_reminders,
            commands::resume_reminders,
            commands::hide_reminder_window,
            commands::open_settings_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItem::with_id(app, "toggle", "⏸ 暂停 20 分钟", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume", "▶ 开启提醒", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "⚙ 设置...", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "❌ 退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle, &resume, &settings, &quit])?;

    // Try to use the default window icon, fallback to generated icon
    let icon = app.default_window_icon().cloned().unwrap_or_else(|| {
        // Generate a simple 32x32 blue circle as fallback
        let size = 32u32;
        let mut rgba = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let dx = x as f64 - size as f64 / 2.0;
                let dy = y as f64 - size as f64 / 2.0;
                let dist = (dx * dx + dy * dy).sqrt();
                let idx = ((y * size + x) * 4) as usize;
                if dist < size as f64 / 2.0 - 1.0 {
                    rgba[idx] = 79; // R
                    rgba[idx + 1] = 195; // G
                    rgba[idx + 2] = 247; // B
                    rgba[idx + 3] = 255; // A
                }
            }
        }
        tauri::image::Image::new_owned(rgba, size, size)
    });

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("Blink Reminder")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle" => {
                let _ = commands::pause_reminders(app.state());
                let h = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(20 * 60)).await;
                    let _ = commands::resume_reminders(h.state());
                });
            }
            "resume" => {
                let _ = commands::resume_reminders(app.state());
            }
            "settings" => {
                let _ = commands::open_settings_window(app.clone());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    app.on_tray_icon_event(move |_tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            // Left click - could show info
        }
    });

    Ok(())
}
