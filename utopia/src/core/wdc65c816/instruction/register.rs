use super::super::{Bus, Core, EMULATION_STACK_PAGE};
use tracing::debug;

pub fn tcd(core: &mut Core<impl Bus>) {
    debug!("TCD");
    core.poll();
    core.idle();
    core.d = core.a;
    core.set_nz16(core.d);
}

pub fn tdc(core: &mut Core<impl Bus>) {
    debug!("TDC");
    core.poll();
    core.idle();
    core.a = core.d;
    core.set_nz16(core.a);
}

pub fn tcs<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("TCS");
    core.poll();
    core.idle();

    if E {
        core.s = EMULATION_STACK_PAGE | (core.a & 0xff);
    } else {
        core.s = core.a;
    }
}

pub fn tsc(core: &mut Core<impl Bus>) {
    debug!("TSC");
    core.poll();
    core.idle();
    core.a = core.s;
    core.set_nz16(core.a);
}
