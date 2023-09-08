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

pub fn tax(core: &mut Core<impl Bus>) {
    debug!("TAX");
    core.poll();
    core.read(core.pc);
    core.x = core.a;
    core.set_nz(core.x);
}

pub fn txa(core: &mut Core<impl Bus>) {
    debug!("TXA");
    core.poll();
    core.read(core.pc);
    core.a = core.x;
    core.set_nz(core.a);
}

pub fn tay(core: &mut Core<impl Bus>) {
    debug!("TAY");
    core.poll();
    core.read(core.pc);
    core.y = core.a;
    core.set_nz(core.y);
}

pub fn tya(core: &mut Core<impl Bus>) {
    debug!("TYA");
    core.poll();
    core.read(core.pc);
    core.a = core.y;
    core.set_nz(core.a);
}

pub fn tsx(core: &mut Core<impl Bus>) {
    debug!("TSX");
    core.poll();
    core.read(core.pc);
    core.x = core.s;
    core.set_nz(core.x);
}

pub fn txs(core: &mut Core<impl Bus>) {
    debug!("TXS");
    core.poll();
    core.read(core.pc);
    core.s = core.x;
}
