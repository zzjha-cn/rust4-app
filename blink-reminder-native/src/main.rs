mod config;
mod render;
mod timer;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
    TrayIconBuilder,
};

#[derive(Debug)]
pub enum AppEvent {
    Blink,
    Rest,
    Hide,
    BlinkReplay,
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

    let interval_menu = Submenu::new("⏱ 眨眼检查周期", true);
    let interval_20_item = MenuItem::new("20 秒", true, None);
    let interval_40_item = MenuItem::new("40 秒", true, None);
    let interval_60_item = MenuItem::new("60 秒", true, None);
    let _ = interval_menu.append_items(&[&interval_20_item, &interval_40_item, &interval_60_item]);

    let window_menu = Submenu::new("⏱ 判定窗口大小", true);
    let window_1_item = MenuItem::new("1 秒", true, None);
    let window_3_item = MenuItem::new("3 秒", true, None);
    let window_5_item = MenuItem::new("5 秒", true, None);
    let _ = window_menu.append_items(&[&window_1_item, &window_3_item, &window_5_item]);

    let threshold_menu = Submenu::new("🎯 专注活跃阈值", true);
    let threshold_5_item = MenuItem::new("5 次", true, None);
    let threshold_10_item = MenuItem::new("10 次", true, None);
    let threshold_15_item = MenuItem::new("15 次", true, None);
    let _ =
        threshold_menu.append_items(&[&threshold_5_item, &threshold_10_item, &threshold_15_item]);

    let rest_menu = Submenu::new("☕ 休息提醒间隔", true);
    let rest_20_item = MenuItem::new("20 分钟", true, None);
    let rest_30_item = MenuItem::new("30 分钟", true, None);
    let rest_40_item = MenuItem::new("40 分钟", true, None);
    let rest_60_item = MenuItem::new("60 分钟", true, None);
    let _ = rest_menu.append_items(&[&rest_20_item, &rest_30_item, &rest_40_item, &rest_60_item]);

    let quit_item = MenuItem::new("❌ 退出", true, None);

    let _ = tray_menu.append_items(&[
        &toggle_blink_item,
        &toggle_item,
        &PredefinedMenuItem::separator(),
        &test_blink_item,
        &test_rest_item,
        &PredefinedMenuItem::separator(),
        &interval_menu,
        &window_menu,
        &threshold_menu,
        &rest_menu,
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
            } else if menu_event.id == interval_20_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.blink_interval_sec = 20;
                let _ = cfg.save();
            } else if menu_event.id == interval_40_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.blink_interval_sec = 40;
                let _ = cfg.save();
            } else if menu_event.id == interval_60_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.blink_interval_sec = 60;
                let _ = cfg.save();
            } else if menu_event.id == window_1_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.time_window_sec = 1;
                let _ = cfg.save();
            } else if menu_event.id == window_3_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.time_window_sec = 3;
                let _ = cfg.save();
            } else if menu_event.id == window_5_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.time_window_sec = 5;
                let _ = cfg.save();
            } else if menu_event.id == threshold_5_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.active_window_threshold = 5;
                let _ = cfg.save();
            } else if menu_event.id == threshold_10_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.active_window_threshold = 10;
                let _ = cfg.save();
            } else if menu_event.id == threshold_15_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.active_window_threshold = 15;
                let _ = cfg.save();
            } else if menu_event.id == rest_20_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.rest_interval_min = 20;
                let _ = cfg.save();
            } else if menu_event.id == rest_30_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.rest_interval_min = 30;
                let _ = cfg.save();
            } else if menu_event.id == rest_40_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.rest_interval_min = 40;
                let _ = cfg.save();
            } else if menu_event.id == rest_60_item.id() {
                let mut cfg = config.write().unwrap();
                cfg.rest_interval_min = 60;
                let _ = cfg.save();
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
