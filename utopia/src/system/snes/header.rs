use tracing::trace;

const BASE_SIZE: usize = 0x0400;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mapper {
    LoRom,
    HiRom,
}

pub struct Header {
    pub title: String,
    pub mapper: Mapper,
    pub rom_size: usize,
    pub sram_size: usize,
}

pub fn parse(rom: &[u8]) -> Header {
    let lo_rom = try_parse(Mapper::LoRom, &rom[0x0000..]);
    let hi_rom = try_parse(Mapper::HiRom, &rom[0x8000..]);

    match (lo_rom, hi_rom) {
        (Some(lo_rom), Some(hi_rom)) => {
            let lo_rom_score = score(&lo_rom);
            let hi_rom_score = score(&hi_rom);

            if hi_rom_score > lo_rom_score {
                trace!("HiROM header found (score: {})", hi_rom_score);
                hi_rom
            } else {
                trace!("LoROM header found (score: {})", lo_rom_score);
                lo_rom
            }
        }
        (Some(lo_rom), None) => {
            trace!("LoROM header found");
            lo_rom
        }
        (None, Some(hi_rom)) => {
            trace!("HiROM header found");
            hi_rom
        }
        (None, None) => {
            trace!("No valid header found. Using default.");
            Header {
                title: String::new(),
                mapper: Mapper::LoRom,
                rom_size: rom.len(),
                sram_size: 0,
            }
        }
    }
}

fn try_parse(id: Mapper, rom: &[u8]) -> Option<Header> {
    let reset_vector = u16::from_le_bytes([rom[0x7ffc], rom[0x7ffd]]);

    if reset_vector < 0x8000 || reset_vector >= 0xffc0 {
        trace!(
            "{:?}: Invalid reset vector location: {:04X}",
            id,
            reset_vector
        );
        return None;
    }

    let expected_map_mode = match id {
        Mapper::LoRom => 0x20,
        Mapper::HiRom => 0x21,
    };

    if (rom[0x7fd5] & 0x21) != expected_map_mode {
        trace!("{:?}: Map mode {:02X} does not match", id, rom[0x7fd5]);
        return None;
    }

    let rom_size = match BASE_SIZE.checked_shl(rom[0x7fd7] as u32) {
        Some(rom_size) => rom_size,
        None => {
            trace!("{:?}: Invalid ROM size", id);
            return None;
        }
    };

    if rom_size != rom.len() {
        trace!(
            "{:?}: ROM size in header does not match ROM file length",
            id
        );
        return None;
    }

    let sram_size = if rom[0x7fd8] > 0 {
        match BASE_SIZE.checked_shl(rom[0x7fd8] as u32) {
            Some(sram_size) => sram_size,
            None => {
                trace!("{:?}: Invalid SRAM size", id);
                return None;
            }
        }
    } else {
        0
    };

    let title = String::from_utf8_lossy(&rom[0x7fc0..=0x7fd4]).into_owned();

    Some(Header {
        title,
        mapper: id,
        rom_size,
        sram_size,
    })
}

fn score(_header: &Header) -> i32 {
    // TODO
    0
}
