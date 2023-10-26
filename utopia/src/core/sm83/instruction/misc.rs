use super::super::{Bus, Core};
use tracing::trace;

pub fn nop(_core: &mut Core<impl Bus>) {
    trace!("NOP");
}

pub fn halt(core: &mut Core<impl Bus>) {
    trace!("HALT");
    core.halted = true;
}

pub fn stop(core: &mut Core<impl Bus>) {
    trace!("STOP");
    core.bus.stop();
}

pub fn di(core: &mut Core<impl Bus>) {
    trace!("DI");
    core.ime = false;
    core.ime_delayed = false;
}

pub fn ei(core: &mut Core<impl Bus>) {
    trace!("EI");
    core.ime_delayed = true;
}

pub fn scf(core: &mut Core<impl Bus>) {
    trace!("SCF");
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = true;
}

pub fn ccf(core: &mut Core<impl Bus>) {
    trace!("CCF");
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = !core.flags.c;
}

pub fn cpl(core: &mut Core<impl Bus>) {
    trace!("CPL");
    core.a = !core.a;
    core.flags.n = true;
    core.flags.h = true;
}

pub fn daa(core: &mut Core<impl Bus>) {
    trace!("DAA");

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
