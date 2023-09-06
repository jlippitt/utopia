use super::super::interrupt::{Interrupt, InterruptType};
use crate::util::MirrorVec;
use axrom::AxRom;
use cnrom::CnRom;
use enum_dispatch::enum_dispatch;
use fme7::Fme7;
use mmc1::Mmc1;
use mmc2::Mmc2;
use mmc3::Mmc3;
use mmc5::Mmc5;
use nrom::NRom;
use uxrom::UxRom;
use vrc6::Vrc6;

mod axrom;
mod cnrom;
mod fme7;
mod mmc1;
mod mmc2;
mod mmc3;
mod mmc5;
mod nrom;
mod uxrom;
mod vrc6;

const PRG_PAGE_SIZE: usize = 4096;
const CHR_PAGE_SIZE: usize = 1024;

#[enum_dispatch]
pub trait Mapper {
    fn init_mappings(&mut self, _mappings: &mut Mappings) {}

    fn read_register(&mut self, _mappings: &mut Mappings, _address: u16, prev_value: u8) -> u8 {
        prev_value
    }

    fn write_register(&mut self, _mappings: &mut Mappings, _address: u16, _value: u8) {}

    fn read_name(&mut self, mappings: &mut Mappings, ci_ram: &MirrorVec<u8>, address: u16) -> u8 {
        let index = address as usize & 0x0fff;
        let offset = mappings.name[index >> 10];
        ci_ram[offset as usize | (index & 0x03ff)]
    }

    fn write_name(
        &mut self,
        mappings: &mut Mappings,
        ci_ram: &mut MirrorVec<u8>,
        address: u16,
        value: u8,
    ) {
        let index = address as usize & 0x0fff;
        let offset = mappings.name[index >> 10];
        ci_ram[offset as usize | (index & 0x03ff)] = value;
    }

    fn on_cpu_cycle(&mut self, _mappings: &mut Mappings) {}

    fn on_ppu_address_changed(&mut self, _ppu_address: u16) {}

    fn on_ppu_chr_fetch(&mut self, _mappings: &mut Mappings, _ppu_address: u16) {}
}

#[enum_dispatch(Mapper)]
pub enum MapperType {
    NRom,
    Mmc1,
    Mmc2,
    Mmc3,
    Mmc5,
    UxRom,
    CnRom,
    AxRom,
    Vrc6,
    Fme7,
}

impl MapperType {
    pub fn new(mapper_number: u8, prg_rom_size: usize, interrupt: Interrupt) -> Self {
        match mapper_number {
            0 => Self::NRom(NRom::new()),
            1 => Self::Mmc1(Mmc1::new(prg_rom_size)),
            2 => Self::UxRom(UxRom::new(prg_rom_size)),
            3 => Self::CnRom(CnRom::new()),
            4 => Self::Mmc3(Mmc3::new(prg_rom_size, interrupt)),
            5 => Self::Mmc5(Mmc5::new(interrupt)),
            7 => Self::AxRom(AxRom::new()),
            9 => Self::Mmc2(Mmc2::new(prg_rom_size)),
            24 => Self::Vrc6(Vrc6::new(prg_rom_size, interrupt)),
            69 => Self::Fme7(Fme7::new(prg_rom_size, interrupt)),
            _ => panic!("Mapper {} not yet supported", mapper_number),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PrgRead {
    Rom(u32),
    Ram(u32),
    Register,
    None,
}

#[derive(Clone, Copy, Debug)]
pub enum PrgWrite {
    Ram(u32),
    Register,
    None,
}

#[derive(Clone, Copy, Debug)]
pub enum MirrorMode {
    Horizontal,
    Vertical,
}

struct NameTable;

impl NameTable {
    const LOW: u16 = 0x0000;
    const HIGH: u16 = 0x0400;
}

const MIRROR_HORIZONTAL: [u16; 4] = [
    NameTable::LOW,
    NameTable::LOW,
    NameTable::HIGH,
    NameTable::HIGH,
];

const MIRROR_VERTICAL: [u16; 4] = [
    NameTable::LOW,
    NameTable::HIGH,
    NameTable::LOW,
    NameTable::HIGH,
];

pub struct Mappings {
    pub prg_read: [PrgRead; 16],
    pub prg_write: [PrgWrite; 16],
    pub name: [u16; 4],
    pub chr: [u32; 8],
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
                MirrorMode::Horizontal => MIRROR_HORIZONTAL,
                MirrorMode::Vertical => MIRROR_VERTICAL,
            },
            chr: [
                0,
                CHR_PAGE_SIZE as u32,
                CHR_PAGE_SIZE as u32 * 2,
                CHR_PAGE_SIZE as u32 * 3,
                CHR_PAGE_SIZE as u32 * 4,
                CHR_PAGE_SIZE as u32 * 5,
                CHR_PAGE_SIZE as u32 * 6,
                CHR_PAGE_SIZE as u32 * 7,
            ],
        }
    }

    pub fn map_prg_rom(&mut self, start: usize, len: usize, base_offset: usize) {
        for index in 0..len {
            let offset = base_offset + index * PRG_PAGE_SIZE;
            self.prg_read[start + index] = PrgRead::Rom(offset.try_into().unwrap());
        }
    }

    pub fn map_prg_ram(&mut self, start: usize, len: usize, base_offset: usize) {
        for index in 0..len {
            let offset = base_offset + index * PRG_PAGE_SIZE;
            self.prg_read[start + index] = PrgRead::Ram(offset.try_into().unwrap());
            self.prg_write[start + index] = PrgWrite::Ram(offset.try_into().unwrap());
        }
    }

    pub fn map_prg_ram_read_only(&mut self, start: usize, len: usize, base_offset: usize) {
        for index in 0..len {
            let offset = base_offset + index * PRG_PAGE_SIZE;
            self.prg_read[start + index] = PrgRead::Ram(offset.try_into().unwrap());
            self.prg_write[start + index] = PrgWrite::None;
        }
    }

    pub fn map_registers(&mut self, start: usize, len: usize) {
        self.prg_write[start..(start + len)].fill(PrgWrite::Register);
    }

    pub fn map_registers_with_read(&mut self, start: usize, len: usize) {
        self.prg_read[start..(start + len)].fill(PrgRead::Register);
        self.map_registers(start, len);
    }

    pub fn map_chr(&mut self, start: usize, len: usize, base_offset: usize) {
        for index in 0..len {
            let offset = base_offset + index * CHR_PAGE_SIZE;
            self.chr[start + index] = offset.try_into().unwrap();
        }
    }

    pub fn mirror_nametables(&mut self, mirror_mode: MirrorMode) {
        self.name = match mirror_mode {
            MirrorMode::Horizontal => MIRROR_HORIZONTAL,
            MirrorMode::Vertical => MIRROR_VERTICAL,
        }
    }

    pub fn unmap_prg(&mut self, start: usize, len: usize) {
        for index in 0..len {
            self.prg_read[start + index] = PrgRead::None;
            self.prg_write[start + index] = PrgWrite::None;
        }
    }

    pub fn unmap_prg_write(&mut self, start: usize, len: usize) {
        for index in 0..len {
            self.prg_write[start + index] = PrgWrite::None;
        }
    }
}
