use super::super::{Bus, Core, IrqDisable, EMULATION_STACK_PAGE};
use tracing::debug;

const EMULATION_RESET_VECTOR: u16 = 0xfffc;

fn jump_to_vector(core: &mut Core<impl Bus>, vector: u16) {
    let low = core.read(vector as u32);
    let high = core.read(vector.wrapping_add(1) as u32);
    core.pc = u32::from_le_bytes([low, high, 0, 0]);
    core.flags.d = false;
    core.flags.i = IrqDisable::Set;
}

pub fn reset(core: &mut Core<impl Bus>) {
    debug!("RESET");
    core.idle();

    for _ in 0..=2 {
        core.read(core.s as u32);
        core.s = EMULATION_STACK_PAGE | (core.s.wrapping_sub(1) & 0xff);
    }

    jump_to_vector(core, EMULATION_RESET_VECTOR);
}
