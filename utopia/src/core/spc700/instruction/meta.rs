use super::super::address_mode::{ReadAddress, WriteAddress};
use super::super::operator::ReadOperator;
use super::super::{Bus, Core};
use tracing::debug;

pub fn read<Lhs: WriteAddress, Rhs: ReadAddress, Op: ReadOperator>(core: &mut Core<impl Bus>) {
    debug!("MOV {}, {}", Lhs::NAME, Rhs::NAME);
    let value = Rhs::read(core);
    Lhs::modify(core, |core, _value| Op::apply(core, value));
}

pub fn write<Lhs: WriteAddress, Rhs: ReadAddress>(core: &mut Core<impl Bus>) {
    debug!("MOV {}, {}", Lhs::NAME, Rhs::NAME);
    let value = Rhs::read(core);
    Lhs::modify(core, |_core, _value| value);
}
