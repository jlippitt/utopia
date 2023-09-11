use bitfield_struct::bitfield;
use tracing::debug;

#[bitfield(u16)]
pub struct Color {
    #[bits(5)]
    pub red: u8,
    #[bits(5)]
    pub green: u8,
    #[bits(5)]
    pub blue: u8,
    __: bool,
}

pub struct Palette {
    address: u8,
    auto_increment: bool,
    name: &'static str,
    data: [Color; 32],
}

impl Palette {
    pub fn new(name: &'static str) -> Self {
        Self {
            address: 0,
            auto_increment: false,
            name,
            data: [Color::new(); 32],
        }
    }

    pub fn color(&self, palette_index: u8, color_index: u8) -> Color {
        self.data[((palette_index << 2) + color_index) as usize]
    }

    pub fn address(&self) -> u8 {
        let mut value = self.address;
        value |= if self.auto_increment { 0x80 } else { 0 };
        value
    }

    pub fn set_address(&mut self, value: u8) {
        self.address = value & 0x3f;
        self.auto_increment = (value & 0x80) != 0;
        debug!("{} Palette Address: {}", self.name, self.address);
        debug!(
            "{} Palette Auto-Increment: {}",
            self.name, self.auto_increment
        );
    }

    pub fn read(&self) -> u8 {
        let color = &self.data[self.address as usize >> 1];

        let value = if (self.address & 1) != 0 {
            (u16::from(*color) >> 8) as u8
        } else {
            u16::from(*color) as u8
        };

        debug!(
            "{} Palette Read: {:02X} => {:02X} ({:04X})",
            self.name,
            self.address,
            value,
            u16::from(*color)
        );

        // Note: No auto-increment after read

        value
    }

    pub fn write(&mut self, value: u8) {
        let color = &mut self.data[self.address as usize >> 1];

        *color = if (self.address & 1) != 0 {
            ((u16::from(*color) & 0xff) | ((value as u16 & 0x7f) << 8)).into()
        } else {
            ((u16::from(*color) & 0xff00) | value as u16).into()
        };

        debug!(
            "{} Palette Write: {:02X} <= {:02X} ({:04X})",
            self.name,
            self.address,
            value,
            u16::from(*color)
        );

        if self.auto_increment {
            self.address = (self.address + 1) & 0x3f;
        }
    }
}
