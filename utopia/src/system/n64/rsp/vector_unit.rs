use crate::core::mips::{Bus, Coprocessor2, Core, REGS};
use load::*;
use store::*;
use tracing::debug;

mod load;
mod store;

pub struct VectorUnit {
    regs: [[u16; 8]; 32],
}

impl VectorUnit {
    pub fn new() -> Self {
        Self { regs: [[0; 8]; 32] }
    }

    fn geth(&self, reg: usize, elem: usize) -> u16 {
        debug_assert!((elem & 1) == 0);
        self.regs[reg][elem >> 1]
    }

    fn getd(&self, reg: usize, elem: usize) -> u64 {
        debug_assert!((elem & 1) == 0);
        let mut value = 0;
        value |= (self.regs[reg][elem >> 1] as u64) << 48;
        value |= (self.regs[reg][(elem >> 1) + 1] as u64) << 32;
        value |= (self.regs[reg][(elem >> 1) + 2] as u64) << 16;
        value |= self.regs[reg][(elem >> 1) + 3] as u64;
        value
    }

    fn seth(&mut self, reg: usize, elem: usize, value: u16) {
        debug_assert!((elem & 1) == 0);

        self.regs[reg][elem >> 1] = value;

        debug!(
            "  $V{:02}: {:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X}",
            reg,
            self.regs[reg][0],
            self.regs[reg][1],
            self.regs[reg][2],
            self.regs[reg][3],
            self.regs[reg][4],
            self.regs[reg][5],
            self.regs[reg][6],
            self.regs[reg][7],
        )
    }

    fn setd(&mut self, reg: usize, elem: usize, value: u64) {
        debug_assert!((elem & 1) == 0);

        self.regs[reg][elem >> 1] = (value >> 48) as u16;
        self.regs[reg][(elem >> 1) + 1] = (value >> 32) as u16;
        self.regs[reg][(elem >> 1) + 2] = (value >> 16) as u16;
        self.regs[reg][(elem >> 1) + 3] = value as u16;

        debug!(
            "  $V{:02}: {:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X}",
            reg,
            self.regs[reg][0],
            self.regs[reg][1],
            self.regs[reg][2],
            self.regs[reg][3],
            self.regs[reg][4],
            self.regs[reg][5],
            self.regs[reg][6],
            self.regs[reg][7],
        )
    }

    fn setq(&mut self, reg: usize, elem: usize, value: u128) {
        debug_assert!((elem & 1) == 0);

        self.regs[reg][elem >> 1] = (value >> 112) as u16;
        self.regs[reg][(elem >> 1) + 1] = (value >> 96) as u16;
        self.regs[reg][(elem >> 1) + 2] = (value >> 80) as u16;
        self.regs[reg][(elem >> 1) + 3] = (value >> 64) as u16;
        self.regs[reg][(elem >> 1) + 4] = (value >> 48) as u16;
        self.regs[reg][(elem >> 1) + 5] = (value >> 32) as u16;
        self.regs[reg][(elem >> 1) + 6] = (value >> 16) as u16;
        self.regs[reg][(elem >> 1) + 7] = value as u16;

        debug!(
            "  $V{:02}: {:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X}",
            reg,
            self.regs[reg][0],
            self.regs[reg][1],
            self.regs[reg][2],
            self.regs[reg][3],
            self.regs[reg][4],
            self.regs[reg][5],
            self.regs[reg][6],
            self.regs[reg][7],
        )
    }
}

impl Coprocessor2 for VectorUnit {
    fn mfc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let vs = ((word >> 11) & 31) as usize;
        let elem = ((word >> 7) & 15) as usize;

        debug!(
            "{:08X} MFC2 {}, $V{:02},E({})",
            core.pc(),
            REGS[rt],
            vs,
            elem >> 1
        );

        core.set(rt, core.cp2().geth(vs, elem) as u32);
    }

    fn mtc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let vs = ((word >> 11) & 31) as usize;
        let elem = ((word >> 7) & 15) as usize;

        debug!(
            "{:08X} MTC2 {}, $V{:02},E({})",
            core.pc(),
            REGS[rt],
            vs,
            elem >> 1
        );

        let value = core.get(rt) as u16;
        core.cp2_mut().seth(vs, elem, value);
    }

    fn lwc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let base = ((word >> 21) & 31) as usize;
        let vt = ((word >> 16) & 31) as usize;
        let opcode = ((word >> 11) & 31) as usize;
        let elem = ((word >> 7) & 15) as usize;
        let offset = (((word & 127) as i32) << 25) >> 25;

        match opcode {
            0b00011 => ldv(core, base, vt, elem, offset),
            0b00100 => lqv(core, base, vt, elem, offset),
            _ => unimplemented!(
                "RSP LWC2 OP={:05b} ({:08X}: {:08X})",
                opcode,
                core.pc(),
                word
            ),
        }
    }

    fn swc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let base = ((word >> 21) & 31) as usize;
        let vt = ((word >> 16) & 31) as usize;
        let opcode = ((word >> 11) & 31) as usize;
        let elem = ((word >> 7) & 15) as usize;
        let offset = (((word & 127) as i32) << 25) >> 25;

        match opcode {
            0b00011 => sdv(core, base, vt, elem, offset),
            _ => unimplemented!(
                "RSP SWC2 OP={:05b} ({:08X}: {:08X})",
                opcode,
                core.pc(),
                word
            ),
        }
    }

    fn cop2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        match word & 63 {
            func => unimplemented!("RSP COP2 FN={:06b} ({:08X}: {:08X})", func, core.pc(), word),
        }
    }
}
