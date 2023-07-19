use super::super::{Bus, Core, IrqDisable};
use tracing::debug;

const NATIVE_NMI_VECTOR: u16 = 0xffea;

const EMULATION_NMI_VECTOR: u16 = 0xfffa;
const EMULATION_RESET_VECTOR: u16 = 0xfffc;

fn push_state<const E: bool>(core: &mut Core<impl Bus>, break_flag: bool) {
    if !E {
        core.push::<E>((core.pc >> 16) as u8);
    }

    core.push::<E>((core.pc >> 8) as u8);
    core.push::<E>(core.pc as u8);
    core.push::<E>(core.flags_to_u8::<E>(break_flag));
}

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
        core.s = (core.s & 0xff00) | (core.s.wrapping_sub(1) & 0xff);
    }

    jump_to_vector(core, EMULATION_RESET_VECTOR);
}

pub fn nmi<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("NMI");
    core.idle();

    let vector = if E {
        EMULATION_NMI_VECTOR
    } else {
        NATIVE_NMI_VECTOR
    };

    push_state::<E>(core, false);
    jump_to_vector(core, vector);
}
