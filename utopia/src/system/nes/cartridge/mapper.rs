use enum_dispatch::enum_dispatch;
use nrom::NROM;
use uxrom::UxROM;

mod nrom;
mod uxrom;

const PRG_PAGE_SIZE: usize = 4096;

#[enum_dispatch]
pub trait Mapper {
    fn init_mappings(&mut self, _mappings: &mut Mappings) {}
    fn write_register(&mut self, _mappings: &mut Mappings, _address: u16, _value: u8) {}
}

#[enum_dispatch(Mapper)]
pub enum MapperType {
    NROM,
    UxROM,
}

impl MapperType {
    pub fn new(mapper_number: u8, prg_rom_size: usize) -> Self {
        match mapper_number {
            0 => Self::NROM(NROM::new()),
            2 => Self::UxROM(UxROM::new(prg_rom_size)),
            _ => panic!("Mapper {} not yet supported", mapper_number),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PrgRead {
    Rom(u32),
    //Ram(u32),
    None,
}

#[derive(Clone, Copy, Debug)]
pub enum PrgWrite {
    Register,
    //Ram(u32),
    None,
}

#[derive(Clone, Copy, Debug)]
pub enum MirrorMode {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug)]
pub enum NameTable {
    Low,
    High,
}

pub struct Mappings {
    pub prg_read: [PrgRead; 16],
    pub prg_write: [PrgWrite; 16],
    pub name: [NameTable; 4],
}

impl Mappings {
    pub fn new(mirror_mode: MirrorMode) -> Self {
        Self {
            prg_read: [
                PrgRead::None,
                PrgRead::None,
                PrgRead::None,
                PrgRead::None,
                PrgRead::None,
                PrgRead::None,
                PrgRead::None,
                PrgRead::None,
                PrgRead::Rom(0),
                PrgRead::Rom(PRG_PAGE_SIZE as u32),
                PrgRead::Rom(PRG_PAGE_SIZE as u32 * 2),
                PrgRead::Rom(PRG_PAGE_SIZE as u32 * 3),
                PrgRead::Rom(PRG_PAGE_SIZE as u32 * 4),
                PrgRead::Rom(PRG_PAGE_SIZE as u32 * 5),
                PrgRead::Rom(PRG_PAGE_SIZE as u32 * 6),
                PrgRead::Rom(PRG_PAGE_SIZE as u32 * 7),
            ],
            prg_write: [PrgWrite::None; 16],
            name: match mirror_mode {
                MirrorMode::Horizontal => [
                    NameTable::Low,
                    NameTable::Low,
                    NameTable::High,
                    NameTable::High,
                ],
                MirrorMode::Vertical => [
                    NameTable::Low,
                    NameTable::High,
                    NameTable::Low,
                    NameTable::High,
                ],
            },
        }
    }

    pub fn map_prg_rom(&mut self, start: usize, len: usize, base_offset: usize) {
        for index in 0..len {
            let offset = base_offset + index * PRG_PAGE_SIZE;
            self.prg_read[start + index] = PrgRead::Rom(offset.try_into().unwrap());
        }
    }

    pub fn map_registers(&mut self, start: usize, len: usize) {
        self.prg_write[start..(start + len)].fill(PrgWrite::Register);
    }
}
