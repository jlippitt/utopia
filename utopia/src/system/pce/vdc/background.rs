use super::super::vce::Color;
use super::vram::Vram;
use tracing::debug;

struct Tile {
    chr_low: u16,
    chr_high: u16,
    palette_offset: usize,
}

pub struct BackgroundLayer {
    enabled: bool,
    scroll_x: u16,
    scroll_y: u16,
    tile_mirror_x: u16,
    tile_mirror_y: u16,
    tile_shift_y: u16,
}

impl BackgroundLayer {
    pub fn new() -> Self {
        Self {
            enabled: false,
            scroll_x: 0,
            scroll_y: 0,
            tile_mirror_x: 31,
            tile_mirror_y: 31,
            tile_shift_y: 5,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        debug!("BG Layer Enabled: {}", enabled);
    }

    pub fn set_scroll_x(&mut self, msb: bool, value: u8) {
        self.scroll_x = if msb {
            (self.scroll_x & 0xff) | ((value as u16 & 0x03) << 8)
        } else {
            (self.scroll_x & 0xff00) | value as u16
        };
        debug!("BG Scroll X: {}", self.scroll_x);
    }

    pub fn set_scroll_y(&mut self, msb: bool, value: u8) {
        self.scroll_y = if msb {
            (self.scroll_y & 0xff) | ((value as u16 & 0x03) << 8)
        } else {
            (self.scroll_y & 0xff00) | value as u16
        };
        debug!("BG Scroll Y: {}", self.scroll_x);
    }

    pub fn set_tile_map_size(&mut self, value: u8) {
        (self.tile_mirror_x, self.tile_shift_y) = match (value >> 4) & 3 {
            0 => (31, 5),
            1 => (63, 6),
            _ => (127, 7),
        };

        self.tile_mirror_y = if (value & 0x40) != 0 { 63 } else { 31 };

        debug!("BG Tile Mirror X: {}", self.tile_mirror_x);
        debug!("BG Tile Mirror Y: {}", self.tile_mirror_y);
        debug!("BG Tile Shift Y: {}", self.tile_shift_y);
    }

    pub fn render_line(
        &self,
        line_buffer: &mut [Color],
        vram: &Vram,
        palette: &[Color],
        line: u16,
    ) {
        if !self.enabled {
            return;
        }

        let pos_y = self.scroll_y + line;

        let mut coarse_x = self.scroll_x >> 3;
        let mut fine_x = self.scroll_x & 7;

        let mut tile = self.next_tile(vram, pos_y, coarse_x);

        for output_pixel in line_buffer {
            let color_low = ((tile.chr_low >> 7) & 1) | ((tile.chr_low >> 14) & 2);
            let color_high = ((tile.chr_high >> 7) & 1) | ((tile.chr_high >> 14) & 2);
            let color_index = (color_high << 2) | color_low;

            if color_index != 0 {
                *output_pixel = palette[tile.palette_offset + color_index as usize];
            }

            if fine_x == 7 {
                fine_x = 0;
                coarse_x = (coarse_x + 1) & self.tile_mirror_x;
                tile = self.next_tile(vram, pos_y, coarse_x);
            } else {
                fine_x += 1;
                tile.chr_low <<= 1;
                tile.chr_high <<= 1;
            }
        }
    }

    fn next_tile(&self, vram: &Vram, pos_y: u16, coarse_x: u16) -> Tile {
        let coarse_y = (pos_y >> 3) & self.tile_mirror_y;
        let tile_address = (coarse_y << self.tile_shift_y) + coarse_x;
        let tile = vram.get(tile_address as usize);
        let chr_address = ((tile & 0x0fff) << 4) + (pos_y & 7);

        Tile {
            chr_low: vram.get(chr_address as usize),
            chr_high: vram.get(chr_address as usize + 8),
            palette_offset: tile as usize >> 8,
        }
    }
}
