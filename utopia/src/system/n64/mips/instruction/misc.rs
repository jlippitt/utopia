use super::super::opcode::IType;
use super::super::{Bus, Core, GPR};
use tracing::trace;

pub fn cache(core: &mut Core<impl Bus>, word: u32) {
    let op = IType::from(word);
    let offset = op.imm() as i16;

    trace!(
        "{:08X} CACHE {:#0X}, {}({})",
        core.pc(),
        op.rt(),
        offset,
        GPR[op.rs()]
    );

    // Ignore for now
}

pub fn sync(core: &mut Core<impl Bus>, _word: u32) {
    trace!("{:08X} SYNC", core.pc(),);

    // This is a NOP on the VR4300 (and presumably on the RSP as well?)
}
