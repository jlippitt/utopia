use super::buffer::Tile;
use super::oam::TOTAL_SPRITES;
use tracing::{debug, trace};

const MAX_SPRITES_PER_LINE: usize = 32;
const MAX_TILES_PER_LINE: usize = 34;

pub const SIZE_X: [[u16; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];

pub const SIZE_Y: [[u16; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [32, 64],
    [32, 32],
];

pub struct ObjectLayer {
    name_base: u16,
    name_offset: u16,
    size_x: [u16; 2],
    size_y: [u16; 2],
    selected_sprites: [usize; MAX_SPRITES_PER_LINE],
}

impl ObjectLayer {
    pub fn new() -> Self {
        Self {
            name_base: 0,
            name_offset: 0,
            size_x: SIZE_X[0],
            size_y: SIZE_Y[0],
            selected_sprites: [0; MAX_SPRITES_PER_LINE],
        }
    }

    pub fn set_control(&mut self, value: u8) {
        self.name_base = (value as u16 & 0x07) << 13;
        self.name_offset = (((value as u16 & 0x18) >> 3) + 1) << 12;

        let size_index = (value >> 5) as usize;
        self.size_x = SIZE_X[size_index];
        self.size_y = SIZE_Y[size_index];

        debug!("OBJ Name Base: {:04X}", self.name_base);
        debug!("OBJ CHR Offset: {:04X}", self.name_offset);
        debug!(
            "OBJ Size: {}x{}, {}x{}",
            self.size_x[0], self.size_y[0], self.size_x[1], self.size_y[1]
        )
    }
}

impl super::Ppu {
    pub(super) fn draw_obj(&mut self, line: u16) {
        let sprite_start_index = self.select_sprites(line);
        let _tile_start_index = self.select_obj_tiles(line, sprite_start_index);
    }

    fn select_sprites(&mut self, line: u16) -> usize {
        let sprite_select_offset = self.oam.sprite_select_offset();
        let mut write_index = self.obj.selected_sprites.len();

        for index in 0..TOTAL_SPRITES {
            let sprite_index = sprite_select_offset + index;
            let sprite = self.oam.sprite(sprite_index);

            let start_y = sprite.y;
            let end_y = sprite.y + self.obj.size_y[sprite.size as usize];

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
                // TODO: Range over flag
                debug!("Line {}: Range Over", line);
                break;
            }

            write_index -= 1;
            self.obj.selected_sprites[write_index] = sprite_index;
        }

        let count = self.obj.selected_sprites.len() - write_index;

        if count > 0 {
            debug!("Line {}: {} Sprites Selected", line, count);
        }

        write_index
    }

    fn select_obj_tiles(&mut self, line: u16, sprite_start_index: usize) -> usize {
        let mut write_index = MAX_TILES_PER_LINE;

        'outer: for sprite_index in &self.obj.selected_sprites[sprite_start_index..] {
            let sprite = self.oam.sprite(*sprite_index);

            let flipped_y = {
                let pixel_y = line.wrapping_sub(sprite.y) & 255;

                if sprite.flip_y {
                    let size_y = self.obj.size_y[sprite.size as usize];
                    pixel_y ^ size_y
                } else {
                    pixel_y
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
                    // TODO: Time over flag
                    debug!("Line {}: Time Over", line);
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
            debug!("Line {}: {} Tiles Selected", line, count);
        }

        write_index
    }
}
