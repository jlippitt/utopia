use super::super::opcode::{IType, RType};
use super::super::{Bus, Core, GPR};
use tracing::trace;

pub fn addi(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} ADDI {}, {}, {}",
        core.pc(),
        GPR[op.rt()],
        GPR[op.rs()],
        op.imm() as i16
    );

    let (result, overflow) = (core.getw(op.rs()) as i32).overflowing_add(op.imm() as i16 as i32);

    if overflow {
        unimplemented!("ADDI overflow handling")
    }

    core.setw(op.rt(), result as u32);
}

pub fn addiu(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} ADDIU {}, {}, {}",
        core.pc(),
        GPR[op.rt()],
        GPR[op.rs()],
        op.imm() as i16
    );

    let result = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
    core.setw(op.rt(), result);
}

pub fn daddi(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} DADDI {}, {}, {}",
        core.pc(),
        GPR[op.rt()],
        GPR[op.rs()],
        op.imm() as i16
    );

    let (result, overflow) = (core.getd(op.rs()) as i64).overflowing_add(op.imm() as i16 as i64);

    if overflow {
        unimplemented!("DADDI overflow handling")
    }

    core.setd(op.rt(), result as u64);
}

pub fn daddiu(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} DADDIU {}, {}, {}",
        core.pc(),
        GPR[op.rt()],
        GPR[op.rs()],
        op.imm() as i16
    );

    let result = core.getd(op.rs()).wrapping_add(op.imm() as i16 as u64);
    core.setd(op.rt(), result);
}

pub fn slti(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SLTI {}, {}, {}",
        core.pc(),
        GPR[op.rt()],
        GPR[op.rs()],
        op.imm() as i16
    );

    let result = (core.getd(op.rs()) as i64) < op.imm() as i16 as i64;
    core.setd(op.rt(), result as u64);
}

pub fn sltiu(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} SLTIU {}, {}, {}",
        core.pc(),
        GPR[op.rt()],
        GPR[op.rs()],
        op.imm() as i16,
    );

    let result = core.getd(op.rs()) < op.imm() as i16 as u64;
    core.setd(op.rt(), result as u64);
}

pub fn add(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} ADD {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let (result, overflow) = (core.getw(op.rs()) as i32).overflowing_add(core.getw(op.rt()) as i32);

    if overflow {
        unimplemented!("ADD overflow handling")
    }

    core.setw(op.rd(), result as u32);
}

pub fn addu(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} ADDU {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let result = core.getw(op.rs()).wrapping_add(core.getw(op.rt()));
    core.setw(op.rd(), result);
}

pub fn sub(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} SUB {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let (result, overflow) = (core.getw(op.rs()) as i32).overflowing_sub(core.getw(op.rt()) as i32);

    if overflow {
        unimplemented!("SUB overflow handling")
    }

    core.setw(op.rd(), result as u32);
}

pub fn subu(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} SUBU {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let result = core.getw(op.rs()).wrapping_sub(core.getw(op.rt()));
    core.setw(op.rd(), result);
}

pub fn dadd(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DADD {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let (result, overflow) = (core.getd(op.rs()) as i64).overflowing_add(core.getd(op.rt()) as i64);

    if overflow {
        unimplemented!("ADD overflow handling")
    }

    core.setd(op.rd(), result as u64);
}

pub fn daddu(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DADDU {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let result = core.getd(op.rs()).wrapping_add(core.getd(op.rt()));
    core.setd(op.rd(), result);
}

pub fn dsub(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSUB {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let (result, overflow) = (core.getd(op.rs()) as i64).overflowing_sub(core.getd(op.rt()) as i64);

    if overflow {
        unimplemented!("SUB overflow handling")
    }

    core.setd(op.rd(), result as u64);
}

pub fn dsubu(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSUBU {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let result = core.getd(op.rs()).wrapping_sub(core.getd(op.rt()));
    core.setd(op.rd(), result);
}

pub fn slt(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} SLT {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let result = (core.getd(op.rs()) as i64) < core.getd(op.rt()) as i64;
    core.setd(op.rd(), result as u64);
}

pub fn sltu(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} SLTU {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let result = core.getd(op.rs()) < core.getd(op.rt());
    core.setd(op.rd(), result as u64);
}
