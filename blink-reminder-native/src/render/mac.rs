use core_graphics::display::CGDisplay;
use foreign_types_shared::ForeignType;
use objc2::rc::Id;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{msg_send, msg_send_id, ClassType};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSColor, NSScreen, NSView, NSWindow, NSWindowStyleMask};
use objc2_core_image::CIFilter;
use objc2_foundation::{MainThreadMarker, NSArray, NSNumber, NSString};
use objc2_quartz_core::{CABasicAnimation, CALayer, CAMediaTimingFunction, CATransaction};

use super::RippleRenderer;
use crate::AppEvent;
use tao::event_loop::EventLoopProxy;

pub struct MacRenderer {
    window: Option<Id<NSWindow>>,
}

impl MacRenderer {
    pub fn new() -> Self {
        Self { window: None }
    }
}

impl RippleRenderer for MacRenderer {
    fn setup(&mut self) {
        unsafe {
            let mtm = MainThreadMarker::new_unchecked();
            let app = NSApplication::sharedApplication(mtm);
            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

            let screen = NSScreen::mainScreen(mtm).expect("No main screen");
            let frame = screen.frame();

            let window = NSWindow::initWithContentRect_styleMask_backing_defer(
                mtm.alloc(),
                frame,
                NSWindowStyleMask::Borderless,
                NSBackingStoreType::NSBackingStoreBuffered,
                false,
            );

            window.setOpaque(false);
            window.setBackgroundColor(Some(&NSColor::clearColor()));
            window.setHasShadow(false);
            window.setIgnoresMouseEvents(true);
            window.setLevel(1000); // CGWindowLevelForKey(kCGOverlayWindowLevelKey)

            // Create a content view
            let view = NSView::initWithFrame(mtm.alloc(), frame);
            view.setWantsLayer(true);

            let layer = CALayer::layer();
            view.setLayer(Some(&layer));

            window.setContentView(Some(&view));

            self.window = Some(window);
        }
    }

    fn show_ripple(&mut self, duration_sec: f64, proxy: EventLoopProxy<AppEvent>, is_replay: bool) {
        let window = self.window.as_ref().expect("Renderer not setup");
        unsafe {
            if !is_replay {
                // Capture the screen image
                let display = CGDisplay::main();
                let cg_image = display.image();

                if let Some(view) = window.contentView() {
                    if let Some(layer) = view.layer() {
                        // Set the captured screen as the layer's contents
                        if let Some(image) = cg_image {
                            let image_ptr = image.as_ptr() as *mut AnyObject;
                            let _: () = msg_send![&layer, setContents: image_ptr];
                        }
                    }
                }
            }

            if let Some(view) = window.contentView() {
                if let Some(layer) = view.layer() {
                    let filter_name = NSString::from_str("CITorusLensDistortion");
                    let filter: Option<Id<CIFilter>> =
                        msg_send_id![CIFilter::class(), filterWithName: &*filter_name];

                    if let Some(filter) = filter {
                        let width_key = NSString::from_str("inputWidth");
                        let refraction_key = NSString::from_str("inputRefraction");
                        let center_key = NSString::from_str("inputCenter");
                        let radius_key = NSString::from_str("inputRadius");

                        // Set Center
                        let center_x = window.frame().size.width / 2.0;
                        let center_y = window.frame().size.height / 2.0;
                        if let Some(ci_vector_cls) = AnyClass::get("CIVector") {
                            let center_vec: Id<AnyObject> =
                                msg_send_id![ci_vector_cls, vectorWithX: center_x as f64 Y: center_y as f64];
                            let _: () =
                                msg_send![&filter, setValue: &*center_vec, forKey: &*center_key];
                        }

                        // Set Width of the ripple
                        let width_val = NSNumber::numberWithDouble(120.0);
                        let _: () = msg_send![&filter, setValue: &*width_val, forKey: &*width_key];

                        // Set Refraction (Distortion strength)
                        let refraction_val = NSNumber::numberWithDouble(1.2);
                        let _: () =
                            msg_send![&filter, setValue: &*refraction_val, forKey: &*refraction_key];

                        // Apply filter to layer's filters (not backgroundFilters)
                        let filters = NSArray::from_slice(&[&*filter]);
                        let _: () = msg_send![&layer, setFilters: &*filters];

                        // Animate the radius
                        CATransaction::begin();
                        CATransaction::setAnimationDuration(duration_sec);

                        let anim: Id<CABasicAnimation> = msg_send_id![objc2::class!(CABasicAnimation), animationWithKeyPath: &*NSString::from_str("filters.CITorusLensDistortion.inputRadius")];

                        let from_val = NSNumber::numberWithDouble(0.0);
                        let to_val = NSNumber::numberWithDouble(
                            window.frame().size.width.max(window.frame().size.height) * 1.0,
                        );

                        let _: () = msg_send![&anim, setFromValue: &*from_val];
                        let _: () = msg_send![&anim, setToValue: &*to_val];

                        let timing_func: Id<CAMediaTimingFunction> = msg_send_id![objc2::class!(CAMediaTimingFunction), functionWithName: &*NSString::from_str("easeOut")];
                        let _: () = msg_send![&anim, setTimingFunction: &*timing_func];

                        layer.addAnimation_forKey(&anim, Some(&*NSString::from_str("ripple")));

                        // Set final value so it doesn't snap back
                        let _: () = msg_send![&filter, setValue: &*to_val, forKey: &*radius_key];

                        CATransaction::commit();
                    }
                }
            }

            window.orderFrontRegardless();

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
    }

    fn show_rest(&mut self, duration_sec: f64, proxy: EventLoopProxy<AppEvent>) {
        let window = self.window.as_ref().expect("Renderer not setup");
        unsafe {
            // Capture the screen image
            let display = CGDisplay::main();
            let cg_image = display.image();

            if let Some(view) = window.contentView() {
                if let Some(layer) = view.layer() {
                    // Set the captured screen as the layer's contents
                    if let Some(image) = cg_image {
                        let image_ptr = image.as_ptr() as *mut AnyObject;
                        let _: () = msg_send![&layer, setContents: image_ptr];
                    }

                    // Create a blur filter
                    let blur_filter_name = NSString::from_str("CIGaussianBlur");
                    let blur_filter: Option<Id<CIFilter>> =
                        msg_send_id![CIFilter::class(), filterWithName: &*blur_filter_name];

                    // Create a darken filter (using CIColorControls)
                    let darken_filter_name = NSString::from_str("CIColorControls");
                    let darken_filter: Option<Id<CIFilter>> =
                        msg_send_id![CIFilter::class(), filterWithName: &*darken_filter_name];

                    if let (Some(blur), Some(darken)) = (blur_filter, darken_filter) {
                        // Set up blur
                        let radius_key = NSString::from_str("inputRadius");
                        let blur_radius = NSNumber::numberWithDouble(20.0);
                        let _: () = msg_send![&blur, setValue: &*blur_radius, forKey: &*radius_key];

                        // Set up darken (brightness < 0)
                        let brightness_key = NSString::from_str("inputBrightness");
                        let brightness_val = NSNumber::numberWithDouble(-0.3);
                        let _: () = msg_send![&darken, setValue: &*brightness_val, forKey: &*brightness_key];

                        // Apply filters
                        let filters = NSArray::from_slice(&[&*blur, &*darken]);
                        let _: () = msg_send![&layer, setFilters: &*filters];

                        // Animate opacity to fade in
                        CATransaction::begin();
                        CATransaction::setAnimationDuration(1.0); // 1 second fade in

                        let anim: Id<CABasicAnimation> = msg_send_id![objc2::class!(CABasicAnimation), animationWithKeyPath: &*NSString::from_str("opacity")];
                        let from_val = NSNumber::numberWithDouble(0.0);
                        let to_val = NSNumber::numberWithDouble(1.0);

                        let _: () = msg_send![&anim, setFromValue: &*from_val];
                        let _: () = msg_send![&anim, setToValue: &*to_val];

                        let timing_func: Id<CAMediaTimingFunction> = msg_send_id![objc2::class!(CAMediaTimingFunction), functionWithName: &*NSString::from_str("easeInEaseOut")];
                        let _: () = msg_send![&anim, setTimingFunction: &*timing_func];

                        layer.addAnimation_forKey(&anim, Some(&*NSString::from_str("fade_in")));
                        
                        // Set final opacity
                        let _: () = msg_send![&layer, setOpacity: 1.0f32];

                        CATransaction::commit();
                    }
                }
            }

            window.orderFrontRegardless();

            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs_f64(duration_sec)).await;
                let _ = proxy.send_event(crate::AppEvent::Hide);
            });
        }
    }

    fn hide_ripple(&mut self) {
        let window = self.window.as_ref().expect("Renderer not setup");
        unsafe {
            if let Some(view) = window.contentView() {
                if let Some(layer) = view.layer() {
                    let empty_filters: Id<NSArray<CIFilter>> = NSArray::new();
                    let _: () = msg_send![&layer, setFilters: &*empty_filters];
                    let nil_ptr: *mut AnyObject = std::ptr::null_mut();
                    let _: () = msg_send![&layer, setContents: nil_ptr];
                }
            }
            window.orderOut(None);
        }
    }
}
