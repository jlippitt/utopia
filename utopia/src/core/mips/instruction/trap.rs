use super::super::opcode::RType;
use super::super::{Bus, Core, GPR};
use tracing::trace;

pub fn teq(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);
    trace!("{:08X} TEQ {}, {}", core.pc(), GPR[op.rs()], GPR[op.rt()],);
    let condition = core.getd(op.rs()) == core.getd(op.rt());
    core.trap_if(condition);
}

pub fn tne(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);
    trace!("{:08X} TNE {}, {}", core.pc(), GPR[op.rs()], GPR[op.rt()],);
    let condition = core.getd(op.rs()) != core.getd(op.rt());
    core.trap_if(condition);
}
