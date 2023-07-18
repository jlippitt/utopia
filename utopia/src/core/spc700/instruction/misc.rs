use super::super::{Bus, Core};
use tracing::debug;

pub fn nop(core: &mut Core<impl Bus>) {
    debug!("NOP");
    core.read(core.pc);
}

pub fn auto_inc_read(core: &mut Core<impl Bus>) {
    debug!("MOV A, (X)+");
    core.read(core.pc);
    core.a = core.read_direct(core.x);
    core.set_nz(core.a);
    core.x = core.x.wrapping_add(1);
    core.idle();
}

pub fn auto_inc_write(core: &mut Core<impl Bus>) {
    debug!("MOV (X)+, A");
    core.read(core.pc);
    core.idle();
    core.write_direct(core.x, core.a);
    core.x = core.x.wrapping_add(1);
}
