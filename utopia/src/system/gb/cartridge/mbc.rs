use enum_dispatch::enum_dispatch;
use mbc1::Mbc1;
use rom_only::RomOnly;

mod mbc1;
mod rom_only;

const ROM_PAGE_SIZE: usize = 16384;
//const RAM_PAGE_SIZE: usize = 8192;

#[enum_dispatch]
pub trait Mbc {
    fn write_register(&mut self, _mappings: &mut Mappings, _address: u16, _value: u8) {}
}

#[enum_dispatch(Mbc)]
pub enum MbcType {
    RomOnly,
    Mbc1,
}

impl MbcType {
    pub fn new(mapper_number: u8) -> Self {
        match mapper_number {
            0x00 => Self::RomOnly(RomOnly::new()),
            0x01..=0x03 => Self::Mbc1(Mbc1::new()),
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
    pub fn new() -> Self {
        Self {
            rom: [0, ROM_PAGE_SIZE],
            ram: None,
        }
    }
}
