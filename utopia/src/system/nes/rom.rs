const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;
const PRG_ROM_MULTIPLIER: usize = 16384;
const CHR_ROM_MULTIPLIER: usize = 8192;

pub struct ParsedRom {
    pub prg_rom: Vec<u8>,
    pub chr: Vec<u8>,
}

pub fn parse(raw_data: Vec<u8>) -> ParsedRom {
    let prg_rom_size = PRG_ROM_MULTIPLIER * (raw_data[4] as usize);
    let chr_rom_size = CHR_ROM_MULTIPLIER * (raw_data[5] as usize);
    let trainer_present = (raw_data[6] & 0x04) != 0;

    let prg_rom_start = HEADER_SIZE + if trainer_present { TRAINER_SIZE } else { 0 };
    let chr_rom_start = prg_rom_start + prg_rom_size;
    let chr_rom_end = chr_rom_start + chr_rom_size;

    let prg_rom = Vec::from(&raw_data[prg_rom_start..chr_rom_start]);
    let chr = Vec::from(&raw_data[chr_rom_start..chr_rom_end]);

    ParsedRom { prg_rom, chr }
}
