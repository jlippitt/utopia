use enum_dispatch::enum_dispatch;
use mbc1::Mbc1;
use mbc3::Mbc3;
use mbc5::Mbc5;
use rom_only::RomOnly;

mod mbc1;
mod mbc3;
mod mbc5;
mod rom_only;

#[enum_dispatch]
pub trait Mbc {
    fn init_mappings(&mut self, _mappings: &mut Mappings) {}
    fn write_register(&mut self, _mappings: &mut Mappings, _address: u16, _value: u8) {}
}

#[enum_dispatch(Mbc)]
pub enum MbcType {
    RomOnly,
    Mbc1,
    Mbc3,
    Mbc5,
}

impl MbcType {
    pub fn new(mapper_number: u8) -> Self {
        match mapper_number {
            0x00 => Self::RomOnly(RomOnly::new()),
            0x01..=0x03 => Self::Mbc1(Mbc1::new()),
            0x0f..=0x13 => Self::Mbc3(Mbc3::new()),
            0x19..=0x1e => Self::Mbc5(Mbc5::new()),
            _ => panic!("Mapper {:02X} not yet supported", mapper_number),
        }
    }
}

#[derive(Debug)]
pub struct Mappings {
    pub rom: [usize; 2],
    pub ram: Option<usize>,
}

impl Mappings {
    const ROM_PAGE_SIZE: usize = 16384;
    const RAM_PAGE_SIZE: usize = 8192;

    pub fn new() -> Self {
        Self {
            rom: [0, Self::ROM_PAGE_SIZE],
            ram: None,
        }
    }
}
