use super::super::opcode::{IType, RType};
use super::super::{Bus, Core, GPR};
use tracing::trace;

pub fn andi(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} ANDI {}, {}, {:#04X}",
        core.pc(),
        GPR[op.rt()],
        GPR[op.rs()],
        op.imm(),
    );

    let result = core.getd(op.rs()) & op.imm() as u64;
    core.setd(op.rt(), result);
}

pub fn ori(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} ORI {}, {}, {:#04X}",
        core.pc(),
        GPR[op.rt()],
        GPR[op.rs()],
        op.imm(),
    );

    let result = core.getd(op.rs()) | op.imm() as u64;
    core.setd(op.rt(), result);
}

pub fn xori(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);

    trace!(
        "{:08X} XORI {}, {}, {:#04X}",
        core.pc(),
        GPR[op.rt()],
        GPR[op.rs()],
        op.imm(),
    );

    let result = core.getd(op.rs()) ^ op.imm() as u64;
    core.setd(op.rt(), result);
}

pub fn and(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} AND {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()],
    );

    let result = core.getd(op.rs()) & core.getd(op.rt());
    core.setd(op.rd(), result);
}

pub fn or(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} OR {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()],
    );

    let result = core.getd(op.rs()) | core.getd(op.rt());
    core.setd(op.rd(), result);
}

pub fn xor(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} XOR {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()],
    );

    let result = core.getd(op.rs()) ^ core.getd(op.rt());
    core.setd(op.rd(), result);
}

pub fn nor(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} NOR {}, {}, {}",
        core.pc(),
        GPR[op.rd()],
        GPR[op.rs()],
        GPR[op.rt()],
    );

    let result = !(core.getd(op.rs()) | core.getd(op.rt()));
    core.setd(op.rd(), result);
}
