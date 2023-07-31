use super::super::operator::{BinaryOperator, Cmp, ComparisonOperator, Mov, MoveOperator};
use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn binary_immediate<Op: BinaryOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let value = word & 0xff;

    debug!("{:08X} {} {}, #0x{:02X}", pc, Op::NAME, REGS[rd], value);

    let result = Op::apply::<true>(core, core.get(rd), value as u32);
    core.set(rd, result);
}

pub fn move_immediate(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let value = word & 0xff;

    debug!("{:08X} MOV {}, #0x{:02X}", pc, REGS[rd], value);

    let result = Mov::apply::<true>(core, value as u32);
    core.set(rd, result);
}

pub fn compare_immediate(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let value = word & 0xff;

    debug!("{:08X} CMP {}, #0x{:02X}", pc, REGS[rd], value);

    Cmp::apply(core, core.get(rd), value as u32);
}
