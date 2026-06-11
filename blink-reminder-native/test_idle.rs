#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceSecondsSinceLastEventType(sourceState: u32, eventType: u32) -> f64;
}

fn main() {
    let idle = unsafe { CGEventSourceSecondsSinceLastEventType(1, 0xFFFFFFFF) };
    println!("Idle time: {}", idle);
}
