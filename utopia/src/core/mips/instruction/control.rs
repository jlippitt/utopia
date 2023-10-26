use super::super::opcode::{IType, JType, RType};
use super::super::{Bus, Core, GPR};
use tracing::trace;

pub fn beq<const LIKELY: bool>(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);
    let offset = (op.imm() as i16 as i32) << 2;

    trace!(
        "{:08X} BEQ{} {}, {}, {:+}",
        core.pc(),
        if LIKELY { "L" } else { "" },
        GPR[op.rs()],
        GPR[op.rt()],
        offset
    );

    let condition = core.getd(op.rs()) == core.getd(op.rt());
    core.branch_if::<LIKELY>(condition, offset);
}

pub fn bne<const LIKELY: bool>(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);
    let offset = (op.imm() as i16 as i32) << 2;

    trace!(
        "{:08X} BNE{} {}, {}, {:+}",
        core.pc(),
        if LIKELY { "L" } else { "" },
        GPR[op.rs()],
        GPR[op.rt()],
        offset
    );

    let condition = core.getd(op.rs()) != core.getd(op.rt());
    core.branch_if::<LIKELY>(condition, offset);
}

pub fn blez<const LIKELY: bool>(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);
    let offset = (op.imm() as i16 as i32) << 2;

    trace!(
        "{:08X} BLEZ{} {}, {:+}",
        core.pc(),
        if LIKELY { "L" } else { "" },
        GPR[op.rs()],
        offset
    );

    let condition = (core.getd(op.rs()) as i64) <= 0;
    core.branch_if::<LIKELY>(condition, offset);
}

pub fn bgtz<const LIKELY: bool>(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);
    let offset = (op.imm() as i16 as i32) << 2;

    trace!(
        "{:08X} BGTZ{} {}, {:+}",
        core.pc(),
        if LIKELY { "L" } else { "" },
        GPR[op.rs()],
        offset
    );

    let condition = (core.getd(op.rs()) as i64) > 0;
    core.branch_if::<LIKELY>(condition, offset);
}

pub fn bltz<const LINK: bool, const LIKELY: bool>(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);
    let offset = (op.imm() as i16 as i32) << 2;

    trace!(
        "{:08X} BLTZ{}{} {}, {:+}",
        core.pc(),
        if LINK { "AL" } else { "" },
        if LIKELY { "L" } else { "" },
        GPR[op.rs()],
        offset
    );

    if LINK {
        core.setw(31, core.next[1]);
    }

    let condition = (core.getd(op.rs()) as i64) < 0;
    core.branch_if::<LIKELY>(condition, offset);
}

pub fn bgez<const LINK: bool, const LIKELY: bool>(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);
    let offset = (op.imm() as i16 as i32) << 2;

    trace!(
        "{:08X} BGEZ{}{} {}, {:+}",
        core.pc(),
        if LINK { "AL" } else { "" },
        if LIKELY { "L" } else { "" },
        GPR[op.rs()],
        offset
    );

    if LINK {
        core.setw(31, core.next[1]);
    }

    let condition = (core.getd(op.rs()) as i64) >= 0;
    core.branch_if::<LIKELY>(condition, offset);
}

pub fn j<const LINK: bool>(core: &mut Core<impl Bus>, word: u32) {
    let op = JType::from(word);
    let address = (core.next[0] & 0xf000_0000) | (op.imm() << 2);

    trace!(
        "{:08X} J{} {:#08X}",
        core.pc(),
        if LINK { "AL" } else { "" },
        address
    );

    if LINK {
        core.setw(31, core.next[1]);
    }

    core.jump_delayed(address);
}

pub fn jr(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);
    trace!("{:08X} JR {}", core.pc(), GPR[op.rs()]);
    let address = core.getw(op.rs());
    core.jump_delayed(address);
}

pub fn jalr(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);
    trace!("{:08X} JALR {}, {}", core.pc(), GPR[op.rd()], GPR[op.rs()]);
    let address = core.getw(op.rs());
    core.setw(op.rd(), core.next[1]);
    core.jump_delayed(address);
}
