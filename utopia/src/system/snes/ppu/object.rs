use super::buffer::Tile;
use super::oam::TOTAL_SPRITES;
use super::window::MASK_NONE;
use bitflags::bitflags;
use tracing::trace;

const MAX_SPRITES_PER_LINE: usize = 32;
const MAX_TILES_PER_LINE: usize = 34;

const LAYER_INDEX: usize = 4;

const LAYER_OBJ_COLOR_MATH: u8 = 0x10;
const LAYER_OBJ_NO_COLOR_MATH: u8 = 0x40;

const SIZE_X: [[u16; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];

const SIZE_Y: [[u16; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [32, 64],
    [32, 32],
];

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct Flags: u8 {
        const TIME_OVER = 0x80;
        const RANGE_OVER = 0x40;
    }
}

pub struct ObjectLayer {
    name_base: u16,
    name_offset: u16,
    size_x: [u16; 2],
    size_y: [u16; 2],
    interlace: bool,
    selected_sprites: [usize; MAX_SPRITES_PER_LINE],
    flags: Flags,
}

impl ObjectLayer {
    pub fn new() -> Self {
        Self {
            name_base: 0,
            name_offset: 0,
            size_x: SIZE_X[0],
            size_y: SIZE_Y[0],
            interlace: false,
            selected_sprites: [0; MAX_SPRITES_PER_LINE],
            flags: Flags::empty(),
        }
    }

    pub fn flags(&self) -> u8 {
        self.flags.bits()
    }

    pub fn clear_flags(&mut self) {
        self.flags = Flags::empty();
        trace!("OBJ Flags Cleared");
    }

    pub fn set_control(&mut self, value: u8) {
        self.name_base = (value as u16 & 0x07) << 13;
        self.name_offset = (((value as u16 & 0x18) >> 3) + 1) << 12;

        let size_index = (value >> 5) as usize;
        self.size_x = SIZE_X[size_index];
        self.size_y = SIZE_Y[size_index];

        trace!("OBJ Name Base: {:04X}", self.name_base);
        trace!("OBJ CHR Offset: {:04X}", self.name_offset);
        trace!(
            "OBJ Size: {}x{}, {}x{}",
            self.size_x[0],
            self.size_y[0],
            self.size_x[1],
            self.size_y[1]
        )
    }

    pub fn set_interlace(&mut self, interlace: bool) {
        self.interlace = interlace;
        trace!("OBJ Interlace: {}", interlace);
    }
}

impl super::Ppu {
    pub(super) fn draw_obj(&mut self, line: u16, odd_frame: bool) {
        let enabled = self.enabled[LAYER_INDEX];

        if !enabled.any() {
            return;
        }

        let sprite_start_index = self.select_sprites(line);

        let tile_start_index = self.select_obj_tiles(line, odd_frame, sprite_start_index);

        for pixel_buffer_index in [0, 1] {
            if enabled.has(pixel_buffer_index) {
                self.draw_obj_pixels(tile_start_index, pixel_buffer_index);
            }
        }
    }

    fn select_sprites(&mut self, line: u16) -> usize {
        let sprite_select_offset = self.oam.sprite_select_offset();
        let mut write_index = self.obj.selected_sprites.len();

        for index in 0..TOTAL_SPRITES {
            let sprite_index = sprite_select_offset + index;
            let sprite = self.oam.sprite(sprite_index);

            let start_y = sprite.y;
            let size_y = self.obj.size_y[sprite.size as usize] >> self.obj.interlace as u32;
            let end_y = start_y + size_y;

            // Determine if sprite is on the current line
            if (start_y > line || end_y <= line) && end_y <= (line + 256) {
                continue;
            }

            let size_x = self.obj.size_x[sprite.size as usize];

            // Determine if sprite is on the screen (horizontally)
            // Sprites at X=256 count as X=0 for this step
            if sprite.x > 256 && sprite.x < (512 - size_x) {
                continue;
            }

            if write_index == 0 {
                self.obj.flags.insert(Flags::RANGE_OVER);
                trace!("Line {}: Range Over", line);
                break;
            }

            write_index -= 1;
            self.obj.selected_sprites[write_index] = sprite_index;
        }

        let count = self.obj.selected_sprites.len() - write_index;

        if count > 0 {
            trace!("Line {}: {} Sprites Selected", line, count);
        }

        write_index
    }

    fn select_obj_tiles(&mut self, line: u16, odd_frame: bool, sprite_start_index: usize) -> usize {
        let mut write_index = MAX_TILES_PER_LINE;

        'outer: for sprite_index in &self.obj.selected_sprites[sprite_start_index..] {
            let sprite = self.oam.sprite(*sprite_index);

            let flipped_y = {
                let pixel_y = line.wrapping_sub(sprite.y) & 255;

                let interlaced_y = if self.obj.interlace {
                    (pixel_y << 1) + (odd_frame as u16)
                } else {
                    pixel_y
                };

                if sprite.flip_y {
                    let size_y = self.obj.size_y[sprite.size as usize] - 1;
                    interlaced_y ^ size_y
                } else {
                    interlaced_y
                }
            };

            let size_x = self.obj.size_x[sprite.size as usize];
            let flip_mask_x = if sprite.flip_x { size_x - 1 } else { 0 };

            // Sprites at X=256 count as X=0 when testing whether tiles are on screen
            let false_start_x = if sprite.x != 256 { sprite.x } else { 0 };

            let chr_map = {
                let name_offset = if sprite.table {
                    self.obj.name_offset
                } else {
                    0
                };

                self.obj.name_base + name_offset
            };

            for pixel_x in (0..size_x).step_by(8) {
                let flipped_x = pixel_x ^ flip_mask_x;
                let false_pos_x = false_start_x + flipped_x;

                if false_pos_x >= 256 && false_pos_x < 504 {
                    continue;
                }

                if write_index == 0 {
                    self.obj.flags.insert(Flags::TIME_OVER);
                    trace!("Line {}: Time Over", line);
                    break 'outer;
                }

                let chr_x = ((flipped_x >> 3) + (sprite.name & 0x0f)) & 0x0f;
                let chr_y = ((flipped_y >> 3) + (sprite.name >> 4)) & 0x0f;
                let chr_index = chr_map + (chr_y << 8) + (chr_x << 4) + (flipped_y & 7);

                let chr_data = self.vram.chr16(chr_index as usize);
                trace!("CHR Load: {:04X} => {:04X}", chr_index, chr_data);

                write_index -= 1;

                self.tiles[write_index] = Tile {
                    chr_data,
                    flip_mask: if sprite.flip_x { 14 } else { 0 },
                    priority: sprite.priority,
                    palette: sprite.palette,
                    pos_x: sprite.x + pixel_x,
                };
            }
        }

        let count = MAX_TILES_PER_LINE - write_index;

        if count > 0 {
            trace!("Line {}: {} Tiles Selected", line, count);
        }

        write_index
    }

    fn draw_obj_pixels(&mut self, tile_start_index: usize, pixel_buffer_index: usize) {
        let mask = if self.window_enabled[LAYER_INDEX].has(pixel_buffer_index) {
            self.window_mask[LAYER_INDEX].mask(&self.window)
        } else {
            &MASK_NONE
        };

        let pixels = &mut self.pixels[pixel_buffer_index];

        for tile in &self.tiles[tile_start_index..MAX_TILES_PER_LINE] {
            for pixel_x in 0..8 {
                let pos_x = ((tile.pos_x + pixel_x) & 511) as usize;

                if pos_x >= 256 || mask[pos_x] {
                    continue;
                }

                let chr = tile.chr_data >> ((pixel_x << 1) ^ tile.flip_mask);
                let color = (chr & 0x03) | ((chr >> 14) & 0x0c);

                if color == 0 {
                    continue;
                }

                let pixel = &mut pixels[pos_x];

                if tile.priority >= pixel.priority {
                    pixel.color = self.cgram.color(tile.palette as usize + color as usize);

                    pixel.layer = if tile.palette >= 192 {
                        LAYER_OBJ_COLOR_MATH
                    } else {
                        LAYER_OBJ_NO_COLOR_MATH
                    };
                }

                // Nothing can cover a sprite pixel, even if it's hidden beneath the background
                pixel.priority = 5;
            }
        }
    }
}
