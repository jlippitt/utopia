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

    match address & 3 {
        0 => core.write_word(address, u32::from_be_bytes(bytes)),
        1 => {
            core.write_byte(address, bytes[0]);
            core.write_byte(address.wrapping_add(1), bytes[1]);
            core.write_byte(address.wrapping_add(2), bytes[2]);
        }
        2 => {
            core.write_byte(address, bytes[0]);
            core.write_byte(address.wrapping_add(1), bytes[1]);
        }
        3 => {
            core.write_byte(address, bytes[0]);
        }
        _ => unreachable!(),
    };
}

pub fn swr(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SWR {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    let bytes = core.get(rt).to_be_bytes();

    match address & 3 {
        0 => {
            core.write_byte(address, bytes[3]);
        }
        1 => {
            core.write_byte(address.wrapping_sub(1), bytes[2]);
            core.write_byte(address, bytes[3]);
        }
        2 => {
            core.write_byte(address.wrapping_sub(2), bytes[1]);
            core.write_byte(address.wrapping_sub(1), bytes[2]);
            core.write_byte(address, bytes[3]);
        }
        3 => core.write_word(address.wrapping_sub(3), u32::from_be_bytes(bytes)),
        _ => unreachable!(),
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
