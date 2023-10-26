use super::super::{Bus, Core, STACK_PAGE};
use tracing::trace;

pub fn clrp(core: &mut Core<impl Bus>) {
    trace!("CLRP");
    core.read(core.pc);
    core.flags.p = 0;
}

pub fn setp(core: &mut Core<impl Bus>) {
    trace!("SETP");
    core.read(core.pc);
    core.flags.p = STACK_PAGE;
}

pub fn clrc(core: &mut Core<impl Bus>) {
    trace!("CLRC");
    core.read(core.pc);
    core.flags.c = false;
}

pub fn setc(core: &mut Core<impl Bus>) {
    trace!("SETC");
    core.read(core.pc);
    core.flags.c = true;
}

pub fn ei(core: &mut Core<impl Bus>) {
    trace!("EI");
    core.read(core.pc);
    core.flags.i = true;
}

pub fn di(core: &mut Core<impl Bus>) {
    trace!("DI");
    core.read(core.pc);
    core.flags.i = false;
}

pub fn clrv(core: &mut Core<impl Bus>) {
    trace!("CLRV");
    core.read(core.pc);
    core.flags.v = false;
    core.flags.h = false;
}

pub fn notc(core: &mut Core<impl Bus>) {
    trace!("NOTC");
    core.read(core.pc);
    core.idle();
    core.flags.c = !core.flags.c;
}
