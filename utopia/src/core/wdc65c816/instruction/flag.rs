use super::super::{Bus, Core, IrqDisable};
use tracing::trace;

pub fn rep<const E: bool>(core: &mut Core<impl Bus>) {
    trace!("REP #const");
    let value = core.next_byte();
    core.poll();
    core.idle();
    core.flags_from_u8::<E>(core.flags_to_u8::<E>(false) & !value);
}

pub fn sep<const E: bool>(core: &mut Core<impl Bus>) {
    trace!("REP #const");
    let value = core.next_byte();
    core.poll();
    core.idle();
    core.flags_from_u8::<E>(core.flags_to_u8::<E>(false) | value);
}

pub fn clc(core: &mut Core<impl Bus>) {
    trace!("CLC");
    core.poll();
    core.idle();
    core.flags.c = false;
}

pub fn sec(core: &mut Core<impl Bus>) {
    trace!("SEC");
    core.poll();
    core.idle();
    core.flags.c = true;
}

pub fn cli(core: &mut Core<impl Bus>) {
    trace!("CLI");
    core.poll();
    core.idle();
    core.flags.i = IrqDisable::Clear;
}

pub fn sei(core: &mut Core<impl Bus>) {
    trace!("SEI");
    core.poll();
    core.idle();
    core.flags.i = IrqDisable::Set;
}

pub fn clv(core: &mut Core<impl Bus>) {
    trace!("CLV");
    core.poll();
    core.idle();
    core.flags.v = false;
}

pub fn cld(core: &mut Core<impl Bus>) {
    trace!("CLD");
    core.poll();
    core.idle();
    core.flags.d = false;
}

pub fn sed(core: &mut Core<impl Bus>) {
    trace!("SED");
    core.poll();
    core.idle();
    core.flags.d = true;
}
