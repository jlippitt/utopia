use crate::util::memory::{Memory, Value};
use std::fmt;
use subslice::SubsliceExt;
use tracing::{info, warn};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum BackupType {
    None,
    Eeprom,
    Sram,
    Flash(usize),
}

const BACKUP_TYPES: [(&str, BackupType); 5] = [
    ("EEPROM_V", BackupType::Eeprom),
    ("SRAM_V", BackupType::Sram),
    ("FLASH_V", BackupType::Flash(65536)),
    ("FLASH512_V", BackupType::Flash(65536)),
    ("FLASH1M_V", BackupType::Flash(131072)),
];

pub struct Cartridge {
    rom: Memory,
    backup_type: BackupType,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let title = String::from_utf8_lossy(&rom[0xa0..=0xab]).into_owned();

        let backup_type =
            BACKUP_TYPES
                .iter()
                .fold(BackupType::None, |acc, (id_string, backup_type)| {
                    if rom.find(id_string.as_bytes()).is_none() {
                        return acc;
                    }

                    if acc != BackupType::None {
                        warn!("ROM contains multiple backup ID strings");
                        return acc;
                    }

                    *backup_type
                });

        info!("Title: {}", title);
        info!("ROM Size: {}", rom.len());
        info!("Backup Type: {}", backup_type);

        Self {
            rom: rom.into(),
            backup_type,
        }
    }

    pub fn read<T: Value>(&self, address: u32) -> T {
        let index = address as usize & 0x01ff_ffff;

        // TODO: ROM sizes >32MB
        if index < self.rom.len() {
            self.rom.read_le(index)
        } else if index >= 0x0100_0000 && self.backup_type == BackupType::Eeprom {
            todo!("EEPROM reads");
        } else {
            T::zero()
        }
    }
}

impl fmt::Display for BackupType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BackupType::None => write!(f, "None"),
            BackupType::Eeprom => write!(f, "EEPROM"),
            BackupType::Sram => write!(f, "SRAM"),
            BackupType::Flash(len) => write!(f, "Flash ({}K)", len / 1024),
        }
    }
}
