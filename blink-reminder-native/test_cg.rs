use core_graphics::display::CGDisplay;
fn main() {
    let display = CGDisplay::main();
    if let Some(image) = display.image() {
        println!("Image width: {}", image.width());
    } else {
        println!("Failed to get image");
    }
}
