use super::super::operator::BranchOperator;
use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn j(core: &mut Core<impl Bus>, value: u32) {
    let target = (core.next[0] & 0xfc00_0000) | (value << 2);
    debug!("{:08X} J 0x{:08X}", core.pc, target);
    core.jump_delayed(target);
}

pub fn jal(core: &mut Core<impl Bus>, value: u32) {
    let target = (core.next[0] & 0xfc00_0000) | (value << 2);
    debug!("{:08X} JAL 0x{:08X}", core.pc, target);
    core.setd(31, core.next[1] as u64);
    core.jump_delayed(target);
}

pub fn jr(core: &mut Core<impl Bus>, rs: usize, _rt: usize, _rd: usize, _sa: u32) {
    debug!("{:08X} JR {}", core.pc, REGS[rs]);
    core.jump_delayed(core.get(rs));
}

pub fn jalr(core: &mut Core<impl Bus>, rs: usize, _rt: usize, _rd: usize, _sa: u32) {
    debug!("{:08X} JALR {}", core.pc, REGS[rs]);
    core.setd(31, core.next[1] as u64);
    core.jump_delayed(core.get(rs));
}

pub fn branch<Op: BranchOperator, const LINK: bool, const LIKELY: bool>(
    core: &mut Core<impl Bus>,
    rs: usize,
    rt: usize,
    value: u32,
) {
    let offset = (value as i16 as i32) << 2;

    if Op::UNARY {
        debug!(
            "{:08X} {}{}{} {}, {:+}",
            core.pc,
            Op::NAME,
            if LINK { "AL" } else { "" },
            if LIKELY { "L" } else { "" },
            REGS[rs],
            offset
        );
    } else {
        debug!(
            "{:08X} {}{}{} {}, {}, {:+}",
            core.pc,
            Op::NAME,
            if LINK { "AL" } else { "" },
            if LIKELY { "L" } else { "" },
            REGS[rs],
            REGS[rt],
            offset
        );
    }

    if LINK {
        core.setd(31, core.next[1] as u64);
    }

    if Op::apply(core.getd(rs), core.getd(rt)) {
        debug!("  Branch taken");
        core.jump_delayed(core.next[0].wrapping_add(offset as u32));
    } else {
        debug!("  Branch not taken");

        if LIKELY {
            // Skip the delay slot
            core.jump_now(core.next[1]);
        }
    }
}
