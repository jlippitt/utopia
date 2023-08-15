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

pub fn swl(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SWL {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let bytes = core.get(rt).to_be_bytes();

    for index in 0..=(address & 3 ^ 3) {
        core.write_byte(address.wrapping_add(index), bytes[index as usize]);
    }
}

pub fn swr(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SWR {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let bytes = core.get(rt).to_be_bytes();

    for index in 0..=(address & 3) {
        core.write_byte(address.wrapping_sub(index), bytes[(index ^ 3) as usize]);
    }
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

pub fn sdl(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SDL {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let bytes = core.getd(rt).to_be_bytes();

    for index in 0..=(address & 7 ^ 7) {
        core.write_byte(address.wrapping_add(index), bytes[index as usize]);
    }
}

pub fn sdr(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SDR {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let bytes = core.getd(rt).to_be_bytes();

    for index in 0..=(address & 7) {
        core.write_byte(address.wrapping_sub(index), bytes[(index ^ 7) as usize]);
    }
}
