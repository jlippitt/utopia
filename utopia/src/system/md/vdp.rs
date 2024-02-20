use crate::util::memory::{Masked, Reader, Writer};
use tracing::warn;

pub struct Vdp {
    //
}

impl Vdp {
    pub fn new() -> Self {
        Self {}
    }
}

impl Reader for Vdp {
    type Value = u16;

    fn read_register(&self, address: u32) -> u16 {
        match address & 0xffff {
            // TODO: VDP Status
            0x0004 | 0x0006 => 0,
            port => unimplemented!("VDP read: {:04X}", port),
        }
    }
}

impl Writer for Vdp {
    fn write_register(&mut self, address: u32, value: Masked<u16>) {
        match address & 0xffff {
            port => warn!("Unmapped VDP write: {:04X} <= {:04X}", port, value.get()),
        }
    }
}
