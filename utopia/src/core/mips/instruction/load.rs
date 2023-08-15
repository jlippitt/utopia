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

    let prev_bytes = core.get(rt).to_be_bytes();

    let result = match address & 3 {
        0 => core.read_word(address),
        1 => u32::from_be_bytes([
            core.read_byte(address),
            core.read_byte(address.wrapping_add(1)),
            core.read_byte(address.wrapping_add(2)),
            prev_bytes[3],
        ]),
        2 => u32::from_be_bytes([
            core.read_byte(address),
            core.read_byte(address.wrapping_add(1)),
            prev_bytes[2],
            prev_bytes[3],
        ]),
        3 => u32::from_be_bytes([
            core.read_byte(address),
            prev_bytes[1],
            prev_bytes[2],
            prev_bytes[3],
        ]),
        _ => unreachable!(),
    };

    core.set(rt, result);
}

pub fn lwr(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} LWR {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);

    let prev_bytes = core.get(rt).to_be_bytes();

    let result = match address & 3 {
        0 => u32::from_be_bytes([
            prev_bytes[0],
            prev_bytes[1],
            prev_bytes[2],
            core.read_byte(address),
        ]),
        1 => u32::from_be_bytes([
            prev_bytes[0],
            prev_bytes[1],
            core.read_byte(address.wrapping_sub(1)),
            core.read_byte(address),
        ]),
        2 => u32::from_be_bytes([
            prev_bytes[0],
            core.read_byte(address.wrapping_sub(2)),
            core.read_byte(address.wrapping_sub(1)),
            core.read_byte(address),
        ]),
        3 => core.read_word(address.wrapping_sub(3)),
        _ => unreachable!(),
    };

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
