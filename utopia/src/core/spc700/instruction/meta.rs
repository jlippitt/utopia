use super::super::address_mode::{ReadAddress, WriteAddress};
use super::super::operator::ReadOperator;
use super::super::{Bus, Core};
use tracing::debug;

pub fn read<Lhs: WriteAddress, Rhs: ReadAddress, Op: ReadOperator>(core: &mut Core<impl Bus>) {
    debug!("MOV {}, {}", Lhs::NAME, Rhs::NAME);
    let value = Rhs::read(core);
    let result = Op::apply(core, value);
    Lhs::write(core, result);
}
