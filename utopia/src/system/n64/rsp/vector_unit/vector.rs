use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct Vector(pub [u16; 8]);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Broadcast {
    None,
    Quarter(usize),
    Half(usize),
    Single(usize),
}

impl Vector {
    pub fn u16(&self, elem: usize) -> u16 {
        debug_assert!((elem & 1) == 0);
        self.0[elem >> 1]
    }

    pub fn u64(&self, elem: usize) -> u64 {
        debug_assert!((elem & 7) == 0);
        let mut value = 0;
        value |= (self.0[elem >> 1] as u64) << 48;
        value |= (self.0[(elem >> 1) + 1] as u64) << 32;
        value |= (self.0[(elem >> 1) + 2] as u64) << 16;
        value |= self.0[(elem >> 1) + 3] as u64;
        value
    }

    pub fn set_u16(&mut self, elem: usize, value: u16) {
        debug_assert!((elem & 1) == 0);
        self.0[elem >> 1] = value;
    }

    pub fn set_u64(&mut self, elem: usize, value: u64) {
        debug_assert!((elem & 7) == 0);
        self.0[elem >> 1] = (value >> 48) as u16;
        self.0[(elem >> 1) + 1] = (value >> 32) as u16;
        self.0[(elem >> 1) + 2] = (value >> 16) as u16;
        self.0[(elem >> 1) + 3] = value as u16;
    }

    pub fn set_u128(&mut self, elem: usize, value: u128) {
        debug_assert!((elem & 15) == 0);
        self.0[elem >> 1] = (value >> 112) as u16;
        self.0[(elem >> 1) + 1] = (value >> 96) as u16;
        self.0[(elem >> 1) + 2] = (value >> 80) as u16;
        self.0[(elem >> 1) + 3] = (value >> 64) as u16;
        self.0[(elem >> 1) + 4] = (value >> 48) as u16;
        self.0[(elem >> 1) + 5] = (value >> 32) as u16;
        self.0[(elem >> 1) + 6] = (value >> 16) as u16;
        self.0[(elem >> 1) + 7] = value as u16;
    }

    pub fn broadcast(self, broadcast: Broadcast) -> Self {
        match broadcast {
            Broadcast::None => self,
            Broadcast::Quarter(index) => Vector([
                self.0[index],
                self.0[index],
                self.0[index + 2],
                self.0[index + 2],
                self.0[index + 4],
                self.0[index + 4],
                self.0[index + 6],
                self.0[index + 6],
            ]),
            Broadcast::Half(index) => Vector([
                self.0[index],
                self.0[index],
                self.0[index],
                self.0[index],
                self.0[index + 4],
                self.0[index + 4],
                self.0[index + 4],
                self.0[index + 4],
            ]),
            Broadcast::Single(index) => Vector([self.0[index]; 8]),
        }
    }
}

impl Index<usize> for Vector {
    type Output = u16;

    fn index(&self, index: usize) -> &u16 {
        &self.0[index]
    }
}

impl IndexMut<usize> for Vector {
    fn index_mut(&mut self, index: usize) -> &mut u16 {
        &mut self.0[index]
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7],
        )
    }
}

impl From<usize> for Broadcast {
    fn from(value: usize) -> Self {
        match value & 15 {
            0..=1 => Self::None,
            2..=3 => Self::Quarter(value & 1),
            4..=7 => Self::Half(value & 3),
            8..=15 => Self::Single(value & 7),
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Broadcast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::Quarter(index) => write!(f, ",E({}q)", index),
            Self::Half(index) => write!(f, ",E({}h)", index),
            Self::Single(index) => write!(f, ",E({})", index),
        }
    }
}
