use super::super::{Bus, Core, STACK_PAGE};
use tracing::debug;

pub fn clrp(core: &mut Core<impl Bus>) {
    debug!("CLRP");
    core.read(core.pc);
    core.flags.p = 0;
}

pub fn setp(core: &mut Core<impl Bus>) {
    debug!("SETP");
    core.read(core.pc);
    core.flags.p = STACK_PAGE;
}

pub fn clrc(core: &mut Core<impl Bus>) {
    debug!("CLRC");
    core.read(core.pc);
    core.flags.c = false;
}

pub fn setc(core: &mut Core<impl Bus>) {
    debug!("SETC");
    core.read(core.pc);
    core.flags.c = true;
}

pub fn ei(core: &mut Core<impl Bus>) {
    debug!("EI");
    core.read(core.pc);
    core.flags.i = true;
}

pub fn di(core: &mut Core<impl Bus>) {
    debug!("DI");
    core.read(core.pc);
    core.flags.i = false;
}

pub fn clrv(core: &mut Core<impl Bus>) {
    debug!("CLRV");
    core.read(core.pc);
    core.flags.v = 0;
    core.flags.h = 0;
}

pub fn notc(core: &mut Core<impl Bus>) {
    debug!("NOTC");
    core.read(core.pc);
    core.idle();
    core.flags.c = !core.flags.c;
}
