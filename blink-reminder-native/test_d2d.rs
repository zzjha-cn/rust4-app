use windows::Win32::Graphics::Direct2D::*;
fn main() {
    unsafe {
        let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap();
    }
}
