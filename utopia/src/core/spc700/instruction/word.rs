use super::super::{Bus, Core};
use tracing::trace;

pub fn decw(core: &mut Core<impl Bus>) {
    trace!("DECW d");

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
    trace!("INCW d");

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

pub fn cmpw(core: &mut Core<impl Bus>) {
    trace!("CMPW YA, d");

    let low_address = core.next_byte();
    let low = core.read_direct(low_address);
    let high_address = low_address.wrapping_add(1);
    let high = core.read_direct(high_address);

    let rhs = u16::from_le_bytes([low, high]);
    let lhs = u16::from_le_bytes([core.a, core.y]);
    let (result, borrow) = lhs.overflowing_sub(rhs);

    let [result_low, result_high] = result.to_le_bytes();

    core.flags.n = result_high;
    core.flags.z = result_high | result_low;
    core.flags.c = !borrow;
}

pub fn addw(core: &mut Core<impl Bus>) {
    trace!("ADDW YA, d");

    let low_address = core.next_byte();
    let low = core.read_direct(low_address);
    core.idle();
    let high_address = low_address.wrapping_add(1);
    let high = core.read_direct(high_address);

    let rhs = u16::from_le_bytes([low, high]);
    let lhs = u16::from_le_bytes([core.a, core.y]);
    let result = lhs.wrapping_add(rhs);
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ result);

    core.a = result as u8;
    core.y = (result >> 8) as u8;
    core.flags.n = core.y;
    core.flags.v = (overflow & 0x8000) != 0;
    core.flags.h = (carries & 0x1000) != 0;
    core.flags.z = core.y | core.a;
    core.flags.c = ((carries ^ overflow) & 0x8000) != 0;
}

pub fn subw(core: &mut Core<impl Bus>) {
    trace!("SUBW YA, d");

    let low_address = core.next_byte();
    let low = core.read_direct(low_address);
    core.idle();
    let high_address = low_address.wrapping_add(1);
    let high = core.read_direct(high_address);

    let rhs = u16::from_le_bytes([low, high]);
    let lhs = u16::from_le_bytes([core.a, core.y]);
    let result = lhs.wrapping_sub(rhs);
    let carries = lhs ^ !rhs ^ result;
    let overflow = (lhs ^ result) & (lhs ^ rhs);

    core.a = result as u8;
    core.y = (result >> 8) as u8;
    core.flags.n = core.y;
    core.flags.v = (overflow & 0x8000) != 0;
    core.flags.h = (carries & 0x1000) != 0;
    core.flags.z = core.y | core.a;
    core.flags.c = ((carries ^ overflow) & 0x8000) != 0;
}

pub fn movw_read(core: &mut Core<impl Bus>) {
    trace!("MOVW YA, d");
    let low_address = core.next_byte();
    core.a = core.read_direct(low_address);
    core.idle();
    let high_address = low_address.wrapping_add(1);
    core.y = core.read_direct(high_address);
    core.flags.n = core.y;
    core.flags.z = core.y | core.a;
}

pub fn movw_write(core: &mut Core<impl Bus>) {
    trace!("MOVW d, YA");
    let low_address = core.next_byte();
    core.read_direct(low_address);
    core.write_direct(low_address, core.a);
    let high_address = low_address.wrapping_add(1);
    core.write_direct(high_address, core.y);
}
