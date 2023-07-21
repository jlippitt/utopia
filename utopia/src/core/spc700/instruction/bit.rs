use super::super::{Bus, Core};
use tracing::debug;

fn decode(bit_address: u16) -> (u16, u16) {
    (bit_address & 0x1fff, (bit_address >> 13) & 7)
}

pub fn or1(core: &mut Core<impl Bus>) {
    debug!("OR1 C, m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.idle();
    core.flags.c |= (value & (1 << bit)) != 0;
}

pub fn or1_not(core: &mut Core<impl Bus>) {
    debug!("OR1 C, /m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.idle();
    core.flags.c |= (value & (1 << bit)) == 0;
}

pub fn and1(core: &mut Core<impl Bus>) {
    debug!("AND1 C, m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.flags.c &= (value & (1 << bit)) != 0;
}

pub fn and1_not(core: &mut Core<impl Bus>) {
    debug!("AND1 C, /m.b");
    let (address, bit) = decode(core.next_word());
    let value = core.read(address);
    core.flags.c &= (value & (1 << bit)) == 0;
}
