mod config;
mod render;
mod timer;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

#[derive(Debug)]
pub enum AppEvent {
    Blink,
    Rest,
    Hide,
    BlinkReplay,
}

#[cfg(target_os = "macos")]
fn prompt_input(prompt: &str, default_val: &str) -> Option<String> {
    let script = format!(
        "text returned of (display dialog \"{}\" default answer \"{}\" buttons {{\"取消\", \"确定\"}} default button \"确定\")",
        prompt, default_val
    );
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
fn prompt_input(prompt: &str, default_val: &str) -> Option<String> {
    // Windows implementation can be added later
    None
}

fn load_icon() -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(include_bytes!("../../src-tauri/icons/32x32.png"))
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn main() {
    // Initialize tokio runtime for timers
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let config = Arc::new(RwLock::new(config::AppConfig::load()));
    let timer_state = Arc::new(timer::TimerState {
        is_paused: AtomicBool::new(false),
    });

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Setup Tray
    let tray_menu = Menu::new();

    let toggle_blink_item = MenuItem::new("👁 开启/关闭眨眼提醒", true, None);
    let toggle_item = MenuItem::new("⏸ 暂停 20 分钟", true, None);

    let test_blink_item = MenuItem::new("▶️ 测试眨眼动画", true, None);
    let test_rest_item = MenuItem::new("▶️ 测试休息动画", true, None);

    let interval_item = MenuItem::new("⏱ 设置眨眼检查周期...", true, None);
    let window_item = MenuItem::new("⏱ 设置判定窗口大小...", true, None);
    let threshold_item = MenuItem::new("🎯 设置专注活跃阈值...", true, None);
    let rest_item = MenuItem::new("☕ 设置休息提醒间隔...", true, None);

    let quit_item = MenuItem::new("❌ 退出", true, None);

    let _ = tray_menu.append_items(&[
        &toggle_blink_item,
        &toggle_item,
        &PredefinedMenuItem::separator(),
        &test_blink_item,
        &test_rest_item,
        &PredefinedMenuItem::separator(),
        &interval_item,
        &window_item,
        &threshold_item,
        &rest_item,
        &PredefinedMenuItem::separator(),
        &quit_item,
    ]);

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Blink Reminder")
        .with_icon(load_icon())
        .build()
        .unwrap();

    // Initialize Renderer
    let mut renderer = render::create_renderer();
    renderer.setup();

    let proxy_blink = proxy.clone();
    let proxy_rest = proxy.clone();
    timer::setup_timer(
        config.clone(),
        timer_state.clone(),
        move || {
            let _ = proxy_blink.send_event(AppEvent::Blink);
        },
        move || {
            let _ = proxy_rest.send_event(AppEvent::Rest);
        },
    );

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Handle tray menu events
        if let Ok(menu_event) = menu_channel.try_recv() {
            if menu_event.id == quit_item.id() {
                *control_flow = ControlFlow::Exit;
            } else if menu_event.id == toggle_blink_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.enable_blink = !cfg.enable_blink;
                let _ = cfg.save();
                println!("Blink enabled: {}", cfg.enable_blink);
            } else if menu_event.id == toggle_item.id() {
                let current = timer_state.is_paused.load(Ordering::Relaxed);
                timer_state.is_paused.store(!current, Ordering::Relaxed);
            } else if menu_event.id == test_blink_item.id() {
                let _ = proxy.send_event(AppEvent::Blink);
            } else if menu_event.id == test_rest_item.id() {
                let _ = proxy.send_event(AppEvent::Rest);
            } else if menu_event.id == interval_item.id() {
                let current_val = config.read().unwrap().blink_interval_sec.to_string();
                if let Some(input) = prompt_input("请输入眨眼检查周期（秒）：", &current_val)
                {
                    if let Ok(val) = input.parse::<u64>() {
                        let mut cfg = config.write().unwrap();
                        cfg.blink_interval_sec = val;
                        let _ = cfg.save();
                    }
                }
            } else if menu_event.id == window_item.id() {
                let current_val = config.read().unwrap().time_window_sec.to_string();
                if let Some(input) = prompt_input("请输入判定窗口大小（秒）：", &current_val)
                {
                    if let Ok(val) = input.parse::<u64>() {
                        let mut cfg = config.write().unwrap();
                        cfg.time_window_sec = val;
                        let _ = cfg.save();
                    }
                }
            } else if menu_event.id == threshold_item.id() {
                let current_val = config.read().unwrap().active_window_threshold.to_string();
                if let Some(input) = prompt_input("请输入专注活跃阈值（次）：", &current_val)
                {
                    if let Ok(val) = input.parse::<u64>() {
                        let mut cfg = config.write().unwrap();
                        cfg.active_window_threshold = val;
                        let _ = cfg.save();
                    }
                }
            } else if menu_event.id == rest_item.id() {
                let current_val = config.read().unwrap().rest_interval_min.to_string();
                if let Some(input) = prompt_input("请输入休息提醒间隔（分钟）：", &current_val)
                {
                    if let Ok(val) = input.parse::<u64>() {
                        let mut cfg = config.write().unwrap();
                        cfg.rest_interval_min = val;
                        let _ = cfg.save();
                    }
                }
            }
        }

        match event {
            Event::UserEvent(AppEvent::Blink) => {
                // println!("Blink on main thread!");
                let duration = config.read().unwrap().blink_animation_duration_sec;
                renderer.show_ripple(duration, proxy.clone(), false);
            }
            Event::UserEvent(AppEvent::BlinkReplay) => {
                // println!("Blink replay on main thread!");
                let duration = config.read().unwrap().blink_animation_duration_sec;
                renderer.show_ripple(duration, proxy.clone(), true);
            }
            Event::UserEvent(AppEvent::Rest) => {
                // println!("Rest on main thread!");
                let duration = config.read().unwrap().rest_animation_duration_sec;
                renderer.show_rest(duration, proxy.clone());
            }
            Event::UserEvent(AppEvent::Hide) => {
                renderer.hide_ripple();
            }
            _ => {}
        }
    });
}
