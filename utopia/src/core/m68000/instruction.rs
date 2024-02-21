pub use bit::*;
pub use control::*;
pub use meta::*;
pub use transfer::*;

use super::condition::Condition;
use super::operator::Operator;
use super::{Bus, Core, Mode, Size};
use address_mode::AddressMode;
use tracing::trace;

mod address_mode;
mod bit;
mod control;
mod meta;
mod transfer;

pub fn reset(core: &mut Core<impl Bus>) {
    trace!("RESET");
    core.set_mode(Mode::Supervisor);
    core.set_areg::<u32>(7, core.read(0x0000_0000));
    core.set_pc(core.read(0x0000_0004));
    core.set_int_level(7);
}

pub fn tst<T: Size>(core: &mut Core<impl Bus>, word: u16) {
    let operand = AddressMode::from(word);
    trace!("TST.{} {}", T::NAME, operand);
    let value: T = operand.read(core);
    core.set_ccr(|flags| {
        flags.set_nz(value);
        flags.v = false;
        flags.c = false;
    });
}
