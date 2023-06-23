use super::super::{Core, Bus, ReadAddress, WriteAddress};
use tracing::debug;

pub fn ld<Lhs: WriteAddress<u8>, Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("LD {}, {}", Lhs::NAME, Rhs::NAME);
    let value = Rhs::read(core);
    Lhs::write(core, value);
}

pub fn ld16<Lhs: WriteAddress<u16>>(core: &mut Core<impl Bus>) {
    debug!("LD {}, nn", Lhs::NAME);
    let value = core.next_word();
    Lhs::write(core, value);
}