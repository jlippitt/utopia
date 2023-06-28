use tracing::debug;

pub struct Oam {
    address: u8,
    primary: [u8; 256],
    secondary: [u8; 32],
}

impl Oam {
    pub fn new() -> Self {
        Self {
            address: 0,
            primary: [0; 256],
            secondary: [0; 32],
        }
    }

    pub fn set_address(&mut self, value: u8) {
        self.address = value;
        debug!("OAM Address: {:02X}", self.address);
    }

    pub fn read(&self) -> u8 {
        let value = self.primary[self.address as usize];
        debug!("OAM Read: {:02X} => {:02X}", self.address, value);
        value
    }

    pub fn write(&mut self, value: u8) {
        debug!("OAM Write: {:02X} <= {:02X}", self.address, value);
        self.primary[self.address as usize] = value;
        self.address = self.address.wrapping_add(1);
    }

    pub fn read_secondary(&self, index: usize) -> u8 {
        self.secondary[index]
    }

    pub fn select_sprites(&mut self, line: i32, sprite_size: bool) -> (usize, bool) {
        self.secondary.fill(0xff);

        let height = 8 << sprite_size as u32;

        let mut m = 0;

        let mut write_index = 0;

        let mut sprite_zero_selected: bool = false;

        for n in 0..=63 {
            let read_index = (n << 2) + m;

            let sprite_y = self.primary[read_index];

            if write_index < self.secondary.len() {
                self.secondary[write_index] = sprite_y;

                if (sprite_y as i32) <= line && (sprite_y as i32 + height) > line {
                    self.secondary[write_index + 1] = self.primary[read_index + 1];
                    self.secondary[write_index + 2] = self.primary[read_index + 2];
                    self.secondary[write_index + 3] = self.primary[read_index + 3];
                    write_index += 4;

                    if n == 0 {
                        debug!("Line {}: Sprite Zero Selected", line);
                        sprite_zero_selected = true;
                    }
                }
            } else {
                if (sprite_y as i32) <= line && (sprite_y as i32 + height) > line {
                    debug!("Line {}: Sprite Overflow", line);
                    // TODO: Set sprite overflow flag
                } else {
                    // Sprite overflow bug
                    m = (m + 1) & 3;
                }
            }
        }

        let sprites_selected = write_index >> 2;

        if write_index > 0 {
            debug!("Line {}: {} Sprites Selected", line, sprites_selected);
        }

        (sprites_selected, sprite_zero_selected)
    }
}
