use super::RippleRenderer;
use crate::AppEvent;
use tao::event_loop::EventLoopProxy;

pub struct WinRenderer {
}

impl WinRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl RippleRenderer for WinRenderer {
    fn setup(&mut self) {
        // TODO: Create WS_EX_LAYERED | WS_EX_TRANSPARENT window
        // Initialize Direct2D / DirectComposition
        println!("Windows setup called");
    }

    fn show_ripple(&mut self, duration_sec: f64, proxy: EventLoopProxy<AppEvent>, is_replay: bool) {
        // TODO: Implement Windows ripple animation
        println!("Windows show_ripple called");
        
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs_f64(duration_sec)).await;
            if !is_replay {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let _ = proxy.send_event(crate::AppEvent::BlinkReplay);
            } else {
                let _ = proxy.send_event(crate::AppEvent::Hide);
            }
        });
    }

    fn show_rest(&mut self, duration_sec: f64, proxy: EventLoopProxy<AppEvent>) {
        // TODO: Implement Windows rest animation
        println!("Windows show_rest called");
        
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs_f64(duration_sec)).await;
            let _ = proxy.send_event(crate::AppEvent::Hide);
        });
    }

    fn hide_ripple(&mut self) {
        // TODO: Hide Windows window
        println!("Windows hide_ripple called");
    }
}
