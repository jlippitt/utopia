use super::super::opcode::IType;
use super::super::{Bus, Core, GPR};
use tracing::trace;

pub fn sb(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SB {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.write_u8(address, core.getd(op.rt()) as u8);
}

pub fn sh(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SH {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.write_u16(address, core.getd(op.rt()) as u16);
}

pub fn sw(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SW {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.write_u32(address, core.getw(op.rt()));
}

pub fn sd(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SD {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.write_u64(address, core.getd(op.rt()));
}

pub fn swl(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SWL {}, {}({})",
        core.pc,
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    let bytes = core.getw(op.rt()).to_be_bytes();

    for index in 0..=(address & 3 ^ 3) {
        core.write_u8(address.wrapping_add(index), bytes[index as usize]);
    }
}

pub fn swr(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SWR {}, {}({})",
        core.pc,
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    let bytes = core.getw(op.rt()).to_be_bytes();

    for index in 0..=(address & 3) {
        core.write_u8(address.wrapping_sub(index), bytes[(index ^ 3) as usize]);
    }
}

pub fn sdl(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SDL {}, {}({})",
        core.pc,
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    let bytes = core.getd(op.rt()).to_be_bytes();

    for index in 0..=(address & 7 ^ 7) {
        core.write_u8(address.wrapping_add(index), bytes[index as usize]);
    }
}

pub fn sdr(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SDR {}, {}({})",
        core.pc,
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    let bytes = core.getd(op.rt()).to_be_bytes();

    for index in 0..=(address & 7) {
        core.write_u8(address.wrapping_sub(index), bytes[(index ^ 7) as usize]);
    }
}
