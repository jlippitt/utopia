use super::super::{Bus, Core};
use tracing::debug;

pub fn jsr<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("JSR addr");
    let low = core.next_byte();
    let high = core.read(core.pc);
    let target = u16::from_le_bytes([low, high]);
    core.idle();
    core.push::<E>((core.pc >> 8) as u8);
    core.poll();
    core.push::<E>(core.pc as u8);
    core.pc = (core.pc & 0xffff0000) | (target as u32);
}

pub fn jsl<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("JSL long");
    let low = core.next_byte();
    let high = core.next_byte();
    core.push::<E>((core.pc >> 16) as u8);
    core.idle();
    let bank = core.read(core.pc);
    core.push::<E>((core.pc >> 8) as u8);
    core.poll();
    core.push::<E>(core.pc as u8);
    core.pc = u32::from_le_bytes([low, high, bank, 0]);
}

pub fn rts<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("RTS");
    core.idle();
    core.idle();
    let low = core.pull::<E>();
    let high = core.pull::<E>();
    let target = u16::from_le_bytes([low, high]).wrapping_add(1);
    core.pc = (core.pc & 0xffff0000) | (target as u32);
    core.poll();
    core.idle();
}
