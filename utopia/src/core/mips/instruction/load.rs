use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn lui(core: &mut Core<impl Bus>, _rs: usize, rt: usize, value: u32) {
    debug!("{:08X} LUI {}, 0x{:04X}", core.pc, REGS[rt], value);
    core.set(rt, value << 16);
}

pub fn lb(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LB {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let result = core.read_byte(address) as i8;
    core.set(rt, result as u32);
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

pub fn lh(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LH {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let result = core.read_halfword(address) as i16;
    core.set(rt, result as u32);
}

pub fn lhu(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LHU {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let result = core.read_halfword(address);
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

pub fn lwl(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LWL {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);

    let new_value = core.read_word(address & !3);
    let shift = (address & 3) << 3;
    let mask_old = 0xffff_ffffu32.checked_shr(32 - shift).unwrap_or(0);

    let result = (core.get(rt) & mask_old) | (new_value << shift);

    core.set(rt, result);
}

pub fn lwr(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LWR {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);

    let new_value = core.read_word(address & !3);
    let shift = ((address & 3) + 1) << 3;
    let mask_old = 0xffff_ffffu32.checked_shl(shift).unwrap_or(0);

    let result = (core.get(rt) & mask_old) | (new_value >> (32 - shift));

    core.set(rt, result);
}

pub fn lwu(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LWU {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let result = core.read_word(address);
    core.setd(rt, result as u64);
}

pub fn ld(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LD {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let result = core.read_doubleword(address);
    core.setd(rt, result);
}
