use super::super::{Bus, Core};
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
