use super::{Bus, Core, Size};
use std::fmt;

#[derive(Copy, Clone)]
pub struct AddressMode(u8);

impl AddressMode {
    pub fn address(self, core: &mut Core<impl Bus>) -> u32 {
        #[allow(clippy::unusual_byte_groupings)]
        match self.0 {
            0b010_000..=0b100_111 => core.areg(self.reg()),
            0b111_000 => self.absolute16(core),
            0b111_001 => self.absolute32(core),
            0b111_010 => self.pc_displacement(core),
            _ => unimplemented!("Address mode lookup: {:06b}", self.0),
        }
    }

    pub fn read<T: Size>(self, core: &mut Core<impl Bus>) -> T {
        #[allow(clippy::unusual_byte_groupings)]
        match self.0 {
            0b111_000 => {
                let address = self.absolute16(core);
                core.read(address)
            }
            0b111_001 => {
                let address = self.absolute32(core);
                core.read(address)
            }
            0b111_010 => {
                let address = self.pc_displacement(core);
                core.read(address)
            }
            _ => unimplemented!("Address mode read: {:06b}", self.0),
        }
    }

    #[allow(clippy::unusual_byte_groupings)]
    pub fn is_post_increment(self) -> bool {
        (self.0 & 0b111_000) == 0b011_000
    }

    #[allow(clippy::unusual_byte_groupings)]
    pub fn is_pre_decrement(self) -> bool {
        (self.0 & 0b111_000) == 0b100_000
    }

    pub fn reg(self) -> usize {
        self.0 as usize & 7
    }

    fn absolute16(self, core: &mut Core<impl Bus>) -> u32 {
        core.next::<u16>() as u32
    }

    fn absolute32(self, core: &mut Core<impl Bus>) -> u32 {
        core.next()
    }

    fn pc_displacement(self, core: &mut Core<impl Bus>) -> u32 {
        let pc = core.pc;
        let displacement: u16 = core.next();
        pc.wrapping_add(displacement as u32)
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
                0b100 => write!(f, "#const"),
                _ => panic!("[invalid]"),
            },
            _ => unreachable!(),
        }
    }
}
