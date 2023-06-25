use super::super::{Bus, Core, ReadAddress, WriteAddress};
use tracing::debug;

pub fn ld<Lhs: WriteAddress<u8>, Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("LD {}, {}", Lhs::NAME, Rhs::NAME);
    let value = Rhs::read(core);
    Lhs::write(core, value);
}

pub fn ld16<Lhs: WriteAddress<u16>>(core: &mut Core<impl Bus>) {
    debug!("LD {}, u16", Lhs::NAME);
    let value = core.next_word();
    Lhs::write(core, value);
}

pub fn ld_u16_sp(core: &mut Core<impl Bus>) {
    debug!("LD (u16), SP");
    let address = core.next_word();
    core.write(address, core.sp as u8);
    core.write(address.wrapping_add(1), (core.sp >> 8) as u8);
}
pub fn pop<Addr: WriteAddress<u16>>(core: &mut Core<impl Bus>) {
    debug!("POP {}", Addr::NAME);
    let value = core.pop();
    Addr::write(core, value);
}

pub fn push<Addr: ReadAddress<u16>>(core: &mut Core<impl Bus>) {
    debug!("PUSH {}", Addr::NAME);
    core.idle();
    let value = Addr::read(core);
    core.push(value);
}
