pub use registers::{DmaType, Registers};

use super::super::interrupt::RcpInterrupt;
use super::super::mips::opcode::RType;
use super::super::mips::{self, Bus, Core, GPR};
use tracing::trace;

mod registers;

pub struct Cp0 {
    regs: Registers,
}

impl Cp0 {
    pub fn new(rcp_int: RcpInterrupt) -> Self {
        Self {
            regs: Registers::new(rcp_int),
        }
    }

    pub fn regs(&self) -> &Registers {
        &self.regs
    }

    pub fn regs_mut(&mut self) -> &mut Registers {
        &mut self.regs
    }
}

impl mips::Cp0 for Cp0 {
    fn translate(&self, address: u32) -> u32 {
        address & 0xfff
    }

    fn mfc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        let op = RType::from(word);
        trace!(
            "{:08X} MFC0 {}, {}",
            core.pc(),
            GPR[op.rt()],
            Registers::NAMES[op.rd()]
        );
        let value = core.cp0().regs.get(op.rd());
        core.setw(op.rt(), value);
    }

    fn mtc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        let op = RType::from(word);
        trace!(
            "{:08X} MTC0 {}, {}",
            core.pc(),
            GPR[op.rt()],
            Registers::NAMES[op.rd()]
        );
        let value = core.getw(op.rt());
        core.cp0_mut().regs.set(op.rd(), value.into());
    }

    fn cop0(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("RSP CP0 COP0");
    }

    fn syscall(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("RSP SYSCALL");
    }

    fn break_(core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        trace!("{:08X} BREAK", core.pc());
        core.cp0_mut().regs.break_();
    }

    fn step(_core: &mut Core<impl Bus<Cp0 = Self>>) {
        // No processing required here
    }
}
