use core_graphics::display::CGDisplay;
use objc2::rc::Id;
use objc2::runtime::AnyObject;
use objc2_quartz_core::CALayer;
use objc2::{msg_send, msg_send_id, ClassType};
use foreign_types_shared::ForeignType;

pub fn test() {
    let display = CGDisplay::main();
    if let Some(image) = display.image() {
        unsafe {
            let layer = CALayer::layer();
            let image_ptr = image.as_ptr() as *mut AnyObject;
            let _: () = msg_send![&layer, setContents: image_ptr];
            println!("Set contents successfully");
        }
    }
}
