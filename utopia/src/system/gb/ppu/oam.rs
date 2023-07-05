use tracing::debug;

const OAM_SIZE: usize = 160;
const TOTAL_SPRITES: usize = OAM_SIZE / 4;
const MAX_SPRITES_PER_LINE: usize = 10;

#[derive(Copy, Clone, Default)]
struct Sprite {
    y: i32,
    x: i32,
    chr: u8,
    palette: bool,
    flip_x: bool,
    flip_y: bool,
    below_bg: bool,
}

pub struct Oam {
    data: Vec<u8>,
    sprites: Vec<Sprite>,
    selected: [usize; MAX_SPRITES_PER_LINE],
}

impl Oam {
    pub fn new() -> Self {
        Self {
            data: vec![0; OAM_SIZE],
            sprites: vec![Default::default(); TOTAL_SPRITES],
            selected: [0; MAX_SPRITES_PER_LINE],
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
        let mut write_index: usize = 0;

        for (read_index, sprite) in self.sprites.iter().enumerate() {
            // TODO: 8x16 sprites
            if sprite.y > line || (sprite.y + 8) <= line {
                continue;
            }

            if write_index >= MAX_SPRITES_PER_LINE {
                debug!("Line {}: Sprite Overflow", line);
                break;
            }

            let mut insert_index = write_index;

            // Sort by X coordinate upon insert
            while insert_index > 0 {
                if sprite.x >= self.sprites[insert_index - 1].x {
                    break;
                }

                self.selected[insert_index] = self.selected[insert_index - 1];
                insert_index -= 1;
            }

            self.selected[insert_index] = read_index;

            write_index += 1;
        }

        debug!("Line {}: {} Sprites Selected", line, write_index);
    }
}
