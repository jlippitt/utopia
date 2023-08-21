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
    fn mfc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;

        let result = if rd < 8 {
            debug!(
                "{:08X} MFC0 {}, {}",
                core.pc(),
                REGS[rt],
                RspRegisters::NAMES[rd]
            );
            core.cp0().rsp_regs.borrow().get(rd)
        } else if rd < 16 {
            debug!(
                "{:08X} MFC0 {}, {}",
                core.pc(),
                REGS[rt],
                RdpRegisters::NAMES[rd - 8]
            );
            core.cp0().rdp_regs.borrow().get(rd - 8)
        } else {
            panic!("Invalid RSP CP0 register: $C{}", rd);
        };

        core.set(rt, result);
    }

    fn mtc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;
        let value = core.get(rt);

        if rd < 8 {
            debug!(
                "{:08X} MTC0 {}, {}",
                core.pc(),
                REGS[rt],
                RspRegisters::NAMES[rd]
            );
            core.cp0_mut().rsp_regs.borrow_mut().set(rd, value);
        } else if rd < 16 {
            debug!(
                "{:08X} MTC0 {}, {}",
                core.pc(),
                REGS[rt],
                RdpRegisters::NAMES[rd - 8]
            );
            core.cp0_mut().rdp_regs.borrow_mut().set(rd - 8, value);
        } else {
            panic!("Invalid RSP CP0 register: $C{}", rd);
        }
    }

    fn cop0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        match word & 63 {
            func => unimplemented!("RSP COP0 FN={:06b} ({:08X}: {:08X})", func, core.pc(), word),
        }
    }
}
