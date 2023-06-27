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

    pub fn select_sprites(&mut self, line: i32) {
        self.secondary.fill(0xff);

        let mut m = 0;

        let mut write_index = 0;

        for n in 0..=63 {
            let read_index = (n << 2) + m;
            let sprite = &self.primary[read_index..(read_index + 4)];
            let sprite_y = sprite[0];

            if write_index <= self.secondary.len() {
                self.secondary[write_index] = sprite_y;

                // TODO: 8x16 sprites
                if (sprite_y as i32) < line && (sprite_y as i32 + 8) >= line {
                    self.secondary[write_index + 1] = sprite[1];
                    self.secondary[write_index + 2] = sprite[2];
                    self.secondary[write_index + 3] = sprite[3];
                    write_index += 4;
                }
            } else {
                if (sprite_y as i32) < line && (sprite_y as i32 + 8) >= line {
                    debug!("Line {}: Sprite overflow", line);
                    // TODO: Set sprite overflow flag
                } else {
                    // Sprite overflow bug
                    m += 1;
                }
            }
        }

        if write_index > 0 {
            debug!("Line {}: {} sprites selected", line, write_index >> 2);
        }
    }
}
