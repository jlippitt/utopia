use super::super::{Bus, Core, STACK_PAGE};
use tracing::debug;

pub fn plp(core: &mut Core<impl Bus>) {
    debug!("PLP");
    core.read(core.pc);
    core.read(STACK_PAGE | (core.s as u16));
    core.poll();
    let flags = core.pull();
    core.flags_from_u8(flags);
}

pub fn pha(core: &mut Core<impl Bus>) {
    debug!("PHA");
    core.read(core.pc);
    core.poll();
    core.push(core.a);
}

pub fn pla(core: &mut Core<impl Bus>) {
    debug!("PLA");
    core.read(core.pc);
    core.read(STACK_PAGE | (core.s as u16));
    core.poll();
    core.a = core.pull();
    core.set_nz(core.a);
}