use super::super::{Bus, Core};
use tracing::{trace, warn};

pub fn nop(core: &mut Core<impl Bus>) {
    trace!("NOP");
    core.read(core.pc);
}

pub fn mov_direct_direct(core: &mut Core<impl Bus>) {
    trace!("MOV d, d");
    let src_address = core.next_byte();
    let value = core.read_direct(src_address);
    let dst_address = core.next_byte();
    core.write_direct(dst_address, value);
}

pub fn auto_inc_read(core: &mut Core<impl Bus>) {
    trace!("MOV A, (X)+");
    core.read(core.pc);
    core.a = core.read_direct(core.x);
    core.set_nz(core.a);
    core.x = core.x.wrapping_add(1);
    core.idle();
}

pub fn auto_inc_write(core: &mut Core<impl Bus>) {
    trace!("MOV (X)+, A");
    core.read(core.pc);
    core.idle();
    core.write_direct(core.x, core.a);
    core.x = core.x.wrapping_add(1);
}

pub fn xcn(core: &mut Core<impl Bus>) {
    trace!("XCN A");
    core.read(core.pc);
    core.idle();
    core.idle();
    core.idle();
    core.a = (core.a << 4) | (core.a >> 4);
    core.set_nz(core.a);
}

pub fn sleep(core: &mut Core<impl Bus>) {
    // Because there are no interrupts, this is functionally the same as STOP
    trace!("SLEEP");
    core.read(core.pc);
    core.idle();
    core.stopped = true;
    warn!("SPC700 stopped");
}

pub fn stop(core: &mut Core<impl Bus>) {
    trace!("STOP");
    core.read(core.pc);
    core.idle();
    core.stopped = true;
    warn!("SPC700 stopped");
}
