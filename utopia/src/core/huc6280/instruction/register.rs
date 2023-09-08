use super::super::{Bus, Core};
use std::mem;
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

pub fn tam(core: &mut Core<impl Bus>) {
    debug!("TAM #const");
    let mask = core.next_byte();
    core.read(core.pc);
    core.read(core.pc);

    for (bit, mpr) in core.mpr.iter_mut().enumerate() {
        if (mask & (1 << bit)) != 0 {
            *mpr = (core.a as u32) << 13;
            debug!("MPR{}: {:02X} ({:06X})", bit, core.a, mpr);
        }
    }
}

pub fn tma(core: &mut Core<impl Bus>) {
    debug!("TMA #const");
    let mask = core.next_byte();
    core.read(core.pc);
    core.read(core.pc);

    core.a = 0;

    for (bit, mpr) in core.mpr.iter().enumerate() {
        if (mask & (1 << bit)) != 0 {
            core.a |= (mpr >> 13) as u8;
        }
    }
}

pub fn sax(core: &mut Core<impl Bus>) {
    debug!("SAX");
    core.poll();
    core.read(core.pc);
    core.read(core.pc);
    mem::swap(&mut core.a, &mut core.x);
}

pub fn say(core: &mut Core<impl Bus>) {
    debug!("SAY");
    core.poll();
    core.read(core.pc);
    core.read(core.pc);
    mem::swap(&mut core.a, &mut core.y);
}

pub fn sxy(core: &mut Core<impl Bus>) {
    debug!("SXY");
    core.poll();
    core.read(core.pc);
    core.read(core.pc);
    mem::swap(&mut core.x, &mut core.y);
}

pub fn cla(core: &mut Core<impl Bus>) {
    debug!("CLA");
    core.poll();
    core.read(core.pc);
    core.a = 0;
}

pub fn clx(core: &mut Core<impl Bus>) {
    debug!("CLX");
    core.poll();
    core.read(core.pc);
    core.x = 0;
}

pub fn cly(core: &mut Core<impl Bus>) {
    debug!("CLY");
    core.poll();
    core.read(core.pc);
    core.y = 0;
}
