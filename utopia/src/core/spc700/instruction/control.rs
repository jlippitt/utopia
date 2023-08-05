use super::super::{Bus, Core};
use tracing::debug;

pub fn jmp(core: &mut Core<impl Bus>) {
    debug!("JMP !a");
    core.pc = core.next_word();
}

pub fn jmp_x_indirect(core: &mut Core<impl Bus>) {
    debug!("JMP [!a+X]");
    let low_address = core.next_word().wrapping_add(core.x as u16);
    core.idle();
    let low = core.read(low_address);
    let high_address = low_address.wrapping_add(1);
    let high = core.read(high_address);
    core.pc = u16::from_le_bytes([low, high]);
}

pub fn call(core: &mut Core<impl Bus>) {
    debug!("CALL !a");
    let target = core.next_word();
    core.idle();
    core.push((core.pc >> 8) as u8);
    core.push(core.pc as u8);
    core.pc = target;
    core.idle();
    core.idle();
}

pub fn tcall(core: &mut Core<impl Bus>, id: u16) {
    debug!("TCALL {}", id);

    core.read(core.pc);
    core.idle();
    core.push((core.pc >> 8) as u8);
    core.push(core.pc as u8);
    core.idle();

    let vector = 0xffc0 + ((id ^ 15) << 1);
    let low = core.read(vector);
    let high = core.read(vector.wrapping_add(1));
    core.pc = u16::from_le_bytes([low, high]);
}

pub fn ret(core: &mut Core<impl Bus>) {
    debug!("RET");
    core.read(core.pc);
    core.idle();
    let low = core.pop();
    let high = core.pop();
    core.pc = u16::from_le_bytes([low, high]);
}

pub fn reti(core: &mut Core<impl Bus>) {
    debug!("RET");
    core.read(core.pc);
    core.idle();
    let flags = core.pop();
    core.flags_from_u8(flags);
    let low = core.pop();
    let high = core.pop();
    core.pc = u16::from_le_bytes([low, high]);
}
