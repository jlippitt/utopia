use super::WIDTH;

#[derive(Copy, Clone, Debug, Default)]
pub struct Pixel {
    pub color: u16,
}

pub type PixelBuffer = [Pixel; WIDTH >> 1];
