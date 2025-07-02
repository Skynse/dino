#[derive(Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub timestamp: f32,
}

impl Frame {
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        let pixels = vec![0; pixel_count * 4];
        Frame {
            width,
            height,
            pixels,
            timestamp: 0.0,
        }
    }
}
