use super::super::{Bus, Core, IrqDisable, STACK_PAGE};
use tracing::debug;

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;

fn push_state(core: &mut Core<impl Bus>, break_flag: bool) {
    core.push((core.pc >> 8) as u8);
    core.push(core.pc as u8);
    core.push(core.flags_to_u8(break_flag));
}

fn jump_to_vector(core: &mut Core<impl Bus>, vector: u16) {
    let low = core.read(vector);
    let high = core.read(vector.wrapping_add(1));
    core.pc = u16::from_le_bytes([low, high]);
    core.flags.i = IrqDisable::Set;
}

pub fn nmi(core: &mut Core<impl Bus>) {
    debug!("NMI");
    core.read(core.pc);
    push_state(core, false);
    jump_to_vector(core, NMI_VECTOR);
}

pub fn reset(core: &mut Core<impl Bus>) {
    debug!("RESET");
    core.read(core.pc);

    for _ in 0..=2 {
        core.read(STACK_PAGE | (core.s as u16));
        core.s = core.s.wrapping_sub(1);
    }

    jump_to_vector(core, RESET_VECTOR);
}
