use super::{Bus, Core};
use crate::util::memory::Value;
use tracing::trace;

pub trait Size: Value {
    const NAME: char;
    fn dreg(core: &Core<impl Bus>, index: usize) -> Self;
    fn set_dreg(core: &mut Core<impl Bus>, index: usize, value: Self);
    fn areg(core: &Core<impl Bus>, index: usize) -> Self;
    fn set_areg(core: &mut Core<impl Bus>, index: usize, value: Self);
    fn read(core: &Core<impl Bus>, address: u32) -> Self;
    fn write(core: &mut Core<impl Bus>, address: u32, value: Self);
    fn next(core: &mut Core<impl Bus>) -> Self;
}

impl Size for u8 {
    const NAME: char = 'B';

    fn dreg(core: &Core<impl Bus>, index: usize) -> Self {
        core.dreg[index] as Self
    }

    fn set_dreg(core: &mut Core<impl Bus>, index: usize, value: Self) {
        core.dreg[index] = value as u32;
        trace!("  D{}: {:02X}", index, value);
    }

    fn areg(core: &Core<impl Bus>, index: usize) -> Self {
        core.areg[index] as Self
    }

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

    fn write(core: &mut Core<impl Bus>, address: u32, value: Self) {
        let address = address & 0x00ff_ffff;
        trace!("  {:06X} <= {:02X}", address, value);
        core.bus.write(address, value);
    }

    fn next(core: &mut Core<impl Bus>) -> Self {
        u16::next(core) as Self
    }
}

impl Size for u16 {
    const NAME: char = 'W';

    fn dreg(core: &Core<impl Bus>, index: usize) -> Self {
        core.dreg[index] as Self
    }

    fn set_dreg(core: &mut Core<impl Bus>, index: usize, value: Self) {
        core.dreg[index] = value as u32;
        trace!("  D{}: {:04X}", index, value);
    }

    fn areg(core: &Core<impl Bus>, index: usize) -> Self {
        core.areg[index] as Self
    }

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

    fn write(core: &mut Core<impl Bus>, address: u32, value: Self) {
        let address = address & 0x00ff_ffff;
        trace!("  {:06X} <= {:04X}", address, value);
        core.bus.write(address, value);
    }

    fn next(core: &mut Core<impl Bus>) -> Self {
        let value = Self::read(core, core.pc);
        core.pc = core.pc.wrapping_add(2);
        value
    }
}

impl Size for u32 {
    const NAME: char = 'L';

    fn dreg(core: &Core<impl Bus>, index: usize) -> Self {
        core.dreg[index] as Self
    }

    fn set_dreg(core: &mut Core<impl Bus>, index: usize, value: Self) {
        core.dreg[index] = value;
        trace!("  D{}: {:08X}", index, value);
    }

    fn areg(core: &Core<impl Bus>, index: usize) -> Self {
        core.areg[index] as Self
    }

    fn set_areg(core: &mut Core<impl Bus>, index: usize, value: Self) {
        core.areg[index] = value;
        trace!("  A{}: {:08X}", index, value);
    }

    fn read(core: &Core<impl Bus>, address: u32) -> Self {
        let high = u16::read(core, address);
        let low = u16::read(core, address.wrapping_add(2));
        ((high as u32) << 16) | low as u32
    }

    fn write(core: &mut Core<impl Bus>, address: u32, value: Self) {
        let high = (value >> 16) as u16;
        let low = value as u16;
        u16::write(core, address, high);
        u16::write(core, address.wrapping_add(2), low);
    }

    fn next(core: &mut Core<impl Bus>) -> Self {
        let high = u16::next(core);
        let low = u16::next(core);
        ((high as u32) << 16) | low as u32
    }
}
