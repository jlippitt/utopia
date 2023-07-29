use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn lui(core: &mut Core<impl Bus>, _rs: usize, rt: usize, value: u32) {
    debug!("{:08X} LUI {}, 0x{:04X}", core.pc, REGS[rt], value);
    core.set(rt, value << 16);
}

pub fn lbu(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LBU {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let result = core.read_byte(address);
    core.set(rt, result as u32);
}

pub fn lw(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LW {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let result = core.read_word(address);
    core.set(rt, result);
}
