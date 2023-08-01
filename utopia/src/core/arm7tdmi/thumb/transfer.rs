use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn ldr_pc_relative(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 15) as usize;
    let offset = (word & 0xff) << 2;

    debug!("{:08X} LDR {}, [PC, #{}]", pc, REGS[rd], offset);

    let address = core.pc.wrapping_add(offset as u32);
    let result = core.read_word(address);
    core.set(rd, result);
}
