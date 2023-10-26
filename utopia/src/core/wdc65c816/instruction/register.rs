use super::super::{Bus, Core, EMULATION_STACK_PAGE};
use tracing::trace;

pub fn dex<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("DEX.{}", super::size(X));
    core.poll();
    core.idle();
    core.x = core.x.wrapping_sub(1);

    if X {
        core.x &= 0xff;
        core.set_nz8(core.x as u8);
    } else {
        core.set_nz16(core.x);
    }
}

pub fn dey<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("DEY.{}", super::size(X));
    core.poll();
    core.idle();
    core.y = core.y.wrapping_sub(1);

    if X {
        core.y &= 0xff;
        core.set_nz8(core.y as u8);
    } else {
        core.set_nz16(core.y);
    }
}

pub fn inx<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("INX.{}", super::size(X));
    core.poll();
    core.idle();
    core.x = core.x.wrapping_add(1);

    if X {
        core.x &= 0xff;
        core.set_nz8(core.x as u8);
    } else {
        core.set_nz16(core.x);
    }
}

pub fn iny<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("INY.{}", super::size(X));
    core.poll();
    core.idle();
    core.y = core.y.wrapping_add(1);

    if X {
        core.y &= 0xff;
        core.set_nz8(core.y as u8);
    } else {
        core.set_nz16(core.y);
    }
}

pub fn tcd(core: &mut Core<impl Bus>) {
    trace!("TCD");
    core.poll();
    core.idle();
    core.d = core.a;
    core.set_nz16(core.d);
}

pub fn tdc(core: &mut Core<impl Bus>) {
    trace!("TDC");
    core.poll();
    core.idle();
    core.a = core.d;
    core.set_nz16(core.a);
}

pub fn tcs<const E: bool>(core: &mut Core<impl Bus>) {
    trace!("TCS");
    core.poll();
    core.idle();

    if E {
        core.s = EMULATION_STACK_PAGE | (core.a & 0xff);
    } else {
        core.s = core.a;
    }
}

pub fn tsc(core: &mut Core<impl Bus>) {
    trace!("TSC");
    core.poll();
    core.idle();
    core.a = core.s;
    core.set_nz16(core.a);
}

pub fn tax<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("TAX.{}", super::size(X));
    core.poll();
    core.idle();

    if X {
        core.x = core.a & 0xff;
        core.set_nz8(core.x as u8);
    } else {
        core.x = core.a;
        core.set_nz16(core.x);
    }
}

pub fn txa<const M: bool>(core: &mut Core<impl Bus>) {
    trace!("TXA.{}", super::size(M));
    core.poll();
    core.idle();

    if M {
        core.a = (core.a & 0xff00) | (core.x & 0xff);
        core.set_nz8(core.a as u8);
    } else {
        core.a = core.x;
        core.set_nz16(core.a);
    }
}

pub fn tay<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("TAY.{}", super::size(X));
    core.poll();
    core.idle();

    if X {
        core.y = core.a & 0xff;
        core.set_nz8(core.y as u8);
    } else {
        core.y = core.a;
        core.set_nz16(core.y);
    }
}

pub fn tya<const M: bool>(core: &mut Core<impl Bus>) {
    trace!("TYA.{}", super::size(M));
    core.poll();
    core.idle();

    if M {
        core.a = (core.a & 0xff00) | (core.y & 0xff);
        core.set_nz8(core.a as u8);
    } else {
        core.a = core.y;
        core.set_nz16(core.a);
    }
}

pub fn txy<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("TXY.{}", super::size(X));
    core.poll();
    core.idle();
    core.y = core.x;

    if X {
        core.set_nz8(core.y as u8);
    } else {
        core.set_nz16(core.y);
    }
}

pub fn tyx<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("TYX.{}", super::size(X));
    core.poll();
    core.idle();
    core.x = core.y;

    if X {
        core.set_nz8(core.x as u8);
    } else {
        core.set_nz16(core.x);
    }
}

pub fn tsx<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("TSX.{}", super::size(X));
    core.poll();
    core.idle();

    if X {
        core.x = core.s & 0xff;
        core.set_nz8(core.x as u8);
    } else {
        core.x = core.s;
        core.set_nz16(core.x);
    }
}

pub fn txs<const E: bool>(core: &mut Core<impl Bus>) {
    trace!("TXS");
    core.poll();
    core.idle();

    if E {
        core.s = EMULATION_STACK_PAGE | (core.x & 0xff);
    } else {
        core.s = core.x;
    }
}
