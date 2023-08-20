use super::super::rdp::Registers as RdpRegisters;
use super::registers::Registers as RspRegisters;
use crate::core::mips::{Bus, Coprocessor0, Core, REGS};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::debug;

pub struct Cp0 {
    rsp_regs: Rc<RefCell<RspRegisters>>,
    rdp_regs: Rc<RefCell<RdpRegisters>>,
}

impl Cp0 {
    pub fn new(rsp_regs: Rc<RefCell<RspRegisters>>, rdp_regs: Rc<RefCell<RdpRegisters>>) -> Self {
        Self { rsp_regs, rdp_regs }
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
    let result = if rd < 8 {
        debug!(
            "{:08X} MFC0 {}, {}",
            core.pc(),
            REGS[rt],
            RspRegisters::NAMES[rd]
        );

        core.cp0().rsp_regs.borrow().get(rd)
    } else {
        debug!(
            "{:08X} MFC0 {}, {}",
            core.pc(),
            REGS[rt],
            RdpRegisters::NAMES[rd - 8]
        );

        core.cp0().rdp_regs.borrow().get(rd - 8)
    };

    core.set(rt, result);
}

fn mtc0(core: &mut Core<impl Bus<Cp0 = Cp0>>, rt: usize, rd: usize) {
    let value = core.get(rt);

    if rd < 8 {
        debug!(
            "{:08X} MTC0 {}, {}",
            core.pc(),
            REGS[rt],
            RspRegisters::NAMES[rd]
        );

        core.cp0_mut().rsp_regs.borrow_mut().set(rd, value);
    } else {
        debug!(
            "{:08X} MTC0 {}, {}",
            core.pc(),
            REGS[rt],
            RdpRegisters::NAMES[rd - 8]
        );

        core.cp0_mut().rdp_regs.borrow_mut().set(rd - 8, value);
    }
}
