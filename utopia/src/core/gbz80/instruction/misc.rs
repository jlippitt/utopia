use super::super::{Bus, Core};
use tracing::debug;

pub fn nop(_core: &mut Core<impl Bus>) {
    debug!("NOP");
}

pub fn di(core: &mut Core<impl Bus>) {
    debug!("DI");
    core.ime = false;
    core.ime_delayed = false;
}

pub fn ei(core: &mut Core<impl Bus>) {
    debug!("EI");
    core.ime_delayed = true;
}

pub fn scf(core: &mut Core<impl Bus>) {
    debug!("SCF");
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = true;
}

pub fn ccf(core: &mut Core<impl Bus>) {
    debug!("CCF");
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = !core.flags.c;
}

pub fn cpl(core: &mut Core<impl Bus>) {
    debug!("CPL");
    core.a = !core.a;
    core.flags.n = true;
    core.flags.h = true;
}
