use super::RippleRenderer;
use crate::AppEvent;
use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopProxy;

#[cfg(target_os = "windows")]
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*, Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN,
    Win32::Graphics::Gdi::ValidateRect, Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
};

#[cfg(target_os = "windows")]
struct WinState {
    hwnd: HWND,
    factory: ID2D1Factory,
    render_target: Option<ID2D1HwndRenderTarget>,
    brush: Option<ID2D1SolidColorBrush>,
    is_rest: bool,
    ripple_radius: f32,
    ripple_max_radius: f32,
    ripple_active: bool,
}

#[cfg(target_os = "windows")]
unsafe impl Send for WinState {}
#[cfg(target_os = "windows")]
unsafe impl Sync for WinState {}

pub struct WinRenderer {
    #[cfg(target_os = "windows")]
    state: Arc<Mutex<WinState>>,
    #[cfg(not(target_os = "windows"))]
    dummy: (),
}

impl WinRenderer {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                let factory: ID2D1Factory =
                    D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED, None).unwrap();

                Self {
                    state: Arc::new(Mutex::new(WinState {
                        hwnd: HWND(0),
                        factory,
                        render_target: None,
                        brush: None,
                        is_rest: false,
                        ripple_radius: 0.0,
                        ripple_max_radius: 1000.0,
                        ripple_active: false,
                    })),
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self { dummy: () }
        }
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_PAINT {
        ValidateRect(hwnd, None);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

#[cfg(target_os = "windows")]
unsafe fn create_render_target(
    factory: &ID2D1Factory,
    hwnd: HWND,
    width: u32,
    height: u32,
) -> Result<ID2D1HwndRenderTarget> {
    let render_target_props = D2D1_RENDER_TARGET_PROPERTIES {
        r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_UNKNOWN,
            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        dpiX: 0.0,
        dpiY: 0.0,
        usage: D2D1_RENDER_TARGET_USAGE_NONE,
        minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
    };

    let hwnd_rt_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
        hwnd,
        pixelSize: D2D_SIZE_U { width, height },
        presentOptions: D2D1_PRESENT_OPTIONS_IMMEDIATELY,
    };

    factory.CreateHwndRenderTarget(&render_target_props, &hwnd_rt_props)
}

#[cfg(target_os = "windows")]
unsafe fn clear_target(rt: &ID2D1HwndRenderTarget, color: &D2D1_COLOR_F) {
    let base: ID2D1RenderTarget = rt.cast().unwrap();
    base.Clear(Some(color));
}

#[cfg(target_os = "windows")]
unsafe fn target_size(rt: &ID2D1HwndRenderTarget) -> D2D_SIZE_F {
    let base: ID2D1RenderTarget = rt.cast().unwrap();
    base.GetSize()
}

#[cfg(target_os = "windows")]
unsafe fn draw_ellipse_outline(
    rt: &ID2D1HwndRenderTarget,
    ellipse: &D2D1_ELLIPSE,
    brush: &ID2D1SolidColorBrush,
    stroke_width: f32,
) {
    let base: ID2D1RenderTarget = rt.cast().unwrap();
    let brush_base: ID2D1Brush = brush.cast().unwrap();
    base.DrawEllipse(ellipse, &brush_base, stroke_width, None);
}

impl RippleRenderer for WinRenderer {
    fn setup(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            let instance = GetModuleHandleW(None).unwrap();
            let class_name = w!("BlinkReminderOverlay");

            let wc = WNDCLASSW {
                lpfnWndProc: Some(wnd_proc),
                hInstance: instance.into(),
                lpszClassName: class_name,
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                ..Default::default()
            };

            let _ = RegisterClassW(&wc);

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name,
                w!("BlinkReminder"),
                WS_POPUP | WS_VISIBLE,
                0,
                0,
                screen_w,
                screen_h,
                None,
                None,
                instance,
                None,
            );

            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_COLORKEY | LWA_ALPHA);

            let mut state = self.state.lock().unwrap();
            state.hwnd = hwnd;
            state.ripple_max_radius = (screen_w.max(screen_h) as f32) * 1.2;

            let rt = create_render_target(&state.factory, hwnd, screen_w as u32, screen_h as u32)
                .unwrap();

            let color = D2D1_COLOR_F {
                r: 0.0,
                g: 0.5,
                b: 1.0,
                a: 0.45,
            };
            let base_rt: ID2D1RenderTarget = rt.cast().unwrap();
            let brush = base_rt.CreateSolidColorBrush(&color, None).unwrap();

            state.render_target = Some(rt);
            state.brush = Some(brush);
        }
    }

    fn show_ripple(&mut self, duration_sec: f64, proxy: EventLoopProxy<AppEvent>, is_replay: bool) {
        #[cfg(target_os = "windows")]
        {
            let state_clone = self.state.clone();
            tokio::spawn(async move {
                {
                    let mut state = state_clone.lock().unwrap();
                    state.ripple_active = true;
                    state.ripple_radius = 0.0;
                    state.is_rest = false;
                }

                let steps = 60;
                let step_duration = duration_sec / (steps as f64);

                for i in 0..=steps {
                    tokio::time::sleep(std::time::Duration::from_secs_f64(step_duration)).await;
                    let mut state = state_clone.lock().unwrap();
                    if !state.ripple_active {
                        break;
                    }

                    let progress = (i as f32) / (steps as f32);
                    let ease_out = 1.0 - (1.0 - progress) * (1.0 - progress);
                    state.ripple_radius = state.ripple_max_radius * ease_out;

                    unsafe {
                        let _ = SetLayeredWindowAttributes(
                            state.hwnd,
                            COLORREF(0),
                            255,
                            LWA_COLORKEY | LWA_ALPHA,
                        );

                        if let (Some(rt), Some(brush)) = (&state.render_target, &state.brush) {
                            rt.BeginDraw();
                            clear_target(
                                rt,
                                &D2D1_COLOR_F {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.0,
                                },
                            );

                            let size = target_size(rt);
                            let ellipse = D2D1_ELLIPSE {
                                point: D2D_POINT_2F {
                                    x: size.width / 2.0,
                                    y: size.height / 2.0,
                                },
                                radiusX: state.ripple_radius,
                                radiusY: state.ripple_radius,
                            };
                            draw_ellipse_outline(rt, &ellipse, brush, 10.0);
                            let _ = rt.EndDraw(None, None);
                        }
                    }
                }

                {
                    let mut state = state_clone.lock().unwrap();
                    state.ripple_active = false;
                    unsafe {
                        if let Some(rt) = &state.render_target {
                            rt.BeginDraw();
                            clear_target(
                                rt,
                                &D2D1_COLOR_F {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.0,
                                },
                            );
                            let _ = rt.EndDraw(None, None);
                        }
                    }
                }

                if !is_replay {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    let _ = proxy.send_event(crate::AppEvent::BlinkReplay);
                } else {
                    let _ = proxy.send_event(crate::AppEvent::Hide);
                }
            });
        }
    }

    fn show_rest(&mut self, duration_sec: f64, proxy: EventLoopProxy<AppEvent>) {
        #[cfg(target_os = "windows")]
        {
            let state_clone = self.state.clone();
            tokio::spawn(async move {
                {
                    let mut state = state_clone.lock().unwrap();
                    state.is_rest = true;
                    state.ripple_active = false;

                    unsafe {
                        let _ = SetLayeredWindowAttributes(state.hwnd, COLORREF(0), 200, LWA_ALPHA);

                        if let Some(rt) = &state.render_target {
                            rt.BeginDraw();
                            clear_target(
                                rt,
                                &D2D1_COLOR_F {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                },
                            );
                            let _ = rt.EndDraw(None, None);
                        }
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs_f64(duration_sec)).await;
                let _ = proxy.send_event(crate::AppEvent::Hide);
            });
        }
    }

    fn hide_ripple(&mut self) {
        #[cfg(target_os = "windows")]
        {
            let mut state = self.state.lock().unwrap();
            state.ripple_active = false;
            state.is_rest = false;
            unsafe {
                let _ = SetLayeredWindowAttributes(
                    state.hwnd,
                    COLORREF(0),
                    255,
                    LWA_COLORKEY | LWA_ALPHA,
                );

                if let Some(rt) = &state.render_target {
                    rt.BeginDraw();
                    clear_target(
                        rt,
                        &D2D1_COLOR_F {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        },
                    );
                    let _ = rt.EndDraw(None, None);
                }
            }
        }
    }
}
