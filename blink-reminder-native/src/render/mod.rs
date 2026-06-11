use tao::event_loop::EventLoopProxy;
use crate::AppEvent;

pub trait RippleRenderer {
    fn setup(&mut self);
    fn show_ripple(&mut self, duration_sec: f64, proxy: EventLoopProxy<AppEvent>, is_replay: bool);
    fn hide_ripple(&mut self);
}

#[cfg(target_os = "macos")]
pub mod mac;

#[cfg(target_os = "windows")]
pub mod win;

pub fn create_renderer() -> Box<dyn RippleRenderer> {
    #[cfg(target_os = "macos")]
    {
        Box::new(mac::MacRenderer::new())
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(win::WinRenderer::new())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        unimplemented!("Unsupported OS")
    }
}
