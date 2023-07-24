use super::oam::TOTAL_SPRITES;
use tracing::debug;

const MAX_SPRITES_PER_LINE: usize = 32;

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
        let _sprite_start_index = self.select_sprites(line);
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
}
