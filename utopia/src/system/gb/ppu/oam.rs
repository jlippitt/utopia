use tracing::debug;

const OAM_SIZE: usize = 160;
const TOTAL_SPRITES: usize = OAM_SIZE / 4;
const MAX_SPRITES_PER_LINE: usize = 10;

#[derive(Copy, Clone, Default)]
pub struct Sprite {
    pub y: i32,
    pub x: i32,
    pub chr: u8,
    pub palette: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub below_bg: bool,
}

pub struct Oam {
    data: Vec<u8>,
    sprites: Vec<Sprite>,
    selected: [usize; MAX_SPRITES_PER_LINE],
    read_index: usize,
    write_index: usize,
}

impl Oam {
    pub fn new() -> Self {
        Self {
            data: vec![0; OAM_SIZE],
            sprites: vec![Default::default(); TOTAL_SPRITES],
            selected: [0; MAX_SPRITES_PER_LINE],
            read_index: 0,
            write_index: 0,
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        self.data[address as usize]
    }

    pub fn write(&mut self, address: u8, value: u8) {
        self.data[address as usize] = value;

        let index = address as usize >> 2;
        let sprite = &mut self.sprites[index];

        match address & 3 {
            0 => {
                sprite.y = (value as i32) - 16;
                debug!("Sprite {} Y: {}", index, sprite.y);
            }
            1 => {
                sprite.x = (value as i32) - 8;
                debug!("Sprite {} X: {}", index, sprite.x);
            }
            2 => {
                sprite.chr = value;
                debug!("Sprite {} CHR: {}", index, sprite.chr);
            }
            3 => {
                sprite.palette = (value & 0x10) != 0;
                sprite.flip_x = (value & 0x20) != 0;
                sprite.flip_y = (value & 0x40) != 0;
                sprite.below_bg = (value & 0x80) != 0;
                debug!("Sprite {} Palette: {}", index, sprite.palette as u32);
                debug!("Sprite {} Flip X: {}", index, sprite.flip_x);
                debug!("Sprite {} Flip Y: {}", index, sprite.flip_y);
                debug!("Sprite {} Below BG: {}", index, sprite.below_bg);
            }
            _ => unreachable!(),
        }
    }

    pub fn select_sprites(&mut self, line: i32) {
        self.read_index = 0;
        self.write_index = 0;

        for (sprite_id, sprite) in self.sprites.iter().enumerate() {
            // TODO: 8x16 sprites
            if sprite.y > line || (sprite.y + 8) <= line {
                continue;
            }

            if self.write_index >= MAX_SPRITES_PER_LINE {
                debug!("Line {}: Sprite Overflow", line);
                break;
            }

            let mut insert_index = self.write_index;

            // Sort by X coordinate upon insert
            while insert_index > 0 {
                if sprite.x >= self.sprites[insert_index - 1].x {
                    break;
                }

                self.selected[insert_index] = self.selected[insert_index - 1];
                insert_index -= 1;
            }

            self.selected[insert_index] = sprite_id;

            self.write_index += 1;
        }

        debug!("Line {}: {} Sprites Selected", line, self.write_index);
    }

    pub fn sprite_ready(&self, pos_x: usize) -> bool {
        if self.read_index >= self.write_index {
            return false;
        }

        if self.current_sprite().x >= (pos_x as i32) {
            return false;
        }

        true
    }

    pub fn current_sprite(&self) -> &Sprite {
        &self.sprites[self.selected[self.read_index]]
    }

    pub fn next_sprite(&mut self) {
        self.read_index += 1;
    }
}
