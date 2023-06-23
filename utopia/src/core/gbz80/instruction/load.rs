use super::super::{Core, Bus, WriteAddress};
use tracing::debug;

pub fn ld16<Lhs: WriteAddress<u16>>(core: &mut Core<impl Bus>) {
    debug!("LD {}, nn", Lhs::NAME);
    let rhs = core.next_word();
    Lhs::write(core, rhs);
}