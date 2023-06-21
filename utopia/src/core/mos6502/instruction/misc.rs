use super::super::{Bus, Core, STACK_PAGE};
use tracing::debug;

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