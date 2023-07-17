use super::super::{Bus, Core};
use tracing::debug;

pub fn movw_read(core: &mut Core<impl Bus>) {
    debug!("MOVW YA, d");
    let low_address = core.next_byte();
    core.a = core.read_direct(low_address);
    core.idle();
    let high_address = low_address.wrapping_add(1);
    core.y = core.read_direct(high_address);
    core.flags.n = core.y;
    core.flags.z = core.y | core.a;
}

pub fn movw_write(core: &mut Core<impl Bus>) {
    debug!("MOVW d, YA");
    let low_address = core.next_byte();
    core.read_direct(low_address);
    core.write_direct(low_address, core.a);
    let high_address = low_address.wrapping_add(1);
    core.write_direct(high_address, core.y);
}
