use super::super::opcode::IType;
use super::super::{Bus, Core, GPR};
use tracing::trace;

pub fn lui(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);
    trace!("{:08X} LUI {}, {:#04X}", core.pc(), GPR[op.rt()], op.imm());
    core.setw(op.rt(), op.imm() << 16);
}

pub fn lb(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LBU {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.setd(op.rt(), core.read_u8(address) as i8 as u64);
}

pub fn lbu(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LBU {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.setd(op.rt(), core.read_u8(address) as u64);
}

pub fn lh(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LH {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.setd(op.rt(), core.read_u16(address) as i16 as u64);
}

pub fn lhu(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LHU {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.setd(op.rt(), core.read_u16(address) as u64);
}

pub fn lw(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LW {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.setw(op.rt(), core.read_u32(address));
}

pub fn lwu(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LWU {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.setd(op.rt(), core.read_u32(address) as u64);
}

pub fn ld(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LD {}, {}({})",
        core.pc(),
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.setd(op.rt(), core.read_u64(address));
}

pub fn lwl(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LWL {}, {}({})",
        core.pc,
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    let mut result = core.getw(op.rt());

    for index in 0..=(address & 3 ^ 3) {
        let shift = (index ^ 3) << 3;
        result &= !0xffu32.rotate_left(shift);
        result |= (core.read_u8(address.wrapping_add(index)) as u32) << shift;
    }

    core.setw(op.rt(), result);
}

pub fn lwr(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LWR {}, {}({})",
        core.pc,
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    let mut result = core.getw(op.rt());

    for index in 0..=(address & 3) {
        let shift = index << 3;
        result &= !0xffu32.rotate_left(shift);
        result |= (core.read_u8(address.wrapping_sub(index)) as u32) << shift;
    }

    core.setw(op.rt(), result);
}

pub fn ldl(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LDL {}, {}({})",
        core.pc,
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    let mut result = core.getd(op.rt());

    for index in 0..=(address & 7 ^ 7) {
        let shift = (index ^ 7) << 3;
        result &= !0xffu64.rotate_left(shift);
        result |= (core.read_u8(address.wrapping_add(index)) as u64) << shift;
    }

    core.setd(op.rt(), result);
}

pub fn ldr(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} LDR {}, {}({})",
        core.pc,
        GPR[op.rt()],
        op.imm() as i16,
        GPR[op.rs()]
    );

    let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    let mut result = core.getd(op.rt());

    for index in 0..=(address & 7) {
        let shift = index << 3;
        result &= !0xffu64.rotate_left(shift);
        result |= (core.read_u8(address.wrapping_sub(index)) as u64) << shift;
    }

    core.setd(op.rt(), result);
}
