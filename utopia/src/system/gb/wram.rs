use crate::util::MirrorVec;
use std::ops::{Index, IndexMut};
use tracing::debug;

const WRAM_BANK_SIZE: usize = 4096;

pub struct Wram {
    data: MirrorVec<u8>,
    bank_offset: usize,
    is_cgb: bool,
}

impl Wram {
    pub fn new(is_cgb: bool) -> Self {
        let num_banks = if is_cgb { 8 } else { 2 };

        Self {
            data: MirrorVec::new(WRAM_BANK_SIZE * num_banks),
            bank_offset: WRAM_BANK_SIZE,
            is_cgb,
        }
    }

    pub fn set_bank(&mut self, value: u8) {
        if !self.is_cgb {
            return;
        }

        let mut bank = value & 0x07;

        if bank == 1 {
            bank = 0;
        }

        self.bank_offset = bank as usize * WRAM_BANK_SIZE;
        debug!("WRAM Bank Offset: {}", self.bank_offset);
    }
}

impl Index<usize> for Wram {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.data[if (index & 0x1000) != 0 {
            self.bank_offset + (index & 0x0fff)
        } else {
            index & 0x0fff
        }]
    }
}

impl IndexMut<usize> for Wram {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.data[if (index & 0x1000) != 0 {
            self.bank_offset + (index & 0x0fff)
        } else {
            index & 0x0fff
        }]
    }
}
