use super::{Bus, Core};
use crate::util::memory::Value;
use tracing::trace;

pub trait Size: Value {
    const NAME: char;
    fn set_areg(core: &mut Core<impl Bus>, index: usize, value: Self);
    fn read(core: &Core<impl Bus>, address: u32) -> Self;
}

impl Size for u8 {
    const NAME: char = 'B';

    fn set_areg(core: &mut Core<impl Bus>, index: usize, value: Self) {
        core.areg[index] = value as u32;
        trace!("  A{}: {:02X}", index, value);
    }

    fn read(core: &Core<impl Bus>, address: u32) -> Self {
        let address = address & 0x00ff_ffff;
        let value = core.bus.read(address);
        trace!("  {:06X} => {:02X}", address, value);
        value
    }
}

impl Size for u16 {
    const NAME: char = 'W';

    fn set_areg(core: &mut Core<impl Bus>, index: usize, value: Self) {
        core.areg[index] = value as u32;
        trace!("  A{}: {:04X}", index, value);
    }

    fn read(core: &Core<impl Bus>, address: u32) -> Self {
        let address = address & 0x00ff_ffff;
        let value = core.bus.read(address);
        trace!("  {:06X} => {:04X}", address, value);
        value
    }
}

impl Size for u32 {
    const NAME: char = 'L';

    fn set_areg(core: &mut Core<impl Bus>, index: usize, value: Self) {
        core.areg[index] = value;
        trace!("  A{}: {:06X}", index, value);
    }

    fn read(core: &Core<impl Bus>, address: u32) -> Self {
        let high = u16::read(core, address);
        let low = u16::read(core, address.wrapping_add(2));
        ((high as u32) << 16) | low as u32
    }
}
