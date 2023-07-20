use tracing::warn;

use vram::Vram;

mod vram;

pub struct Ppu {
    vram: Vram,
}

impl Ppu {
    pub fn new() -> Self {
        Self { vram: Vram::new() }
    }

    pub fn read(&mut self, address: u8) -> u8 {
        match address {
            _ => panic!("Unmapped PPU read: {:02X}", address),
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0x15 => self.vram.set_control(value),
            0x16 => self.vram.set_address_low(value),
            0x17 => self.vram.set_address_high(value),
            0x18 => self.vram.write_low(value),
            0x19 => self.vram.write_high(value),
            _ => warn!("Unmapped PPU write: {:02X} <= {:02X}", address, value),
        }
    }
}
