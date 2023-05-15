use super::super::{Bus, Core};
use tracing::debug;

pub fn dex(core: &mut Core<impl Bus>) {
    debug!("DEX");
    core.poll();
    core.read(core.pc);
    core.x = core.x.wrapping_sub(1);
    core.set_nz(core.x);
}

pub fn dey(core: &mut Core<impl Bus>) {
    debug!("DEY");
    core.poll();
    core.read(core.pc);
    core.y = core.y.wrapping_sub(1);
    core.set_nz(core.y);
}

pub fn inx(core: &mut Core<impl Bus>) {
    debug!("INX");
    core.poll();
    core.read(core.pc);
    core.x = core.x.wrapping_add(1);
    core.set_nz(core.x);
}

pub fn iny(core: &mut Core<impl Bus>) {
    debug!("INY");
    core.poll();
    core.read(core.pc);
    core.y = core.y.wrapping_add(1);
    core.set_nz(core.y);
}

pub fn txs(core: &mut Core<impl Bus>) {
    debug!("TXS");
    core.poll();
    core.read(core.pc);
    core.s = core.x;
}
