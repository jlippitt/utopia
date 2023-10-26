use super::super::address_mode::{ReadAddress, WriteAddress};
use super::super::operator::{BinaryOperator, BranchOperator, UnaryOperator};
use super::super::{Bus, Core};
use tracing::trace;

pub fn binary<Lhs: WriteAddress, Rhs: ReadAddress, Op: BinaryOperator>(core: &mut Core<impl Bus>) {
    trace!("{} {}, {}", Op::NAME, Lhs::NAME, Rhs::NAME);
    Rhs::init(core);
    let rhs = Rhs::read(core);
    Lhs::modify(core, |core, lhs| Op::apply(core, lhs, rhs));
}

pub fn unary<Addr: WriteAddress, Op: UnaryOperator>(core: &mut Core<impl Bus>) {
    trace!("{} {}", Op::NAME, Addr::NAME);
    Addr::init(core);
    Addr::modify(core, |core, value| Op::apply(core, value));
}

pub fn write<Lhs: WriteAddress, Rhs: ReadAddress>(core: &mut Core<impl Bus>) {
    trace!("MOV {}, {}", Lhs::NAME, Rhs::NAME);
    Lhs::init(core);
    let value = Rhs::read(core);
    Lhs::modify(core, |_core, _value| value);
}

pub fn compare<Lhs: WriteAddress, Rhs: ReadAddress>(core: &mut Core<impl Bus>) {
    trace!("CMP {}, {}", Lhs::NAME, Rhs::NAME);

    Rhs::init(core);
    let rhs = Rhs::read(core);
    let lhs = Lhs::read(core);
    Lhs::finalize(core);

    let (result, borrow) = lhs.overflowing_sub(rhs);
    core.set_nz(result);
    core.flags.c = !borrow;
}

pub fn pop<Addr: WriteAddress>(core: &mut Core<impl Bus>) {
    trace!("POP {}", Addr::NAME);
    core.read(core.pc);
    core.idle();
    Addr::modify(core, |core, _value| core.pop());
}

pub fn push<Addr: ReadAddress>(core: &mut Core<impl Bus>) {
    trace!("PUSH {}", Addr::NAME);
    core.read(core.pc);
    let value = Addr::read(core);
    core.push(value);
    core.idle();
}

pub fn branch<Op: BranchOperator>(core: &mut Core<impl Bus>) {
    trace!("{} r", Op::NAME);
    let condition = Op::apply(core);
    let offset = core.next_byte();

    if condition {
        trace!("Branch taken");
        core.pc = core.pc.wrapping_add(offset as i8 as i16 as u16);
        core.idle();
        core.idle();
    } else {
        trace!("Branch not taken");
    }
}
