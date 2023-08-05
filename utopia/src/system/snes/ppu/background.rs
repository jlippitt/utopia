use super::buffer::Pixel;
use super::vram::Vram;
use super::window::MASK_NONE;
use tracing::{debug, trace};

const TILE_MIRROR_32: u16 = 31;
const TILE_MIRROR_64: u16 = 63;

const TILE_SHIFT_32: u16 = 5;
const TILE_SHIFT_64: u16 = 6;

pub struct BackgroundLayer {
    tile_size: bool,
    tile_map: u16,
    tile_mirror_x: u16,
    tile_mirror_y: u16,
    tile_shift_y: u16,
    chr_map: u16,
    scroll_x: u16,
    scroll_y: u16,
    mosaic_size: Option<u16>,
    mosaic_y: u16,
    name: &'static str,
}

impl BackgroundLayer {
    pub fn new(name: &'static str) -> Self {
        Self {
            tile_size: false,
            tile_map: 0,
            tile_mirror_x: TILE_MIRROR_32,
            tile_mirror_y: TILE_MIRROR_32,
            tile_shift_y: TILE_SHIFT_32,
            chr_map: 0,
            scroll_x: 0,
            scroll_y: 0,
            mosaic_size: None,
            mosaic_y: 0,
            name,
        }
    }

    pub fn set_tile_size(&mut self, tile_size: bool) {
        self.tile_size = tile_size;
        debug!("{} Tile Size: {}", self.name, 8 << (self.tile_size as u32));
    }

    pub fn set_mosaic(&mut self, enabled: bool, size: u8) {
        self.mosaic_size = enabled.then_some(size as u16);
        debug!("{} Mosaic Size: {:?}", self.name, self.mosaic_size);
    }

    pub fn set_tile_map(&mut self, value: u8) {
        let mirror_x = (value & 0x01) == 0;
        let mirror_y = (value & 0x02) == 0;

        self.tile_mirror_x = if mirror_x {
            TILE_MIRROR_32
        } else {
            TILE_MIRROR_64
        };

        self.tile_mirror_y = if mirror_y {
            TILE_MIRROR_32
        } else {
            TILE_MIRROR_64
        };

        self.tile_shift_y = if mirror_x || mirror_y {
            TILE_SHIFT_32
        } else {
            TILE_SHIFT_64
        };

        self.tile_map = ((value & 0xfc) as u16) << 8;

        debug!("{} Tile Map: {:04X}", self.name, self.tile_map);
        debug!("{} Mirror Mask X: {}", self.name, self.tile_mirror_x as u16);
        debug!("{} Mirror Mask Y: {}", self.name, self.tile_mirror_y as u16);
        debug!("{} Name Shift Y: {}", self.name, self.tile_shift_y as u16);
    }

    pub fn set_chr_map(&mut self, value: u8) {
        self.chr_map = (value as u16) << 12;
        debug!("{} CHR Map: {:04X}", self.name, self.chr_map);
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

    pub fn reset_mosaic_counter(&mut self) {
        self.mosaic_y = 0;
    }

    fn load_tile(&self, vram: &Vram, coarse_x: u16, coarse_y: u16, tile_width: bool) -> u16 {
        let tile_y = (coarse_y >> (self.tile_size as u32)) & self.tile_mirror_y;
        let tile_x = (coarse_x >> (tile_width as u32)) & self.tile_mirror_x;

        let address = self.tile_map
            | ((tile_y & 0x20) << self.tile_shift_y)
            | ((tile_x & 0x20) << TILE_SHIFT_32)
            | ((tile_y & 0x1f) << TILE_SHIFT_32)
            | (tile_x & 0x1f);

        let value = vram.data(address as usize);
        trace!("Tile Load: {:04X} => {:04X}", address, value);
        value
    }
}

impl super::Ppu {
    pub(super) fn select_bg_offsets<const HI_RES: bool>(&mut self, bg_index: usize) {
        let bg = &mut self.bg[bg_index];
        let tile_width = HI_RES || bg.tile_size;

        let coarse_x = bg.scroll_x >> 3;
        let coarse_y = bg.scroll_y >> 3;

        for (index, offset) in self.offsets.iter_mut().enumerate() {
            offset.x = bg.load_tile(&self.vram, coarse_x + index as u16, coarse_y, tile_width);

            offset.y = bg.load_tile(
                &self.vram,
                coarse_x + index as u16,
                coarse_y + 1,
                tile_width,
            );
        }
    }

    pub(super) fn select_bg_offsets_half(&mut self, bg_index: usize) {
        let bg = &mut self.bg[bg_index];

        let coarse_x = bg.scroll_x >> 3;
        let coarse_y = bg.scroll_y >> 3;

        for (index, offset) in self.offsets.iter_mut().enumerate() {
            let value = bg.load_tile(&self.vram, coarse_x + index as u16, coarse_y, bg.tile_size);

            if (value & 0x8000) != 0 {
                offset.x = 0;
                offset.y = value;
            } else {
                offset.x = value;
                offset.y = 0;
            }
        }
    }

    pub(super) fn draw_bg<const COLOR_DEPTH: u8, const OFFSET_MASK: u16, const HI_RES: bool>(
        &mut self,
        bg_index: usize,
        priority_high: u8,
        priority_low: u8,
        palette_offset: u16,
        line: u16,
    ) {
        let enabled = self.enabled[bg_index];

        if !enabled.any() {
            return;
        }

        self.select_bg_tiles::<COLOR_DEPTH, OFFSET_MASK, HI_RES>(
            bg_index,
            priority_high,
            priority_low,
            palette_offset,
            line,
        );

        if HI_RES {
            self.draw_bg_pixels_hi_res::<COLOR_DEPTH>(bg_index);
            return;
        }

        for pixel_buffer_index in [0, 1] {
            if enabled.has(pixel_buffer_index) {
                self.draw_bg_pixels_lo_res::<COLOR_DEPTH>(bg_index, pixel_buffer_index);
            }
        }
    }

    fn select_bg_tiles<const COLOR_DEPTH: u8, const OFFSET_MASK: u16, const HI_RES: bool>(
        &mut self,
        bg_index: usize,
        priority_high: u8,
        priority_low: u8,
        palette_offset: u16,
        line: u16,
    ) {
        let bg = &mut self.bg[bg_index];

        let (tile_width, tile_count) = if HI_RES {
            (true, self.tiles.len())
        } else {
            (bg.tile_size, (self.tiles.len() / 2) + 1)
        };

        let mosaic_line = {
            let mosaic_y = bg.mosaic_y;

            bg.mosaic_y += 1;

            if bg.mosaic_y == bg.mosaic_size.unwrap_or(1) {
                bg.mosaic_y = 0;
            }

            line - mosaic_y
        };

        let mut scroll_x = bg.scroll_x;
        let mut scroll_y = bg.scroll_y;

        for (index, tile) in self.tiles[0..tile_count].iter_mut().enumerate() {
            let (coarse_y, fine_y) = {
                let pos_y = scroll_y.wrapping_add(mosaic_line);
                (pos_y >> 3, pos_y & 7)
            };

            let coarse_x = (scroll_x >> 3) + index as u16;

            let tile_data = bg.load_tile(&self.vram, coarse_x, coarse_y, tile_width);

            let mut name_y = coarse_y & (bg.tile_size as u16);

            let fine_y = if (tile_data & 0x8000) != 0 {
                name_y ^= bg.tile_size as u16;
                fine_y ^ 7
            } else {
                fine_y
            };

            let mut name_x = coarse_x & (tile_width as u16);

            tile.flip_mask = if (tile_data & 0x4000) != 0 {
                name_x ^= tile_width as u16;
                14
            } else {
                0
            };

            tile.priority = if (tile_data & 0x2000) != 0 {
                priority_high
            } else {
                priority_low
            };

            let chr_name = (tile_data + (name_y << 4) + (name_x)) & 0x03ff;

            match COLOR_DEPTH {
                0 => {
                    let chr_index = bg.chr_map.wrapping_add(chr_name << 3) | fine_y;
                    let chr_data = self.vram.chr4(chr_index as usize);
                    trace!("CHR Load: {:04X} => {:04X}", chr_index, chr_data);

                    tile.chr_data = chr_data;
                    tile.palette = palette_offset + ((tile_data & 0x1c00) >> 8);
                }
                1 => {
                    let chr_index = bg.chr_map.wrapping_add(chr_name << 4) | fine_y;
                    let chr_data = self.vram.chr16(chr_index as usize);
                    trace!("CHR Load: {:04X} => {:08X}", chr_index, chr_data);

                    tile.chr_data = chr_data;
                    tile.palette = (tile_data & 0x1c00) >> 6;
                }
                2 => {
                    let chr_index = bg.chr_map.wrapping_add(chr_name << 5) | fine_y;
                    let chr_data = self.vram.chr256(chr_index as usize);
                    trace!("CHR Load: {:04X} => {:016X}", chr_index, chr_data);

                    tile.chr_data = chr_data;
                    tile.palette = 0;
                }
                _ => unreachable!(),
            }

            if OFFSET_MASK != 0 {
                let offset = self.offsets[index >> (tile_width as u32)];

                if (offset.x & OFFSET_MASK) != 0 {
                    scroll_x = offset.x & 0x03ff;
                } else {
                    scroll_x = bg.scroll_x;
                }

                if (offset.y & OFFSET_MASK) != 0 {
                    scroll_y = offset.y & 0x03ff;
                } else {
                    scroll_y = bg.scroll_y;
                }
            }
        }

        trace!("{} Tiles: {:?}", bg.name, self.tiles);
    }

    fn draw_bg_pixels_lo_res<const COLOR_DEPTH: u8>(
        &mut self,
        bg_index: usize,
        pixel_buffer_index: usize,
    ) {
        let mask = if self.window_enabled[bg_index].has(pixel_buffer_index) {
            self.window_mask[bg_index].mask(&self.window)
        } else {
            &MASK_NONE
        };

        let bg = &self.bg[bg_index];
        let pixels = &mut self.pixels[pixel_buffer_index];

        let mosaic_size = bg.mosaic_size.unwrap_or(1);

        let mut tiles = self.tiles.into_iter();
        let mut tile = tiles.next().unwrap();
        let mut mosaic_x = 1;
        let mut color = 0;
        let mut shift = (bg.scroll_x & 7) << 1;

        for (index, pixel) in pixels.iter_mut().enumerate() {
            mosaic_x -= 1;

            if mosaic_x == 0 {
                mosaic_x = mosaic_size;
                color = pixel_color::<COLOR_DEPTH>(tile.chr_data >> (shift ^ tile.flip_mask));
            }

            if color != 0 && !mask[index] && tile.priority > pixel.priority {
                *pixel = Pixel {
                    color: self.cgram.color((tile.palette as usize) | color),
                    priority: tile.priority,
                    layer: 1 << bg_index,
                };
            }

            shift += 2;

            if shift == 16 {
                tile = tiles.next().unwrap();
                shift = 0;
            }
        }
    }

    fn draw_bg_pixels_hi_res<const COLOR_DEPTH: u8>(&mut self, bg_index: usize) {
        let window_enabled = self.window_enabled[bg_index];
        let window_mask = self.window_mask[bg_index].mask(&self.window);

        let mask = [
            if window_enabled.has(0) {
                window_mask
            } else {
                &MASK_NONE
            },
            if window_enabled.has(1) {
                window_mask
            } else {
                &MASK_NONE
            },
        ];

        let bg = &self.bg[bg_index];

        let mosaic_size = bg.mosaic_size.map_or(1, |size| size << 1);

        let mut tiles = self.tiles.into_iter();
        let mut tile = tiles.next().unwrap();
        let mut mosaic_x = 1;
        let mut color = 0;
        let mut shift = (bg.scroll_x & 7) << 1;

        for index in 0..(self.pixels[0].len() << 1) {
            mosaic_x -= 1;

            if mosaic_x == 0 {
                mosaic_x = mosaic_size;
                color = pixel_color::<COLOR_DEPTH>(tile.chr_data >> (shift ^ tile.flip_mask));
            }

            let pixel_buffer_index = index & 1;
            let screen_x = index >> 1;
            let pixel = &mut self.pixels[pixel_buffer_index][screen_x];

            if color != 0 && !mask[pixel_buffer_index][screen_x] && tile.priority > pixel.priority {
                *pixel = Pixel {
                    color: self.cgram.color((tile.palette as usize) | color),
                    priority: tile.priority,
                    layer: 1 << bg_index,
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

fn pixel_color<const COLOR_DEPTH: u8>(chr: u64) -> usize {
    let mut color = chr & 0x03;

    if COLOR_DEPTH > 0 {
        color |= (chr >> 14) & 0x0c;

        if COLOR_DEPTH > 1 {
            color |= (chr >> 28) & 0x30;
            color |= (chr >> 42) & 0xc0;
        }
    }

    color as usize
}
