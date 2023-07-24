use tracing::trace;

const BASE_SIZE: usize = 0x0400;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum HeaderLocation {
    LoRom,
    HiRom,
}

pub struct Header {
    pub title: String,
    pub map_mode: u8,
    pub rom_size: usize,
    pub sram_size: usize,
}

pub fn parse(rom: &[u8]) -> Header {
    let lo_rom = try_parse(HeaderLocation::LoRom, &rom[0x0000..]);

    let hi_rom = if rom.len() > 0x0001_0000 {
        try_parse(HeaderLocation::HiRom, &rom[0x8000..])
    } else {
        None
    };

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
                map_mode: 0x20,
                rom_size: rom.len(),
                sram_size: 0,
            }
        }
    }
}

fn try_parse(id: HeaderLocation, rom: &[u8]) -> Option<Header> {
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
        HeaderLocation::LoRom => 0x20,
        HeaderLocation::HiRom => 0x21,
    };

    let map_mode = rom[0x7fd5];

    if (map_mode & 0x21) != expected_map_mode {
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

    if rom_size != rom.len().next_power_of_two() {
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
        map_mode,
        rom_size,
        sram_size,
    })
}

fn score(_header: &Header) -> i32 {
    // TODO
    0
}
