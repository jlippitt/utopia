use super::{Bus, Core};
use crate::util::memory::Value;
use tracing::trace;

pub trait Size: Value {
    fn set_areg(core: &mut Core<impl Bus>, index: usize, value: Self);
    fn read(core: &Core<impl Bus>, address: u32) -> Self;
}

impl Size for u16 {
    fn set_areg(core: &mut Core<impl Bus>, index: usize, value: Self) {
        core.areg[index] = value as u32;
        trace!("  A{}: {:04X}", index, value);
    }

    fn read(core: &Core<impl Bus>, address: u32) -> Self {
        let value = core.bus.read(address);
        trace!("  {:08X} => {:04X}", address, value);
        value
    }
}

impl Size for u32 {
    fn set_areg(core: &mut Core<impl Bus>, index: usize, value: Self) {
        core.areg[index] = value;
        trace!("  A{}: {:08X}", index, value);
    }

    fn read(core: &Core<impl Bus>, address: u32) -> Self {
        let high = u16::read(core, address);
        let low = u16::read(core, address.wrapping_add(2));
        ((high as u32) << 16) | low as u32
    }
}
