use super::super::address_mode::{ReadAddress, WriteAddress};
use super::super::operator::{BinaryOperator, UnaryOperator};
use super::super::{Bus, Core};
use tracing::debug;

pub fn write<Lhs: WriteAddress, Rhs: ReadAddress>(core: &mut Core<impl Bus>) {
    debug!("MOV {}, {}", Lhs::NAME, Rhs::NAME);
    let value = Rhs::read(core);
    Lhs::modify(core, |_core, _value| value);
}

pub fn binary<Lhs: WriteAddress, Rhs: ReadAddress, Op: BinaryOperator>(core: &mut Core<impl Bus>) {
    debug!("{} {}, {}", Op::NAME, Lhs::NAME, Rhs::NAME);
    let rhs = Rhs::read(core);
    Lhs::modify(core, |core, lhs| Op::apply(core, lhs, rhs));
}

pub fn unary<Addr: WriteAddress, Op: UnaryOperator>(core: &mut Core<impl Bus>) {
    debug!("{} {}", Op::NAME, Addr::NAME);
    Addr::modify(core, |core, value| Op::apply(core, value));
}
