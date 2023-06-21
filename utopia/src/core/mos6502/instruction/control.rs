use super::super::{Bus, Core, STACK_PAGE};
use tracing::debug;

pub fn jmp(core: &mut Core<impl Bus>) {
    debug!("JMP addr");
    let low = core.next_byte();
    core.poll();
    let high = core.next_byte();
    core.pc = u16::from_le_bytes([low, high]);
}

pub fn jmp_indirect(core: &mut Core<impl Bus>) {
    debug!("JMP (addr)");
    let address = core.next_word();
    let low = core.read(address);
    core.poll();
    // 6502 JMP bug:
    let high = core.read((address & 0xff00) | (address.wrapping_add(1) & 0xff));
    core.pc = u16::from_le_bytes([low, high]);
}

pub fn jsr(core: &mut Core<impl Bus>) {
    debug!("JSR addr");
    let low = core.next_byte();
    core.read(STACK_PAGE | (core.s as u16));
    core.push((core.pc >> 8) as u8);
    core.push(core.pc as u8);
    core.poll();
    let high = core.next_byte();
    core.pc = u16::from_le_bytes([low, high]);
}

pub fn rts(core: &mut Core<impl Bus>) {
    debug!("RTS");
    core.read(core.pc);
    core.read(STACK_PAGE | (core.s as u16));
    let low = core.pull();
    let high = core.pull();
    core.pc = u16::from_le_bytes([low, high]);
    core.poll();
    core.read(core.pc);
    core.pc = core.pc.wrapping_add(1);
}

pub fn rti(core: &mut Core<impl Bus>) {
    debug!("RTI");
    core.read(core.pc);
    core.read(STACK_PAGE | (core.s as u16));
    let flags = core.pull();
    core.flags_from_u8(flags);
    let low = core.pull();
    core.poll();
    let high = core.pull();
    core.pc = u16::from_le_bytes([low, high]);
}

