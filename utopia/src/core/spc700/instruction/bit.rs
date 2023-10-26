use super::super::{Bus, Core};
use tracing::trace;

fn decode(bit_address: u16) -> (u16, u16) {
    (bit_address & 0x1fff, (bit_address >> 13) & 7)
}

pub fn or1(core: &mut Core<impl Bus>) {
    trace!("OR1 C, m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.idle();
    core.flags.c |= (value & (1 << bit)) != 0;
}

pub fn or1_not(core: &mut Core<impl Bus>) {
    trace!("OR1 C, /m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.idle();
    core.flags.c |= (value & (1 << bit)) == 0;
}

pub fn and1(core: &mut Core<impl Bus>) {
    trace!("AND1 C, m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.flags.c &= (value & (1 << bit)) != 0;
}

pub fn and1_not(core: &mut Core<impl Bus>) {
    trace!("AND1 C, /m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.flags.c &= (value & (1 << bit)) == 0;
}

pub fn eor1(core: &mut Core<impl Bus>) {
    trace!("EOR1 C, m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.idle();
    core.flags.c ^= (value & (1 << bit)) != 0;
}

pub fn mov1_read(core: &mut Core<impl Bus>) {
    trace!("MOV1 C, m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.flags.c = (value & (1 << bit)) != 0;
}

pub fn mov1_write(core: &mut Core<impl Bus>) {
    trace!("MOV1 m.b, C");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.idle();
    let carry = (core.flags.c as u8) << bit;
    core.write(address, value & !(1 << bit) | carry);
}

pub fn not1(core: &mut Core<impl Bus>) {
    trace!("NOT1 m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.write(address, value ^ (1 << bit));
}

pub fn tset1(core: &mut Core<impl Bus>) {
    trace!("TSET1 !a");
    let address = core.next_word();
    let value = core.read(address);
    core.read(address); // This happens a second time?
    let result = value | core.a;
    core.set_nz(core.a.wrapping_sub(value));
    core.write(address, result);
}

pub fn tclr1(core: &mut Core<impl Bus>) {
    trace!("TCLR1 !a");
    let address = core.next_word();
    let value = core.read(address);
    core.read(address); // This happens a second time?
    let result = value & !core.a;
    core.set_nz(core.a.wrapping_sub(value));
    core.write(address, result);
}
