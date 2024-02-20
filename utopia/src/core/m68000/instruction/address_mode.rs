use super::{Bus, Core, Size};
use std::fmt;
use std::mem;

#[derive(Copy, Clone)]
pub struct AddressMode(u8);

impl AddressMode {
    pub fn address(self, core: &mut Core<impl Bus>) -> u32 {
        #[allow(clippy::unusual_byte_groupings)]
        match self.0 {
            0b010_000..=0b100_111 => core.areg(self.reg()),
            0b101_000..=0b101_111 => self.areg_displacement(core),
            0b111_000 => self.absolute16(core),
            0b111_001 => self.absolute32(core),
            0b111_010 => self.displacement(core, core.pc),
            _ => unimplemented!("Address mode lookup: {:06b}", self.0),
        }
    }

    pub fn read<T: Size>(self, core: &mut Core<impl Bus>) -> T {
        #[allow(clippy::unusual_byte_groupings)]
        match self.0 {
            0b000_000..=0b000_111 => core.dreg(self.reg()),
            0b001_000..=0b001_111 => core.areg(self.reg()),
            0b010_000..=0b010_111 => {
                let address = core.areg(self.reg());
                core.read(address)
            }
            0b011_000..=0b011_111 => {
                let index = self.reg();
                let address = core.areg(index);
                let value = core.read(address);
                core.set_areg(index, address.wrapping_add(mem::size_of::<T>() as u32));
                value
            }
            0b101_000..=0b101_111 => {
                let address = self.areg_displacement(core);
                core.read(address)
            }
            0b111_000 => {
                let address = self.absolute16(core);
                core.read(address)
            }
            0b111_001 => {
                let address = self.absolute32(core);
                core.read(address)
            }
            0b111_010 => {
                let address = self.displacement(core, core.pc);
                core.read(address)
            }
            _ => unimplemented!("Address mode read: {:06b}", self.0),
        }
    }

    pub fn write<T: Size>(self, core: &mut Core<impl Bus>, value: T) {
        #[allow(clippy::unusual_byte_groupings)]
        match self.0 {
            0b000_000..=0b000_111 => core.set_dreg(self.reg(), value),
            0b001_000..=0b001_111 => core.set_areg(self.reg(), value),
            0b010_000..=0b010_111 => {
                let address = core.areg(self.reg());
                core.write(address, value);
            }
            0b011_000..=0b011_111 => {
                let index = self.reg();
                let address = core.areg(index);
                core.write(address, value);
                core.set_areg(index, address.wrapping_add(mem::size_of::<T>() as u32));
            }
            0b101_000..=0b101_111 => {
                let address = self.areg_displacement(core);
                core.write(address, value);
            }
            0b111_000 => {
                let address = self.absolute16(core);
                core.write(address, value);
            }
            0b111_001 => {
                let address = self.absolute32(core);
                core.write(address, value);
            }
            0b111_010 => {
                let address = self.displacement(core, core.pc);
                core.write(address, value);
            }
            _ => unimplemented!("Address mode write: {:06b}", self.0),
        }
    }

    pub fn modify<T: Bus, U: Size>(self, core: &mut Core<T>, cb: impl Fn(&mut Core<T>, U) -> U) {
        #[allow(clippy::unusual_byte_groupings)]
        match self.0 {
            0b000_000..=0b000_111 => {
                let index = self.reg();
                let result = cb(core, core.dreg(index));
                core.set_dreg(index, result);
            }
            0b001_000..=0b001_111 => {
                let index = self.reg();
                let result = cb(core, core.areg(index));
                core.set_areg(index, result);
            }
            0b010_000..=0b010_111 => {
                let address = core.areg(self.reg());
                core.modify(address, cb);
            }
            0b101_000..=0b101_111 => {
                let address = self.areg_displacement(core);
                core.modify(address, cb);
            }
            0b111_000 => {
                let address = self.absolute16(core);
                core.modify(address, cb);
            }
            0b111_001 => {
                let address = self.absolute32(core);
                core.modify(address, cb);
            }
            0b111_010 => {
                let address = self.displacement(core, core.pc);
                core.modify(address, cb);
            }
            _ => unimplemented!("Address mode write: {:06b}", self.0),
        }
    }

    pub fn reg(self) -> usize {
        self.0 as usize & 7
    }

    #[allow(clippy::unusual_byte_groupings)]
    pub fn is_post_increment(self) -> bool {
        (self.0 & 0b111_000) == 0b011_000
    }

    #[allow(clippy::unusual_byte_groupings)]
    pub fn is_pre_decrement(self) -> bool {
        (self.0 & 0b111_000) == 0b100_000
    }

    #[allow(clippy::unusual_byte_groupings)]
    pub fn is_immediate(self) -> bool {
        self.0 == 0b111_100
    }

    fn absolute16(self, core: &mut Core<impl Bus>) -> u32 {
        core.next::<u16>() as u32
    }

    fn absolute32(self, core: &mut Core<impl Bus>) -> u32 {
        core.next()
    }

    fn areg_displacement(self, core: &mut Core<impl Bus>) -> u32 {
        let base = core.areg(self.reg());
        self.displacement(core, base)
    }

    fn displacement(self, core: &mut Core<impl Bus>, base: u32) -> u32 {
        let displacement: u16 = core.next();
        base.wrapping_add(displacement as i16 as u32)
    }
}

impl From<u16> for AddressMode {
    fn from(value: u16) -> Self {
        Self(value as u8 & 0x3f)
    }
}

impl fmt::Display for AddressMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.0 >> 3) & 7 {
            0b000 => write!(f, "D{}", self.0 & 7),
            0b001 => write!(f, "A{}", self.0 & 7),
            0b010 => write!(f, "(A{})", self.0 & 7),
            0b011 => write!(f, "(A{})+", self.0 & 7),
            0b100 => write!(f, "-(A{})", self.0 & 7),
            0b101 => write!(f, "u16(A{})", self.0 & 7),
            0b110 => write!(f, "u8(A{}, Xn)", self.0 & 7),
            0b111 => match self.0 & 7 {
                0b000 => write!(f, "u16"),
                0b001 => write!(f, "u32"),
                0b010 => write!(f, "u16(PC)"),
                0b011 => write!(f, "u8(PC, Xn)"),
                0b100 => write!(f, "#imm"),
                _ => panic!("[invalid]"),
            },
            _ => unreachable!(),
        }
    }
}
