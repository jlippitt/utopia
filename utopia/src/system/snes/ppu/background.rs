use super::buffer::{Pixel, Tile};
use super::toggle::ScreenToggle;
use tracing::{debug, trace};

const MIRROR_MASK_32: u16 = 31;
const MIRROR_MASK_64: u16 = 63;

const NAME_SHIFT_32: u16 = 5;
const NAME_SHIFT_64: u16 = 6;

fn color<const COLOR_DEPTH: u8>(chr: u64) -> usize {
    let mut color = chr & 0x03;

    if COLOR_DEPTH > 1 {
        color |= (chr >> 14) & 0x0c;
    }

    color as usize
}

pub struct BackgroundLayer {
    tile_map: u16,
    mirror_mask_x: u16,
    mirror_mask_y: u16,
    name_shift_y: u16,
    chr_map: u16,
    scroll_x: u16,
    scroll_y: u16,
    name: &'static str,
}

impl BackgroundLayer {
    pub fn new(name: &'static str) -> Self {
        Self {
            tile_map: 0,
            mirror_mask_x: MIRROR_MASK_32,
            mirror_mask_y: MIRROR_MASK_32,
            name_shift_y: NAME_SHIFT_32,
            chr_map: 0,
            scroll_x: 0,
            scroll_y: 0,
            name,
        }
    }

    pub fn set_tile_map(&mut self, value: u8) {
        let mirror_x = (value & 0x01) == 0;
        let mirror_y = (value & 0x02) == 0;

        self.mirror_mask_x = if mirror_x {
            MIRROR_MASK_32
        } else {
            MIRROR_MASK_64
        };

        self.mirror_mask_y = if mirror_y {
            MIRROR_MASK_32
        } else {
            MIRROR_MASK_64
        };

        self.name_shift_y = if mirror_x || mirror_y {
            NAME_SHIFT_32
        } else {
            NAME_SHIFT_64
        };

        self.tile_map = ((value & 0xfc) as u16) << 8;

        debug!("{} Tile Map: {:04X}", self.name, self.tile_map);
        debug!("{} Mirror Mask X: {}", self.name, self.mirror_mask_x as u16);
        debug!("{} Mirror Mask Y: {}", self.name, self.mirror_mask_y as u16);
        debug!("{} Name Shift Y: {}", self.name, self.name_shift_y as u16);
    }

    pub fn set_chr_map(&mut self, value: u8) {
        let chr_address = (value as u16) << 12;
        debug!("{} CHR Map: {:04X}", self.name, chr_address);
        self.chr_map = chr_address >> 3;
    }

    pub fn set_scroll_x(&mut self, regs: &mut (u8, u8), value: u8) {
        self.scroll_x =
            (((value & 0x03) as u16) << 8) | ((regs.0 & 0xf8) as u16) | ((regs.1 & 0x07) as u16);

        regs.0 = value;
        regs.1 = value;

        debug!("{} Scroll X: {}", self.name, self.scroll_x);
    }

    pub fn set_scroll_y(&mut self, regs: &mut (u8, u8), value: u8) {
        self.scroll_y = (((value & 0x03) as u16) << 8) | (regs.0 as u16);

        regs.0 = value;

        debug!("{} Scroll Y: {}", self.name, self.scroll_y);
    }
}

impl super::Ppu {
    pub(super) fn draw_bg<const COLOR_DEPTH: u8>(
        &mut self,
        bg_index: usize,
        priority_high: u8,
        priority_low: u8,
        line: u16,
    ) {
        let enabled = self.enabled[bg_index];

        if !enabled.any_enabled() {
            return;
        }

        self.select_tiles::<COLOR_DEPTH>(bg_index, priority_high, priority_low, line);

        if enabled.screen_enabled(ScreenToggle::Main) {
            self.draw_lo_res::<COLOR_DEPTH>(bg_index, 0);
        }

        if enabled.screen_enabled(ScreenToggle::Sub) {
            self.draw_lo_res::<COLOR_DEPTH>(bg_index, 1);
        }
    }

    fn select_tiles<const COLOR_DEPTH: u8>(
        &mut self,
        bg_index: usize,
        priority_high: u8,
        priority_low: u8,
        line: u16,
    ) {
        let bg = &self.bg[bg_index];

        let (coarse_y, mut fine_y) = {
            let pos_y = bg.scroll_y.wrapping_add(line);
            ((pos_y >> 3) & bg.mirror_mask_y, pos_y & 7)
        };

        let mut coarse_x = bg.scroll_x >> 3;

        for tile in &mut self.tiles {
            let tile_data = {
                let address = bg.tile_map
                    | ((coarse_y & 0x20) << bg.name_shift_y)
                    | ((coarse_x & 0x20) << NAME_SHIFT_32)
                    | ((coarse_y & 0x1f) << NAME_SHIFT_32)
                    | (coarse_x & 0x1f);

                let value = self.vram.data(address);
                trace!("Tile Load: {:04X} => {:04X}", address, value);
                value
            };

            if (tile_data & 0x8000) != 0 {
                fine_y ^= 7;
            }

            let flip_mask = if (tile_data & 0x4000) != 0 { 14 } else { 0 };

            let priority = if (tile_data & 0x2000) != 0 {
                priority_high
            } else {
                priority_low
            };

            let chr_name = tile_data & 0x03ff;

            match COLOR_DEPTH {
                0 => {
                    let chr_index = (fine_y << 12) | ((bg.chr_map + chr_name) & 0x0fff);
                    let chr_data = self.vram.chr4(chr_index as usize);
                    trace!("CHR Load: {:04X} => {:04X}", chr_index, chr_data);

                    *tile = Tile {
                        chr_data,
                        flip_mask,
                        priority,
                        palette: ((tile_data & 0x1c00) >> 8),
                    };
                }
                1 => {
                    let chr_index = (fine_y << 12) | ((bg.chr_map + (chr_name << 1)) & 0x0fff);
                    let chr_data = self.vram.chr16(chr_index as usize);
                    trace!("CHR Load: {:04X} => {:04X}", chr_index, chr_data);

                    *tile = Tile {
                        chr_data,
                        flip_mask,
                        priority,
                        palette: ((tile_data & 0x1c00) >> 6),
                    };
                }
                2 => todo!("256 color backgrounds"),
                _ => unreachable!(),
            }

            coarse_x = coarse_x.wrapping_add(1) & bg.mirror_mask_x;
        }

        trace!("{} Tiles: {:?}", bg.name, self.tiles);
    }

    fn draw_lo_res<const COLOR_DEPTH: u8>(&mut self, bg_index: usize, pixels_index: usize) {
        let bg = &self.bg[bg_index];
        let pixels = &mut self.pixels[pixels_index];

        let mut shift = (bg.scroll_x & 7) << 1;
        let mut tiles = self.tiles.into_iter();
        let mut tile = tiles.next().unwrap();

        for pixel in pixels {
            let color = color::<COLOR_DEPTH>(tile.chr_data >> (shift ^ tile.flip_mask));

            if color != 0 && tile.priority > pixel.priority {
                *pixel = Pixel {
                    color: self.cgram.color((tile.palette as usize) | color),
                    priority: tile.priority,
                };
            }

            shift += 2;

            if shift == 16 {
                tile = tiles.next().unwrap();
                shift = 0;
            }
        }
    }
}
