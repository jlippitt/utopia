use super::super::{Bus, Core, REGS};
use tracing::trace;

pub fn branch<const LINK: bool>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let offset = ((word as i32) << 8) >> 6;

    trace!("{:08X} B{} {:+}", pc, if LINK { "L" } else { "" }, offset);

    if LINK {
        core.set(14, core.pc);
    }

    core.pc = core.pc.wrapping_add(4).wrapping_add(offset as u32);
}

pub fn bx(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let rn = (word & 15) as usize;
    trace!("{:08X} BX {}", pc, REGS[rn]);
    let target = core.get(rn);
    core.pc = target & 0xffff_fffe;
    core.cpsr.t = (target & 0x0000_0001) != 0;

    if core.cpsr.t {
        trace!("  Thumb Mode");
    } else {
        trace!("  ARM Mode");
    }
}
