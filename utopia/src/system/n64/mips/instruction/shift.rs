use super::super::opcode::RType;
use super::super::{Bus, Core, GPR};
use tracing::trace;

pub fn sll(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    if word == 0 {
        trace!("{:08X} NOP", core.pc());
    } else {
        trace!(
            "{:08X} SLL {}, {}, {}",
            core.pc(),
            GPR[op.rd()],
            GPR[op.rt()],
            op.sa(),
        );
    }

    let result = core.getw(op.rt()) << op.sa();
    core.setw(op.rd(), result);
}

pub fn srl(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} SRL {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        op.sa(),
    );

    let result = core.getw(op.rt()) >> op.sa();
    core.setw(op.rd(), result);
}

pub fn sra(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} SRA {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        op.sa(),
    );

    let result = (core.getd(op.rt()) as i64) >> op.sa();
    core.setw(op.rd(), result as u32);
}

pub fn sllv(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} SLLV {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        GPR[op.rs()],
    );

    let result = core.getw(op.rt()) << (core.getw(op.rs()) & 31);
    core.setw(op.rd(), result);
}

pub fn srlv(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} SRLV {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        GPR[op.rs()],
    );

    let result = core.getw(op.rt()) >> (core.getw(op.rs()) & 31);
    core.setw(op.rd(), result);
}

pub fn srav(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} SRAV {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        GPR[op.rs()],
    );

    let result = (core.getd(op.rt()) as i64) >> (core.getw(op.rs()) & 31);
    core.setw(op.rd(), result as u32);
}

pub fn dsll(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSLL {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        op.sa(),
    );

    let result = core.getd(op.rt()) << op.sa();
    core.setd(op.rd(), result);
}

pub fn dsrl(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSRL {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        op.sa(),
    );

    let result = core.getd(op.rt()) >> op.sa();
    core.setd(op.rd(), result);
}

pub fn dsra(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSRA {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        op.sa(),
    );

    let result = (core.getd(op.rt()) as i64) >> op.sa();
    core.setd(op.rd(), result as u64);
}

pub fn dsll32(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSLL32 {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        op.sa(),
    );

    let result = core.getd(op.rt()) << (op.sa() + 32);
    core.setd(op.rd(), result);
}

pub fn dsrl32(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSRL32 {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        op.sa(),
    );

    let result = core.getd(op.rt()) >> (op.sa() + 32);
    core.setd(op.rd(), result);
}

pub fn dsra32(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSRA32 {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        op.sa(),
    );

    let result = (core.getd(op.rt()) as i64) >> (op.sa() + 32);
    core.setd(op.rd(), result as u64);
}

pub fn dsllv(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSLLV {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        GPR[op.rs()],
    );

    let result = core.getd(op.rt()) << (core.getd(op.rs()) & 63);
    core.setd(op.rd(), result);
}

pub fn dsrlv(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSRLV {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        GPR[op.rs()],
    );

    let result = core.getd(op.rt()) >> (core.getd(op.rs()) & 63);
    core.setd(op.rd(), result);
}

pub fn dsrav(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DSRAV {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rt()],
        GPR[op.rs()],
    );

    let result = (core.getd(op.rt()) as i64) >> (core.getd(op.rs()) & 63);
    core.setd(op.rd(), result as u64);
}
