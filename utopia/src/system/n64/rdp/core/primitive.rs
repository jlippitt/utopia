pub type Position = [f32; 3];
pub type Color = [f32; 4];

#[derive(Clone, Debug, Default)]
pub struct Rectangle {
    pub xh: f32,
    pub yh: f32,
    pub xl: f32,
    pub yl: f32,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TextureLayout {
    #[default]
    Rgba = 0,
    Yuv,
    ColorIndex,
    IntensityAlpha,
    Intensity,
}

impl TextureLayout {
    pub const fn into_bits(self) -> u64 {
        self as u64
    }

    pub const fn from_bits(value: u64) -> Self {
        match value & 7 {
            0 => Self::Rgba,
            1 => Self::Yuv,
            2 => Self::ColorIndex,
            3 => Self::IntensityAlpha,
            4 => Self::Intensity,
            _ => unreachable!(),
        }
    }
}
