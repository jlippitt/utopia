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
