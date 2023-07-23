use tracing::debug;

const LOWER_TABLE_SIZE: usize = 256;
const UPPER_TABLE_SIZE: usize = 16;

pub struct Oam {
    lower_table: [u16; LOWER_TABLE_SIZE],
    upper_table: [u16; UPPER_TABLE_SIZE],
    external_address: u16,
    internal_address: u16,
    high_byte: bool,
    buffer: u8,
    priority_enabled: bool,
}

impl Oam {
    pub fn new() -> Self {
        Self {
            lower_table: [0; LOWER_TABLE_SIZE],
            upper_table: [0; UPPER_TABLE_SIZE],
            external_address: 0,
            internal_address: 0,
            high_byte: false,
            buffer: 0,
            priority_enabled: false,
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
        }

        self.high_byte = !self.high_byte;
    }
}
