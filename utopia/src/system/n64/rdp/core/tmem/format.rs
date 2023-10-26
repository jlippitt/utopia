use super::super::primitive::TextureLayout;
use crate::system::n64::video::decode_rgba16;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TextureFormat(TextureLayout, u32);

impl TextureFormat {
    pub fn bits_per_pixel(&self) -> usize {
        4 << self.1
    }

    pub fn decode<'a>(&self, buffer: &'a mut Vec<u8>, input: &'a [u8]) -> &'a [u8] {
        match self {
            Self(TextureLayout::Rgba, 3) => input,
            Self(TextureLayout::Rgba, 2) => {
                *buffer = input.chunks_exact(2).flat_map(decode_rgba16).collect();
                buffer
            }
            Self(TextureLayout::IntensityAlpha, 2) => {
                *buffer = input
                    .chunks_exact(2)
                    .flat_map(|chunks| [chunks[0], chunks[0], chunks[0], chunks[1]])
                    .collect();
                buffer
            }
            Self(TextureLayout::IntensityAlpha, 1) => {
                *buffer = input
                    .iter()
                    .flat_map(|byte| {
                        let intensity = byte & 0xf0;
                        let alpha = byte << 4;
                        [intensity, intensity, intensity, alpha]
                    })
                    .collect();
                buffer
            }
            Self(TextureLayout::IntensityAlpha, 0) => {
                *buffer = input
                    .iter()
                    .flat_map(|&byte| {
                        let high_i = byte & 0xe0;
                        let high_a = ((byte >> 4) & 1) * 255;
                        let low_i = (byte & 0x0e) << 4;
                        let low_a = (byte & 1) * 255;
                        [high_i, high_i, high_i, high_a, low_i, low_i, low_i, low_a]
                    })
                    .collect();
                buffer
            }
            Self(TextureLayout::Intensity, 1) => {
                *buffer = input.iter().flat_map(|&byte| [byte; 4]).collect();
                buffer
            }
            Self(TextureLayout::Intensity, 0) => {
                *buffer = input
                    .iter()
                    .flat_map(|&byte| {
                        let high = byte & 0xf0;
                        let low = byte << 4;
                        [high, high, high, high, low, low, low, low]
                    })
                    .collect();
                buffer
            }
            _ => unimplemented!("Texture decoding for {}", self),
        }
    }
}

impl Default for TextureFormat {
    fn default() -> Self {
        Self(TextureLayout::Rgba, 3)
    }
}

impl From<(TextureLayout, u32)> for TextureFormat {
    fn from((layout, size): (TextureLayout, u32)) -> Self {
        Self(layout, size)
    }
}

impl fmt::Display for TextureFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            match self.0 {
                TextureLayout::Rgba => "RGBA",
                TextureLayout::Yuv => "YUV",
                TextureLayout::ColorIndex => "ClrIndex",
                TextureLayout::IntensityAlpha => "IA",
                TextureLayout::Intensity => "I",
            },
            self.bits_per_pixel(),
        )
    }
}
