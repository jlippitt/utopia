use crate::core::mips::{Bus, Coprocessor2, Core, REGS};
use load::*;
use multiply::*;
use store::*;
use tracing::debug;
use vector::Vector;

mod load;
mod multiply;
mod store;
mod vector;

pub struct VectorUnit {
    regs: [Vector; 32],
    acc: [u64; 8],
}

impl VectorUnit {
    pub fn new() -> Self {
        Self {
            regs: [Vector::default(); 32],
            acc: [0; 8],
        }
    }

    fn get_v(&self, reg: usize) -> Vector {
        self.regs[reg]
    }

    fn get_h(&self, reg: usize, elem: usize) -> u16 {
        self.regs[reg].u16(elem)
    }

    fn get_d(&self, reg: usize, elem: usize) -> u64 {
        self.regs[reg].u64(elem)
    }

    fn get_q(&self, reg: usize, elem: usize) -> u128 {
        self.regs[reg].u128(elem)
    }

    fn set_v(&mut self, reg: usize, value: Vector) {
        self.regs[reg] = value;
        debug!("  $V{:02}: {}", reg, self.regs[reg]);
    }

    fn set_h(&mut self, reg: usize, elem: usize, value: u16) {
        self.regs[reg].set_u16(elem, value);
        debug!("  $V{:02}: {}", reg, self.regs[reg]);
    }

    fn set_d(&mut self, reg: usize, elem: usize, value: u64) {
        self.regs[reg].set_u64(elem, value);
        debug!("  $V{:02}: {}", reg, self.regs[reg]);
    }

    fn set_q(&mut self, reg: usize, elem: usize, value: u128) {
        self.regs[reg].set_u128(elem, value);
        debug!("  $V{:02}: {}", reg, self.regs[reg]);
    }

    fn accumulate(&mut self, lane: usize, value: u64) {
        self.acc[lane] = self.acc[lane].wrapping_add(value & 0xffff_ffff_ffff);
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

        core.set(rt, core.cp2().get_h(vs, elem) as u32);
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
        core.cp2_mut().set_h(vs, elem, value);
    }

    fn lwc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let base = ((word >> 21) & 31) as usize;
        let vt = ((word >> 16) & 31) as usize;
        let opcode = ((word >> 11) & 31) as usize;
        let elem = ((word >> 7) & 15) as usize;
        let offset = (((word & 127) as i32) << 25) >> 25;

        match opcode {
            0b00001 => lsv(core, base, vt, elem, offset),
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
            0b00001 => ssv(core, base, vt, elem, offset),
            0b00011 => sdv(core, base, vt, elem, offset),
            0b00100 => sqv(core, base, vt, elem, offset),
            _ => unimplemented!(
                "RSP SWC2 OP={:05b} ({:08X}: {:08X})",
                opcode,
                core.pc(),
                word
            ),
        }
    }

    fn cop2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let elem = ((word >> 21) & 15) as usize;
        let vt = ((word >> 16) & 31) as usize;
        let vs = ((word >> 11) & 31) as usize;
        let vd = ((word >> 6) & 31) as usize;

        match word & 63 {
            0o00 => vmulf(core, elem, vt, vs, vd),
            0o10 => vmacf(core, elem, vt, vs, vd),
            func => unimplemented!("RSP COP2 FN={:06b} ({:08X}: {:08X})", func, core.pc(), word),
        }
    }
}
