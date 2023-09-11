use bitfield_struct::bitfield;
use tracing::debug;

const OAM_SIZE: usize = 160;
const TOTAL_SPRITES: usize = OAM_SIZE / 4;
const MAX_SPRITES_PER_LINE: usize = 10;

#[bitfield(u8)]
pub struct SpriteAttrByte {
    #[bits(3)]
    pub cgb_palette: u8,
    pub bank: bool,
    pub dmg_palette: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub below_bg: bool,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Sprite {
    pub y: u8,
    pub x: u8,
    pub chr: u8,
    pub attr: SpriteAttrByte,
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
                sprite.y = value;
                debug!("Sprite {} Y: {}", index, sprite.y);
            }
            1 => {
                sprite.x = value;
                debug!("Sprite {} X: {}", index, sprite.x);
            }
            2 => {
                sprite.chr = value;
                debug!("Sprite {} CHR: {}", index, sprite.chr);
            }
            3 => {
                sprite.attr = value.into();
                debug!("Sprite {} Attr: {:?}", index, sprite.attr);
            }
            _ => unreachable!(),
        }
    }

    pub fn select_sprites(&mut self, line: u8, size: u8) {
        let obj_line = line + 16;

        self.read_index = 0;
        self.write_index = 0;

        for (sprite_id, sprite) in self.sprites.iter().enumerate() {
            if sprite.y > obj_line || (sprite.y + size) <= obj_line {
                continue;
            }

            if self.write_index >= MAX_SPRITES_PER_LINE {
                debug!("Line {}: Sprite Overflow", line);
                break;
            }

            let mut insert_index = self.write_index;

            // Sort by X coordinate upon insert
            while insert_index > 0 {
                let prev_sprite_id = self.selected[insert_index - 1];

                if sprite.x >= self.sprites[prev_sprite_id].x {
                    break;
                }

                self.selected[insert_index] = prev_sprite_id;
                insert_index -= 1;
            }

            self.selected[insert_index] = sprite_id;

            self.write_index += 1;
        }

        debug!("Line {}: {} Sprites Selected", line, self.write_index);
    }

    pub fn sprite_ready(&self, pos_x: u8) -> bool {
        if self.read_index >= self.write_index {
            return false;
        }

        if self.current_sprite().x > pos_x + 8 {
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
