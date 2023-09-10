use super::super::{Bus, Core};
use tracing::debug;

pub fn nop(core: &mut Core<impl Bus>) {
    debug!("NOP");
    core.read(core.pc);
}

pub fn csl(core: &mut Core<impl Bus>) {
    debug!("CSL");
    core.read(core.pc);
    core.read(core.pc);
    core.bus.set_clock_speed(false)
}

pub fn csh(core: &mut Core<impl Bus>) {
    debug!("CSH");
    core.read(core.pc);
    core.read(core.pc);
    core.bus.set_clock_speed(true)
}

pub fn st0(core: &mut Core<impl Bus>) {
    debug!("ST0 #const");
    let value = core.next_byte();
    core.read(core.pc);
    core.write_physical(0x1fe000, value);
}

pub fn st1(core: &mut Core<impl Bus>) {
    debug!("ST1 #const");
    let value = core.next_byte();
    core.read(core.pc);
    core.write_physical(0x1fe002, value);
}

pub fn st2(core: &mut Core<impl Bus>) {
    debug!("ST2 #const");
    let value = core.next_byte();
    core.read(core.pc);
    core.write_physical(0x1fe003, value);
}
