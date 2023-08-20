use super::super::rdp::Registers as RdpRegisters;
use super::registers::Registers as RspRegisters;
use crate::core::mips::{Bus, Coprocessor0, Core};
use std::cell::RefCell;
use std::rc::Rc;

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
    #[rustfmt::skip]
    const REGS: [&'static str; 32] = [
        RspRegisters::NAMES[0],
        RspRegisters::NAMES[1],
        RspRegisters::NAMES[2],
        RspRegisters::NAMES[3],
        RspRegisters::NAMES[4],
        RspRegisters::NAMES[5],
        RspRegisters::NAMES[6],
        RspRegisters::NAMES[7],
        RdpRegisters::NAMES[0],
        RdpRegisters::NAMES[1],
        RdpRegisters::NAMES[2],
        RdpRegisters::NAMES[3],
        RdpRegisters::NAMES[4],
        RdpRegisters::NAMES[5],
        RdpRegisters::NAMES[6],
        RdpRegisters::NAMES[7],
        "$C16", "$C17", "$C18", "$C19", "$C20", "$C21", "$C22", "$C23",
        "$C24", "$C25", "$C26", "$C27", "$C28", "$C29", "$C30", "$C31",
    ];

    fn get(core: &Core<impl Bus<Cp0 = Self>>, index: usize) -> u32 {
        let cp0 = &core.cp0();

        if index < 8 {
            cp0.rsp_regs.borrow().get(index)
        } else if index < 16 {
            cp0.rdp_regs.borrow().get(index - 8)
        } else {
            panic!("Invalid RSP CP0 register: {}", Self::REGS[index]);
        }
    }

    fn set(core: &mut Core<impl Bus<Cp0 = Self>>, index: usize, value: u32) {
        let cp0 = &mut core.cp0_mut();

        if index < 8 {
            cp0.rsp_regs.borrow_mut().set(index, value);
        } else if index < 16 {
            cp0.rdp_regs.borrow_mut().set(index - 8, value);
        } else {
            panic!("Invalid RSP CP0 register: {}", Self::REGS[index]);
        }
    }

    fn dispatch(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        match word & 63 {
            func => unimplemented!(
                "RSP CP0 RS=10000 FN={:06b} ({:08X}: {:08X})",
                func,
                core.pc(),
                word
            ),
        }
    }
}
