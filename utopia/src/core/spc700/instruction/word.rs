use super::super::{Bus, Core};
use tracing::debug;

pub fn decw(core: &mut Core<impl Bus>) {
    debug!("DECW d");

    let low_address = core.next_byte();
    let low_value = core.read_direct(low_address);
    let (low_result, borrow) = low_value.overflowing_sub(1);
    core.write_direct(low_address, low_result);

    let high_address = low_address.wrapping_add(1);
    let high_value = core.read_direct(high_address);
    let high_result = high_value.wrapping_sub(borrow as u8);
    core.write_direct(high_address, high_result);

    core.flags.n = high_result;
    core.flags.z = high_result | low_result;
}

pub fn incw(core: &mut Core<impl Bus>) {
    debug!("INCW d");

    let low_address = core.next_byte();
    let low_value = core.read_direct(low_address);
    let (low_result, carry) = low_value.overflowing_add(1);
    core.write_direct(low_address, low_result);

    let high_address = low_address.wrapping_add(1);
    let high_value = core.read_direct(high_address);
    let high_result = high_value.wrapping_add(carry as u8);
    core.write_direct(high_address, high_result);

    core.flags.n = high_result;
    core.flags.z = high_result | low_result;
}

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
