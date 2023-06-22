use crate::util::MirrorVec;

const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;
const PRG_ROM_MULTIPLIER: usize = 16384;
const CHR_ROM_MULTIPLIER: usize = 8192;

pub struct Cartridge {
    prg_rom: MirrorVec<u8>,
    chr_data: MirrorVec<u8>
}

impl Cartridge {
    pub fn new(data: Vec<u8>) -> Cartridge {
        let prg_rom_size = PRG_ROM_MULTIPLIER * (data[4] as usize);
        let chr_rom_size = CHR_ROM_MULTIPLIER * (data[5] as usize);
        let trainer_present = (data[6] & 0x04) != 0;

        let prg_rom_start = HEADER_SIZE + if trainer_present { TRAINER_SIZE } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;
        let chr_rom_end = chr_rom_start + chr_rom_size;

        let prg_rom = Vec::from(&data[prg_rom_start..chr_rom_start]);
        let chr_data = Vec::from(&data[chr_rom_start..chr_rom_end]);

        Self {
            prg_rom: prg_rom.into(),
            chr_data: chr_data.into()
        }
    }

    pub fn read_prg_rom(&self, index: u16) -> u8 {
        self.prg_rom[index as usize]
    }
}