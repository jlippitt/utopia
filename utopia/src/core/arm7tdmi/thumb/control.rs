use super::super::condition::Condition;
use super::super::{Bus, Core};
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
