use super::super::address_mode::{ReadAddress, WriteAddress, AF};
use super::super::{Bus, Core, IRQ_DISABLE, IRQ_ENABLE};
use tracing::trace;

pub fn nop(_core: &mut Core<impl Bus>) {
    trace!("NOP");
}

pub fn halt(_core: &mut Core<impl Bus>) {
    trace!("HALT");
    // TODO: Interrupt handling
}

pub fn di(core: &mut Core<impl Bus>) {
    trace!("DI");
    core.iff = [IRQ_DISABLE; 2];
    core.iff_delayed = [IRQ_DISABLE; 2];
}

pub fn ei(core: &mut Core<impl Bus>) {
    trace!("EI");
    core.iff_delayed = [IRQ_ENABLE; 2];
}

pub fn im(core: &mut Core<impl Bus>, mode: u8) {
    trace!("IM {}", mode);
    core.im = mode;
}

pub fn ex_af_af(core: &mut Core<impl Bus>) {
    trace!("EX AF, AF'");
    let tmp = AF::read(core);
    AF::write(core, core.af_banked);
    core.af_banked = tmp;
}

pub fn exx(core: &mut Core<impl Bus>) {
    trace!("EXX");
    std::mem::swap(&mut core.bc, &mut core.bc_banked);
    std::mem::swap(&mut core.de, &mut core.de_banked);
    std::mem::swap(&mut core.hl, &mut core.hl_banked);
}

pub fn scf(core: &mut Core<impl Bus>) {
    trace!("SCF");
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = true;
}

pub fn ccf(core: &mut Core<impl Bus>) {
    trace!("CCF");
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = !core.flags.c;
}

pub fn cpl(core: &mut Core<impl Bus>) {
    trace!("CPL");
    core.a = !core.a;
    core.flags.n = true;
    core.flags.h = true;
}

pub fn daa(core: &mut Core<impl Bus>) {
    trace!("DAA");

    if core.flags.n {
        if core.flags.h {
            core.a = core.a.wrapping_sub(0x06);
        }

        if core.flags.c {
            core.a = core.a.wrapping_sub(0x60);
        }
    } else {
        if core.flags.h || (core.a & 0x0f) > 0x09 {
            let (result, carries) = core.a.overflowing_add(0x06);
            core.a = result;
            core.flags.c |= carries;
        }

        if core.flags.c || (core.a & 0xf0) > 0x90 {
            let (result, carries) = core.a.overflowing_add(0x60);
            core.a = result;
            core.flags.c |= carries;
        }
    }

    core.set_sz(core.a);
    core.flags.h = false;
}
