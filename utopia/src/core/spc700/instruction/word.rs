use super::super::{Bus, Core};
use tracing::debug;

pub fn movw_read(core: &mut Core<impl Bus>) {
    debug!("MOVW YA, d");
    let low_address = core.next_byte();
    core.a = core.read_direct(low_address);
    core.idle();
    let high_address = low_address.wrapping_add(1);
    core.y = core.read_direct(high_address);
}
