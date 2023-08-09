use super::super::condition::Condition;
use super::super::{Bus, Core, REGS};
use num_traits::FromPrimitive;
use tracing::debug;

pub fn branch_conditional(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let code = (word >> 8) & 15;

    if code == 0b1111 {
        todo!("SWI");
    }

    let condition = Condition::from_u16(code).unwrap();

    if !condition.apply(core) {
        debug!("{:08X}: ({}: Skipped)", core.pc, condition);
        return;
    }

    let offset = ((word & 0xff) as i8 as i32) << 1;
    debug!("{:08X} B {:+}", pc, offset);
    core.pc = core.pc.wrapping_add(2).wrapping_add(offset as u32);
}

pub fn branch_and_link<const SELECTOR: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = word as u32 & 0x07ff;

    if SELECTOR {
        let next = core.pc;
        core.pc = core.get(14).wrapping_add(offset << 1);
        debug!("{:08X} BL 0x{:08X}", pc, core.pc);
        core.set(14, next | 1);
    } else {
        let signed_offset = (((offset as i32) << 21) >> 9) as u32;
        let result = core.pc.wrapping_add(2).wrapping_add(signed_offset);
        core.regs[14] = result;
    }
}

pub fn bx(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rs = ((word >> 3) & 15) as usize;

    debug!("{:08X} BX {}", pc, REGS[rs]);

    let target = core.get(rs);
    core.pc = target & 0xffff_fffe;
    core.cpsr.t = (target & 0x0000_0001) != 0;

    if core.cpsr.t {
        debug!("  Thumb Mode");
    } else {
        debug!("  ARM Mode");
    }
}
