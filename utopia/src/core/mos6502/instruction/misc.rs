use super::super::{Bus, Core, STACK_PAGE};
use tracing::trace;

pub fn php(core: &mut Core<impl Bus>) {
    trace!("PHP");
    core.read(core.pc);
    core.poll();
    core.push(core.flags_to_u8(true));
}

pub fn plp(core: &mut Core<impl Bus>) {
    trace!("PLP");
    core.read(core.pc);
    core.read(STACK_PAGE | (core.s as u16));
    core.poll();
    let flags = core.pull();
    core.flags_from_u8(flags);
}

pub fn pha(core: &mut Core<impl Bus>) {
    trace!("PHA");
    core.read(core.pc);
    core.poll();
    core.push(core.a);
}

pub fn pla(core: &mut Core<impl Bus>) {
    trace!("PLA");
    core.read(core.pc);
    core.read(STACK_PAGE | (core.s as u16));
    core.poll();
    core.a = core.pull();
    core.set_nz(core.a);
}

pub fn nop(core: &mut Core<impl Bus>) {
    trace!("NOP");
    core.read(core.pc);
}
