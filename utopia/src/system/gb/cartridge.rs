use crate::util::MirrorVec;
use tracing::info;

const BASE_ROM_SIZE: usize = 32768;

pub struct Cartridge {
    rom: MirrorVec<u8>,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let rom_size = BASE_ROM_SIZE << rom[0x0148];

        let ram_size = match rom[0x0149] {
            2 => 8192,
            3 => 32768,
            4 => 131072,
            5 => 65536,
            _ => 0,
        };

        info!("Title: {}", String::from_utf8_lossy(&rom[0x0134..=0x0143]));
        info!("Mapper Number: {}", rom[0x0147]);
        info!("ROM Size: {}", rom_size);
        info!("RAM Size: {}", ram_size);

        Self { rom: rom.into() }
    }

    pub fn read_rom(&self, index: usize) -> u8 {
        self.rom[index]
    }
}
