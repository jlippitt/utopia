use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn sb(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SB {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    core.write_byte(address, core.get(rt) as u8);
}

pub fn sh(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SH {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    core.write_halfword(address, core.get(rt) as u16);
}

pub fn sw(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SW {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    core.write_word(address, core.get(rt));
}

pub fn sd(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SD {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    core.write_doubleword(address, core.getd(rt));
}
