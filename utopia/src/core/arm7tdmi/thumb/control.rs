use super::super::condition::Condition;
use super::super::{Bus, Core, Mode, REGS};
use num_traits::FromPrimitive;
use tracing::trace;

pub fn branch_conditional(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let code = (word >> 8) & 15;

    let condition = Condition::from_u16(code).unwrap();

    if !condition.apply(core) {
        trace!("{:08X}: ({}: Skipped)", core.pc, condition);
        return;
    }

    let offset = ((word & 0xff) as i8 as i32) << 1;
    trace!("{:08X} B {:+}", pc, offset);
    core.pc = core.pc.wrapping_add(2).wrapping_add(offset as u32);
}

pub fn branch_unconditional(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = (((word & 0x07ff) as i32) << 21) >> 20;
    trace!("{:08X} B {:+}", pc, offset);
    core.pc = core.pc.wrapping_add(2).wrapping_add(offset as u32);
}

pub fn branch_and_link<const SELECTOR: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = word as u32 & 0x07ff;

    if SELECTOR {
        let next = core.pc;
        core.pc = core.get(14).wrapping_add(offset << 1);
        trace!("{:08X} BL 0x{:08X}", pc.wrapping_sub(2), core.pc);
        core.set(14, next | 1);
    } else {
        let signed_offset = (((offset as i32) << 21) >> 9) as u32;
        let result = core.pc.wrapping_add(2).wrapping_add(signed_offset);
        core.regs[14] = result;
    }
}

pub fn bx(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rs = ((word >> 3) & 15) as usize;

    trace!("{:08X} BX {}", pc, REGS[rs]);

    let target = core.get(rs);
    core.cpsr.t = (target & 0x0000_0001) != 0;

    if core.cpsr.t {
        core.pc = target & 0xffff_fffe;
        trace!("  Thumb Mode");
    } else {
        core.pc = target & 0xffff_fffc;
        trace!("  ARM Mode");
    }
}

pub fn swi(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    trace!("{:08X} SWI #{:X}", pc, word & 0xff);
    let saved_cpsr = core.cpsr_to_u32();
    core.cpsr.t = false;
    trace!("  ARM Mode");
    core.set_mode(Mode::Supervisor);
    core.set(14, core.pc);
    core.pc = 0x0000_0008;
    core.spsr_from_u32(saved_cpsr, true);
}
