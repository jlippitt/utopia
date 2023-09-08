use super::super::{Bus, Core, IrqDisable};
use tracing::debug;

pub fn clc(core: &mut Core<impl Bus>) {
    debug!("CLC");
    core.poll();
    core.read(core.pc);
    core.flags.c = false;
}

pub fn sec(core: &mut Core<impl Bus>) {
    debug!("SEC");
    core.poll();
    core.read(core.pc);
    core.flags.c = true;
}

pub fn cli(core: &mut Core<impl Bus>) {
    debug!("CLI");
    core.poll();
    core.read(core.pc);
    core.flags.i = IrqDisable::Clear;
}

pub fn sei(core: &mut Core<impl Bus>) {
    debug!("SEI");
    core.poll();
    core.read(core.pc);
    core.flags.i = IrqDisable::Set;
}

pub fn clv(core: &mut Core<impl Bus>) {
    debug!("CLV");
    core.poll();
    core.read(core.pc);
    core.flags.v = 0;
}

pub fn cld(core: &mut Core<impl Bus>) {
    debug!("CLD");
    core.poll();
    core.read(core.pc);
    core.flags.d = false;
}

pub fn sed(core: &mut Core<impl Bus>) {
    debug!("SED");
    core.poll();
    core.read(core.pc);
    core.flags.d = true;
}
