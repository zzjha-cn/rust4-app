mod config;
mod render;
mod timer;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder,
};

#[derive(Debug)]
pub enum AppEvent {
    Blink,
    Rest,
    Hide,
    BlinkReplay,
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
    let quit_item = MenuItem::new("❌ 退出", true, None);
    let _ = tray_menu.append_items(&[&toggle_blink_item, &toggle_item, &quit_item]);

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Blink Reminder")
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
            }
        }

        match event {
            Event::UserEvent(AppEvent::Blink) => {
                println!("Blink on main thread!");
                let duration = config.read().unwrap().blink_animation_duration_sec;
                renderer.show_ripple(duration, proxy.clone(), false);
            }
            Event::UserEvent(AppEvent::BlinkReplay) => {
                println!("Blink replay on main thread!");
                let duration = config.read().unwrap().blink_animation_duration_sec;
                renderer.show_ripple(duration, proxy.clone(), true);
            }
            Event::UserEvent(AppEvent::Rest) => {
                println!("Rest on main thread!");
                // renderer.show_ripple(5.0, proxy.clone(), false);
            }
            Event::UserEvent(AppEvent::Hide) => {
                renderer.hide_ripple();
            }
            _ => {}
        }
    });
}
