use super::super::{Bus, Core, IrqDisable, STACK_PAGE};
use tracing::debug;

const IRQ2_VECTOR: u16 = 0xfff6;
const IRQ1_VECTOR: u16 = 0xfff8;
const TIMER_VECTOR: u16 = 0xfffa;
const NMI_VECTOR: u16 = 0xfffc;
const RESET_VECTOR: u16 = 0xfffe;

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

pub fn timer(core: &mut Core<impl Bus>) {
    debug!("TIMER");
    core.read(core.pc);
    push_state(core, false);
    jump_to_vector(core, TIMER_VECTOR);
}

pub fn irq1(core: &mut Core<impl Bus>) {
    debug!("IRQ1");
    core.read(core.pc);
    push_state(core, false);
    jump_to_vector(core, IRQ1_VECTOR);
}

pub fn irq2(core: &mut Core<impl Bus>) {
    debug!("IRQ2");
    core.read(core.pc);
    push_state(core, false);
    jump_to_vector(core, IRQ2_VECTOR);
}

pub fn reset(core: &mut Core<impl Bus>) {
    debug!("RESET");
    core.read(core.pc);

    for _ in 0..=2 {
        core.read_physical(STACK_PAGE | (core.s as u32));
        core.s = core.s.wrapping_sub(1);
    }

    jump_to_vector(core, RESET_VECTOR);
}

pub fn brk(core: &mut Core<impl Bus>) {
    debug!("BRK #const");
    core.next_byte();
    push_state(core, true);
    jump_to_vector(core, IRQ2_VECTOR);
}
