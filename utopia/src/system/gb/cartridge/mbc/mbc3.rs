use super::{Mappings, Mbc, RamMapping};
use tracing::debug;

#[derive(Clone, Default)]
struct RtcState {
    second: u8,
    minute: u8,
    hour: u8,
    day: u16,
    halted: bool,
    carry: bool,
}

pub struct Mbc3 {
    rom_bank: u8,
    ram_bank: u8,
    ram_enable: bool,
    rtc_current: RtcState,
    rtc_latched: RtcState,
}

impl Mbc3 {
    pub fn new() -> Self {
        Self {
            ram_enable: false,
            rom_bank: 1,
            ram_bank: 0,
            rtc_current: Default::default(),
            rtc_latched: Default::default(),
        }
    }

    fn update_mappings(&self, mappings: &mut Mappings) {
        mappings.rom[1] = Mappings::ROM_PAGE_SIZE * (self.rom_bank as usize);

        mappings.ram = if self.ram_enable {
            if self.ram_bank <= 0x03 {
                RamMapping::Offset(Mappings::RAM_PAGE_SIZE * (self.ram_bank as usize))
            } else {
                RamMapping::Custom
            }
        } else {
            RamMapping::None
        };
    }
}

impl Mbc for Mbc3 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.rom[0] = 0;
        self.update_mappings(mappings)
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        match address & 0xe000 {
            0x0000 => {
                self.ram_enable = (value & 0x0f) == 0x0a;
                debug!("MBC3 RAM Enable: {}", self.ram_enable);
            }
            0x2000 => {
                self.rom_bank = value & 0x7f;

                // Value of 0 behaves as if it was set to 1
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }

                debug!("MBC3 ROM Bank: {}", self.rom_bank);
            }
            0x4000 => {
                self.ram_bank = value & 0x0f;
                debug!("MBC3 RAM Bank: {}", self.ram_bank);
            }
            0x6000 => {
                self.rtc_latched = self.rtc_current.clone();
                debug!("MBC3 RTC Latched");
            }
            _ => unreachable!(),
        }

        self.update_mappings(mappings);
    }

    fn read_ram(&self, _address: u16) -> u8 {
        match self.ram_bank {
            0x08 => self.rtc_latched.second,
            0x09 => self.rtc_latched.minute,
            0x0a => self.rtc_latched.hour,
            0x0b => self.rtc_latched.day as u8,
            0x0c => {
                let mut value = (self.rtc_latched.day >> 8) as u8 & 0x01;
                value |= if self.rtc_latched.halted { 0x40 } else { 0 };
                value |= if self.rtc_latched.carry { 0x80 } else { 0 };
                value
            }
            _ => unimplemented!("MBC3 RTC Register Read: {:02X}", self.ram_bank),
        }
    }

    fn write_ram(&mut self, _address: u16, value: u8) {
        match self.ram_bank {
            0x08 => self.rtc_current.second = value,
            0x09 => self.rtc_current.minute = value,
            0x0a => self.rtc_current.hour = value,
            0x0b => self.rtc_current.day = (self.rtc_current.day & 0xff00) | value as u16,
            0x0c => {
                self.rtc_current.day = (self.rtc_current.day & 0xff) | ((value as u16 & 0x01) << 8);
                self.rtc_current.halted = (value & 0x40) != 0;
                self.rtc_current.carry = (value & 0x80) != 0;
            }
            _ => unimplemented!(
                "MBC3 RTC Register Write: {:02X} <= {:02X}",
                self.ram_bank,
                value
            ),
        }
    }
}
