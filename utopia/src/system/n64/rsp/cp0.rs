use super::registers::Registers;
use crate::core::mips::{Bus, Coprocessor0, Core, REGS};
use tracing::debug;

pub struct Cp0 {
    regs: Registers,
}

impl Cp0 {
    pub fn new(regs: Registers) -> Self {
        Self { regs }
    }
}

impl Coprocessor0 for Cp0 {
    fn dispatch(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        match (word >> 21) & 31 {
            0b00000 => type_r(core, mfc0, word),
            0b00100 => type_r(core, mtc0, word),
            rs => unimplemented!("RSP CP0 RS={:05b} ({:08X}: {:08X})", rs, core.pc(), word),
        }
    }
}

fn type_r<T: Bus>(core: &mut Core<T>, instr: impl Fn(&mut Core<T>, usize, usize), word: u32) {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    instr(core, rt, rd);
}

fn mfc0(core: &mut Core<impl Bus<Cp0 = Cp0>>, rt: usize, rd: usize) {
    debug!(
        "{:08X} MFC0 {}, {}",
        core.pc(),
        REGS[rt],
        Registers::NAMES[rd]
    );
    let result = core.cp0().regs.get(rd);
    core.set(rt, result);
}

fn mtc0(core: &mut Core<impl Bus<Cp0 = Cp0>>, rt: usize, rd: usize) {
    debug!(
        "{:08X} MTC0 {}, {}",
        core.pc(),
        REGS[rt],
        Registers::NAMES[rd]
    );
    let value = core.get(rt);
    core.cp0_mut().regs.set(rd, value);
}
