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

pub fn phx(core: &mut Core<impl Bus>) {
    debug!("PHX");
    core.read(core.pc);
    core.poll();
    core.push(core.x);
}

pub fn plx(core: &mut Core<impl Bus>) {
    debug!("PLX");
    core.read(core.pc);
    core.read_physical(STACK_PAGE | (core.s as u32));
    core.poll();
    core.x = core.pull();
    core.set_nz(core.x);
}

pub fn phy(core: &mut Core<impl Bus>) {
    debug!("PHY");
    core.read(core.pc);
    core.poll();
    core.push(core.y);
}

pub fn ply(core: &mut Core<impl Bus>) {
    debug!("PLY");
    core.read(core.pc);
    core.read_physical(STACK_PAGE | (core.s as u32));
    core.poll();
    core.y = core.pull();
    core.set_nz(core.y);
}
