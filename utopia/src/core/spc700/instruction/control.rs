use super::super::operator::BranchOperator;
use super::super::{Bus, Core};
use tracing::debug;

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

pub fn branch<Op: BranchOperator>(core: &mut Core<impl Bus>) {
    debug!("{} r", Op::NAME);
    let offset = core.next_byte();

    if Op::apply(&core.flags) {
        debug!("Branch taken");
        core.pc = core.pc.wrapping_add(offset as i8 as i16 as u16);
        core.idle();
        core.idle();
    } else {
        debug!("Branch not taken");
    }
}
