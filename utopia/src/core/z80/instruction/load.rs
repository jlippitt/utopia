use super::super::address_mode::{ReadAddress, WriteAddress};
use super::super::{Bus, Core};
use tracing::trace;

pub fn ld<T, Lhs: WriteAddress<T>, Rhs: ReadAddress<T>>(core: &mut Core<impl Bus>) {
    trace!("LD {}, {}", Lhs::NAME, Rhs::NAME);
    let value = Rhs::read(core);
    Lhs::write(core, value);
}

pub fn ld_sp_hl(core: &mut Core<impl Bus>) {
    trace!("LD SP, HL");
    core.idle(2);
    core.sp = core.hl;
}

pub fn pop<Addr: WriteAddress<u16>>(core: &mut Core<impl Bus>) {
    trace!("POP {}", Addr::NAME);
    let value = core.pop();
    Addr::write(core, value);
}

pub fn push<Addr: ReadAddress<u16>>(core: &mut Core<impl Bus>) {
    trace!("PUSH {}", Addr::NAME);
    core.idle(1);
    let value = Addr::read(core);
    core.push(value);
}

pub fn in_n(core: &mut Core<impl Bus>) {
    trace!("IN (n), A");
    let address = u16::from_le_bytes([core.next_byte(), core.a]);
    core.a = core.read_port(address);
}

pub fn out_n(core: &mut Core<impl Bus>) {
    trace!("OUT (n), A");
    let address = u16::from_le_bytes([core.next_byte(), core.a]);
    core.write_port(address, core.a);
}
