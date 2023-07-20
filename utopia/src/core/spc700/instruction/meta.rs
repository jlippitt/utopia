use super::super::address_mode::{ReadAddress, WriteAddress};
use super::super::operator::{BinaryOperator, BranchOperator, UnaryOperator};
use super::super::{Bus, Core};
use tracing::debug;

pub fn binary<Lhs: WriteAddress, Rhs: ReadAddress, Op: BinaryOperator>(core: &mut Core<impl Bus>) {
    debug!("{} {}, {}", Op::NAME, Lhs::NAME, Rhs::NAME);
    Rhs::init(core);
    let rhs = Rhs::read(core);
    Lhs::modify(core, |core, lhs| Op::apply(core, lhs, rhs));
}

pub fn unary<Addr: WriteAddress, Op: UnaryOperator>(core: &mut Core<impl Bus>) {
    debug!("{} {}", Op::NAME, Addr::NAME);
    Addr::init(core);
    Addr::modify(core, |core, value| Op::apply(core, value));
}

pub fn write<Lhs: WriteAddress, Rhs: ReadAddress>(core: &mut Core<impl Bus>) {
    debug!("MOV {}, {}", Lhs::NAME, Rhs::NAME);
    Lhs::init(core);
    let value = Rhs::read(core);
    Lhs::modify(core, |_core, _value| value);
}

pub fn compare<Lhs: WriteAddress, Rhs: ReadAddress>(core: &mut Core<impl Bus>) {
    debug!("CMP {}, {}", Lhs::NAME, Rhs::NAME);

    Rhs::init(core);
    let rhs = Rhs::read(core);
    let lhs = Lhs::read(core);
    Lhs::finalize(core);

    let (result, borrow) = lhs.overflowing_sub(rhs);
    core.set_nz(result);
    core.flags.c = !borrow;
}

pub fn pop<Addr: WriteAddress>(core: &mut Core<impl Bus>) {
    debug!("POP {}", Addr::NAME);
    core.read(core.pc);
    core.idle();
    Addr::modify(core, |core, _value| core.pop());
}

pub fn push<Addr: ReadAddress>(core: &mut Core<impl Bus>) {
    debug!("PUSH {}", Addr::NAME);
    core.read(core.pc);
    let value = Addr::read(core);
    core.push(value);
    core.idle();
}

pub fn branch<Op: BranchOperator>(core: &mut Core<impl Bus>) {
    debug!("{} r", Op::NAME);
    let condition = Op::apply(core);
    let offset = core.next_byte();

    if condition {
        debug!("Branch taken");
        core.pc = core.pc.wrapping_add(offset as i8 as i16 as u16);
        core.idle();
        core.idle();
    } else {
        debug!("Branch not taken");
    }
}
