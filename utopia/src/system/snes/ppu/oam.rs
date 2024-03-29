use crate::util::MirrorVec;
use tracing::trace;

pub const TOTAL_SPRITES: usize = 128;

const LOWER_TABLE_SIZE: usize = 256;
const UPPER_TABLE_SIZE: usize = 16;

const SPRITE_PALETTE_OFFSET: u16 = 128;

#[derive(Clone, Default, Debug)]
pub struct Sprite {
    pub x: u16,
    pub y: u16,
    pub name: u16,
    pub table: bool,
    pub palette: u16,
    pub priority: u8,
    pub flip_x: bool,
    pub flip_y: bool,
    pub size: bool,
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

    pub fn sprite_select_offset(&self) -> usize {
        if self.priority_enabled {
            (self.external_address as usize >> 1) & 0x7f
        } else {
            0
        }
    }

    pub fn sprite(&mut self, index: usize) -> &Sprite {
        &self.sprites[index]
    }

    pub fn reload_internal_address(&mut self) {
        self.internal_address = self.external_address;
        trace!("OAM Internal Address: {:04X}", self.internal_address);
        self.high_byte = false;
    }

    pub fn set_address_low(&mut self, value: u8) {
        self.external_address = (self.external_address & 0xff00) | (value as u16);
        trace!("OAM External Address: {:04X}", self.external_address);
        self.reload_internal_address();
    }

    pub fn set_address_high(&mut self, value: u8) {
        self.external_address = (self.external_address & 0xff) | ((value as u16 & 0x01) << 8);
        trace!("OAM External Address: {:04X}", self.external_address);
        self.reload_internal_address();
        self.priority_enabled = (value & 0x80) != 0;
        trace!("OAM Priority Enabled: {}", self.priority_enabled);
    }

    pub fn read(&mut self) -> u8 {
        let address = self.internal_address as usize;

        let value = if address < LOWER_TABLE_SIZE {
            if self.high_byte {
                let value = (self.lower_table[address] >> 8) as u8;
                self.internal_address = self.internal_address.wrapping_add(1) & 0x01ff;
                value
            } else {
                self.lower_table[address] as u8
            }
        } else {
            let address = address & 15;

            if self.high_byte {
                let value = (self.upper_table[address] >> 8) as u8;
                self.internal_address = self.internal_address.wrapping_add(1) & 0x01ff;
                value
            } else {
                self.upper_table[address] as u8
            }
        };

        trace!(
            "OAM Read: {:02X}.{} => {:02X}",
            address,
            self.high_byte as u32,
            value
        );

        self.high_byte = !self.high_byte;

        value
    }

    pub fn write(&mut self, value: u8) {
        let address = self.internal_address as usize;

        if address < LOWER_TABLE_SIZE {
            if self.high_byte {
                let word_value = ((value as u16) << 8) | (self.buffer as u16);

                self.lower_table[address] = word_value;

                trace!(
                    "OAM Write (Lower Table): {:02X} <= {:04X}",
                    address,
                    word_value
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

            trace!(
                "OAM Write (Upper Table): {:02X}.{} <= {:02X}",
                address,
                self.high_byte as u32,
                value
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
            trace!("Sprite {} X: {}", id, sprite.x);
            trace!("Sprite {} Y: {}", id, sprite.y);
        } else {
            sprite.name = word_value & 0xff;
            sprite.table = (word_value & 0x0100) != 0;
            sprite.palette = SPRITE_PALETTE_OFFSET + ((word_value & 0x0e00) >> 5);
            sprite.priority = ((word_value & 0x3000) >> 12) as u8 + 1;
            sprite.flip_x = (word_value & 0x4000) != 0;
            sprite.flip_y = (word_value & 0x8000) != 0;

            trace!("Sprite {} Name: {:02X}", id, sprite.name);
            trace!("Sprite {} Table: {}", id, sprite.table);
            trace!("Sprite {} Palette: {}", id, sprite.palette);
            trace!("Sprite {} Priority: {}", id, sprite.priority);
            trace!("Sprite {} Flip X: {}", id, sprite.flip_x);
            trace!("Sprite {} Flip Y: {}", id, sprite.flip_y);
        }
    }

    fn update_sprite_cache_upper(&mut self, address: usize, high_byte: bool, mut value: u8) {
        let start = ((address << 1) + high_byte as usize) << 2;

        for id in start..(start + 4) {
            let sprite = &mut self.sprites[id];
            sprite.x = (sprite.x & 0xff) | ((value as u16 & 0x01) << 8);
            sprite.size = (value & 0x02) != 0;
            trace!("Sprite {} X: {}", id, sprite.x);
            trace!("Sprite {} Size: {}", id, sprite.size);
            value >>= 2;
        }
    }
}
