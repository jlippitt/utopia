use super::super::{Bus, Core, IrqDisable, STACK_PAGE};
use tracing::debug;

const VEC_RESET: u16 = 0xfffc;

fn jump_to_vector(core: &mut Core<impl Bus>, vector: u16) {
    let low = core.read(vector);
    let high = core.read(vector.wrapping_add(1));
    core.pc = u16::from_le_bytes([low, high]);
    core.flags.i = IrqDisable::Set;
}

pub fn reset(core: &mut Core<impl Bus>) {
    debug!("RESET");
    core.read(core.pc);

    for _ in 0..=2 {
        core.read(STACK_PAGE | (core.s as u16));
        core.s = core.s.wrapping_sub(1);
    }

    jump_to_vector(core, VEC_RESET);
}
