use super::{Bus, Core, Size};
use std::fmt;

#[derive(Copy, Clone)]
pub struct AddressMode(u8);

impl AddressMode {
    pub fn read<T: Size>(self, core: &mut Core<impl Bus>) -> T {
        match (self.0 >> 3) & 7 {
            0b111 => match self.0 & 7 {
                0b000 => {
                    let address = self.absolute16(core);
                    core.read(address)
                }
                0b001 => {
                    let address = self.absolute32(core);
                    core.read(address)
                }
                _ => unimplemented!("Address mode read: {:06b}", self.0),
            },
            _ => unimplemented!("Address mode read: {:06b}", self.0),
        }
    }

    fn absolute16(self, core: &mut Core<impl Bus>) -> u32 {
        core.next::<u16>() as u32
    }

    fn absolute32(self, core: &mut Core<impl Bus>) -> u32 {
        core.next()
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
