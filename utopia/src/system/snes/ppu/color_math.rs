use super::buffer::LAYER_BACKDROP;
use super::window::{BoolMask, MASK_NONE};
use tracing::debug;

const WINDOW_INDEX: usize = 5;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum WindowOperator {
    Never,
    Outside,
    Inside,
    Always,
}

struct ClipMask<'a> {
    mask: &'a BoolMask,
    invert: bool,
}

pub struct ColorMath {
    prevent: WindowOperator,
    clip: WindowOperator,
    lhs_mask: u8,
    rhs_mask: u8,
    fixed_color: u16,
    subtract: bool,
    half: bool,
}

impl ColorMath {
    pub fn new() -> Self {
        Self {
            prevent: WindowOperator::Never,
            clip: WindowOperator::Never,
            lhs_mask: 0,
            rhs_mask: 0,
            fixed_color: 0,
            subtract: false,
            half: false,
        }
    }

    pub fn set_control(&mut self, value: u8) {
        self.clip = match (value >> 6) & 0x03 {
            0x00 => WindowOperator::Never,
            0x01 => WindowOperator::Outside,
            0x02 => WindowOperator::Inside,
            0x03 => WindowOperator::Always,
            _ => unreachable!(),
        };

        self.prevent = match (value >> 4) & 0x03 {
            0x00 => WindowOperator::Never,
            0x01 => WindowOperator::Outside,
            0x02 => WindowOperator::Inside,
            0x03 => WindowOperator::Always,
            _ => unreachable!(),
        };

        self.rhs_mask = if (value & 0x02) != 0 {
            !LAYER_BACKDROP
        } else {
            0
        };

        debug!("Color Math Clip: {:?}", self.clip);
        debug!("Color Math Prevent: {:?}", self.prevent);
        debug!("Color Math RHS Mask: {:08b}", self.rhs_mask);
    }

    pub fn set_operator(&mut self, value: u8) {
        self.subtract = (value & 0x80) != 0;
        self.half = (value & 0x40) != 0;
        self.lhs_mask = value & 0x3f;
        debug!("Color Math Subtract: {}", self.subtract);
        debug!("Color Math Half: {}", self.half);
        debug!("Color Math LHS Mask: {:08b}", self.lhs_mask);
    }

    pub fn set_fixed_color(&mut self, value: u8) {
        let intensity = value as u16 & 0x1f;

        if (value & 0x20) != 0 {
            self.fixed_color = (self.fixed_color & !0x1f) | intensity;
        }

        if (value & 0x40) != 0 {
            self.fixed_color = (self.fixed_color & !(0x1f << 5)) | (intensity << 5);
        }

        if (value & 0x80) != 0 {
            self.fixed_color = (self.fixed_color & !(0x1f << 10)) | (intensity << 10);
        }

        debug!("Color Math Fixed Color: {:04X}", self.fixed_color);
    }
}

impl WindowOperator {
    fn apply(self, mask: &BoolMask) -> ClipMask<'_> {
        match self {
            WindowOperator::Never => ClipMask {
                mask: &MASK_NONE,
                invert: false,
            },
            WindowOperator::Outside => ClipMask { mask, invert: true },
            WindowOperator::Inside => ClipMask {
                mask,
                invert: false,
            },
            WindowOperator::Always => ClipMask {
                mask: &MASK_NONE,
                invert: true,
            },
        }
    }
}

impl<'a> ClipMask<'a> {
    fn apply(&self, index: usize) -> bool {
        self.mask[index] ^ self.invert
    }
}

impl super::Ppu {
    pub(super) fn apply_color_math(&mut self) {
        let mask = self.window_mask[WINDOW_INDEX].mask(&self.window);
        let prevent = self.color_math.prevent.apply(mask);
        let clip = self.color_math.clip.apply(mask);

        let [main_screen, sub_screen] = &mut self.pixels;

        for (index, lhs_pixel) in main_screen.iter_mut().enumerate() {
            if (lhs_pixel.layer & self.color_math.lhs_mask) == 0 || prevent.apply(index) {
                continue;
            }

            let rhs_pixel = &sub_screen[index];

            let (rhs, fixed) = if (rhs_pixel.layer & self.color_math.rhs_mask) == 0 {
                (self.color_math.fixed_color, true)
            } else {
                (rhs_pixel.color, false)
            };

            if clip.apply(index) {
                lhs_pixel.color = if self.color_math.subtract { 0 } else { rhs };
                continue;
            }

            let lhs = lhs_pixel.color;

            let (mut red, mut green, mut blue) = if self.color_math.subtract {
                let red = (lhs & 31).saturating_sub(rhs & 31);
                let green = ((lhs >> 5) & 31).saturating_sub((rhs >> 5) & 31);
                let blue = ((lhs >> 10) & 31).saturating_sub((rhs >> 10) & 31);
                (red, green, blue)
            } else {
                let red = (lhs & 31) + (rhs & 31);
                let green = ((lhs >> 5) & 31) + ((rhs >> 5) & 31);
                let blue = ((lhs >> 10) & 31) + ((rhs >> 10) & 31);
                (red, green, blue)
            };

            if !fixed && self.color_math.half {
                red >>= 1;
                green >>= 1;
                blue >>= 1;
            }

            lhs_pixel.color = (red.min(31)) | (green.min(31) << 5) | (blue.min(31) << 10);
        }
    }
}
