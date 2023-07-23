use crate::util::MirrorVec;
use tracing::debug;

const LOWER_TABLE_SIZE: usize = 256;
const UPPER_TABLE_SIZE: usize = 16;
const TOTAL_SPRITES: usize = 128;

const SPRITE_PALETTE_OFFSET: u8 = 128;

const LAYER_OBJ_COLOR_MATH: u8 = 0x10;
const LAYER_OBJ_NO_COLOR_MATH: u8 = 0x40;

#[derive(Clone, Default, Debug)]
pub struct Sprite {
    x: u16,
    y: u16,
    name: u16,
    table: bool,
    palette: u8,
    priority: u8,
    flip_x: bool,
    flip_y: bool,
    layer: u8,
    size: bool,
}

pub struct Oam {
    external_address: u16,
    internal_address: u16,
    high_byte: bool,
    buffer: u8,
    priority_enabled: bool,
    sprites: MirrorVec<Sprite>,
    lower_table: [u16; LOWER_TABLE_SIZE],
    upper_table: [u16; UPPER_TABLE_SIZE],
}

impl Oam {
    pub fn new() -> Self {
        Self {
            external_address: 0,
            internal_address: 0,
            high_byte: false,
            buffer: 0,
            priority_enabled: false,
            sprites: MirrorVec::new(TOTAL_SPRITES),
            lower_table: [0; LOWER_TABLE_SIZE],
            upper_table: [0; UPPER_TABLE_SIZE],
        }
    }

    pub fn reload_internal_address(&mut self) {
        self.internal_address = self.external_address;
        debug!("OAM Internal Address: {:04X}", self.internal_address);
        self.high_byte = false;
    }

    pub fn set_address_low(&mut self, value: u8) {
        self.external_address = (self.external_address & 0xff00) | (value as u16);
        debug!("OAM External Address: {:04X}", self.external_address);
        self.reload_internal_address();
    }

    pub fn set_address_high(&mut self, value: u8) {
        self.external_address = (self.external_address & 0xff) | ((value as u16 & 0x01) << 8);
        debug!("OAM External Address: {:04X}", self.external_address);
        self.reload_internal_address();
        self.priority_enabled = (value & 0x80) != 0;
        debug!("OAM Priority Enabled: {}", self.priority_enabled);
    }

    pub fn write(&mut self, value: u8) {
        let address = self.internal_address as usize;

        if address < LOWER_TABLE_SIZE {
            if self.high_byte {
                let word_value = ((value as u16 & 0x7f) << 8) | (self.buffer as u16);

                self.lower_table[address] = word_value;

                debug!(
                    "OAM Write (Lower Table): {:02X} <= {:04X}",
                    address, word_value
                );

                self.update_sprite_cache_lower(address, word_value);

                self.internal_address = self.internal_address.wrapping_add(1) & 0x01ff;
            } else {
                self.buffer = value;
            }
        } else {
            let address = address & 15;

            if self.high_byte {
                self.upper_table[address] =
                    (self.upper_table[address] & 0xff) | ((value as u16) << 8);
                self.internal_address = self.internal_address.wrapping_add(1) & 0x01ff;
            } else {
                self.upper_table[address] = (self.upper_table[address] & 0xff00) | (value as u16);
            }

            debug!(
                "OAM Write (Upper Table): {:02X}.{} <= {:02X}",
                address, self.high_byte as u32, value
            );

            self.update_sprite_cache_upper(address, self.high_byte, value);
        }

        self.high_byte = !self.high_byte;
    }

    fn update_sprite_cache_lower(&mut self, address: usize, word_value: u16) {
        let id = address >> 1;
        let sprite = &mut self.sprites[id];

        if (address & 1) == 0 {
            sprite.x = (sprite.x & 0xff00) | (word_value & 0xff);
            sprite.y = word_value >> 8;
            debug!("Sprite {} X: {}", id, sprite.x);
            debug!("Sprite {} Y: {}", id, sprite.y);
        } else {
            sprite.name = word_value & 0xff;
            sprite.table = (word_value & 0x0100) != 0;
            sprite.palette = SPRITE_PALETTE_OFFSET + (((word_value & 0x0e00) >> 5) as u8);
            sprite.priority = ((word_value & 0x3000) >> 12) as u8 + 1;
            sprite.flip_x = (word_value & 0x4000) != 0;
            sprite.flip_y = (word_value & 0x8000) != 0;

            sprite.layer = if sprite.palette >= 192 {
                LAYER_OBJ_NO_COLOR_MATH
            } else {
                LAYER_OBJ_COLOR_MATH
            };

            debug!("Sprite {} Name: {:02X}", id, sprite.x);
            debug!("Sprite {} Table: {}", id, sprite.table);
            debug!("Sprite {} Palette: {}", id, sprite.palette);
            debug!("Sprite {} Priority: {}", id, sprite.priority);
            debug!("Sprite {} Flip X: {}", id, sprite.flip_x);
            debug!("Sprite {} Flip Y: {}", id, sprite.flip_y);
            debug!("Sprite {} Layer: {:08b}", id, sprite.layer);
        }
    }

    fn update_sprite_cache_upper(&mut self, address: usize, high_byte: bool, mut value: u8) {
        let start = ((address << 1) + high_byte as usize) << 2;

        for id in start..(start + 4) {
            let sprite = &mut self.sprites[id];
            sprite.x = (sprite.x & 0xff) | ((value as u16 & 0x01) << 8);
            sprite.size = (value & 0x02) != 0;
            debug!("Sprite {} X: {}", id, sprite.x);
            debug!("Sprite {} Size: {}", id, sprite.size);
            value >>= 2;
        }
    }
}
