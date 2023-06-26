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

pub fn daa(core: &mut Core<impl Bus>) {
    debug!("DAA");

    if core.flags.n {
        if core.flags.h {
            core.a = core.a.wrapping_sub(0x06);
        }

        if core.flags.c {
            core.a = core.a.wrapping_sub(0x60);
        }
    } else {
        if core.flags.h || (core.a & 0x0f) > 0x09 {
            let (result, carries) = core.a.overflowing_add(0x06);
            core.a = result;
            core.flags.c |= carries;
        }

        if core.flags.c || (core.a & 0xf0) > 0x90 {
            let (result, carries) = core.a.overflowing_add(0x60);
            core.a = result;
            core.flags.c |= carries;
        }
    }

    core.flags.z = core.a;
    core.flags.h = false;
}
