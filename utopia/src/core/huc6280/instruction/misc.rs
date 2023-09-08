use super::super::{Bus, Core, STACK_PAGE};
use tracing::debug;

pub fn php(core: &mut Core<impl Bus>) {
    debug!("PHP");
    core.read(core.pc);
    core.poll();
    core.push(core.flags_to_u8(true));
}

pub fn plp(core: &mut Core<impl Bus>) {
    debug!("PLP");
    core.read(core.pc);
    core.read_physical(STACK_PAGE | (core.s as u32));
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
    core.read_physical(STACK_PAGE | (core.s as u32));
    core.poll();
    core.a = core.pull();
    core.set_nz(core.a);
}

pub fn nop(core: &mut Core<impl Bus>) {
    debug!("NOP");
    core.read(core.pc);
}
