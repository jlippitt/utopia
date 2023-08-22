use super::registers::Registers;
use crate::core::mips::{Bus, Coprocessor0, Core, REGS};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::debug;

pub struct Cp0 {
    regs: Rc<RefCell<Registers>>,
}

impl Cp0 {
    pub fn new(regs: Rc<RefCell<Registers>>) -> Self {
        Self { regs }
    }
}

impl Coprocessor0 for Cp0 {
    fn mfc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;

        let result = if rd < 16 {
            debug!(
                "{:08X} MFC0 {}, {}",
                core.pc(),
                REGS[rt],
                Registers::NAMES[rd]
            );
            core.cp0().regs.borrow().get(rd)
        } else {
            panic!("Invalid RSP CP0 register: $C{}", rd);
        };

        core.set(rt, result);
    }

    fn mtc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;
        let value = core.get(rt);

        if rd < 16 {
            debug!(
                "{:08X} MTC0 {}, {}",
                core.pc(),
                REGS[rt],
                Registers::NAMES[rd]
            );
            core.cp0_mut().regs.borrow_mut().set(rd, value);
        } else {
            panic!("Invalid RSP CP0 register: $C{}", rd);
        }
    }

    fn cop0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        match word & 63 {
            func => unimplemented!("RSP COP0 FN={:06b} ({:08X}: {:08X})", func, core.pc(), word),
        }
    }

    fn break_(core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        debug!("{:08X} BREAK", core.pc());
        core.cp0_mut().regs.borrow_mut().break_();
    }
}
