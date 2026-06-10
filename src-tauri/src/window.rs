use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub const REMINDER_WINDOW_LABEL: &str = "reminder";
pub const SETTINGS_WINDOW_LABEL: &str = "settings";

pub fn create_or_show_reminder(
    app: &AppHandle,
    reminder_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let label = REMINDER_WINDOW_LABEL;

    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        // 移除 window.set_focus()，防止抢占当前工作窗口的焦点
        let js = format!("window.startReminder?.('{}')", reminder_type);
        let _ = window.eval(&js);
        return Ok(());
    }

    let url = WebviewUrl::App(format!("index.html?type={}", reminder_type).into());

    let mut builder = WebviewWindowBuilder::new(app, label, url)
        .title("Blink Reminder")
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .shadow(false)
        .focused(false)
        .transparent(true);

    // Cover the whole screen without triggering macOS native fullscreen (which creates a new Space)
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let size = monitor.size();
        let position = monitor.position();
        builder = builder
            .inner_size(size.width as f64, size.height as f64)
            .position(position.x as f64, position.y as f64);
    }

    let _window = builder.build()?;

    // Set window to ignore mouse events so it doesn't steal focus
    let _ = _window.set_ignore_cursor_events(true);

    Ok(())
}

pub fn open_settings(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let label = SETTINGS_WINDOW_LABEL;

    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let url = WebviewUrl::App("settings.html".into());

    WebviewWindowBuilder::new(app, label, url)
        .title("Blink Reminder 设置")
        .inner_size(400.0, 500.0)
        .resizable(false)
        .center()
        .build()?;

    Ok(())
}

pub fn hide_reminder(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(REMINDER_WINDOW_LABEL) {
        let _ = window.hide();
    }
    Ok(())
}
