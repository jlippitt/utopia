use super::{Bus, Core, REGS};
use arithmetic::*;
use compare::*;
use control::*;
use convert::*;
use num_derive::{FromPrimitive, ToPrimitive};
use round::*;
use std::fmt;
use tracing::debug;
use transfer::*;

mod arithmetic;
mod compare;
mod control;
mod convert;
mod round;
mod transfer;

#[derive(Copy, Clone, Default)]
struct Flags {
    e: bool,
    v: bool,
    z: bool,
    o: bool,
    u: bool,
    i: bool,
}

#[derive(Default, Debug, FromPrimitive, ToPrimitive)]
enum RoundingMode {
    #[default]
    Nearest,
    Zero,
    PlusInfinity,
    MinusInfinity,
}

#[derive(Default)]
struct Control {
    fs: bool,
    c: bool,
    cause: Flags,
    enable: Flags,
    flags: Flags,
    rm: RoundingMode,
}

#[derive(Copy, Clone)]
union Register {
    s: f32,
    d: f64,
    w: i32,
    l: i64,
}

pub struct Cp1 {
    regs: [Register; 32],
    ctrl: Control,
    reg_size: bool,
}

impl Cp1 {
    pub fn new() -> Self {
        Self {
            regs: [Register { l: 0 }; 32],
            ctrl: Control::default(),
            reg_size: false,
        }
    }

    pub fn set_reg_size(&mut self, reg_size: bool) {
        self.reg_size = reg_size;
    }

    fn s(&self, reg: usize) -> f32 {
        if !self.reg_size && (reg & 1) != 0 {
            panic!("Tried to get odd-numbered CP1 register when FR=0");
        }

        unsafe { self.regs[reg].s }
    }

    fn d(&self, reg: usize) -> f64 {
        if self.reg_size {
            unsafe { self.regs[reg].d }
        } else if (reg & 1) == 0 {
            let low = unsafe { self.regs[reg].w.to_le_bytes() };
            let high = unsafe { self.regs[reg + 1].w.to_le_bytes() };
            f64::from_le_bytes([
                low[0], low[1], low[2], low[3], high[0], high[1], high[2], high[3],
            ])
        } else {
            panic!("Tried to get odd-numbered CP1 register when FR=0");
        }
    }

    fn w(&self, reg: usize) -> i32 {
        unsafe { self.regs[reg].w }
    }

    fn set_c(&mut self, value: bool) {
        self.ctrl.c = value;
        debug!("  C = {}", value as u32);
    }

    fn set_s(&mut self, reg: usize, value: f32) {
        if !self.reg_size && (reg & 1) != 0 {
            panic!("Tried to set odd-numbered CP1 register when FR=0");
        }

        self.regs[reg].s = value;
        debug!("  $F{}.S = {}", reg, value);
    }

    fn set_d(&mut self, reg: usize, value: f64) {
        if self.reg_size {
            self.regs[reg].d = value;
        } else if (reg & 1) == 0 {
            let bytes = value.to_le_bytes();
            self.regs[reg].w = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            self.regs[reg + 1].w = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        } else {
            panic!("Tried to set odd-numbered CP1 register when FR=0");
        }

        debug!("  $F{}.D = {}", reg, value);
    }

    fn set_w(&mut self, reg: usize, value: i32) {
        self.regs[reg].w = value;
        debug!("  $F{}.W = {}", reg, value);
    }
}

impl From<u32> for Flags {
    fn from(value: u32) -> Self {
        Self {
            e: (value & 0x20) != 0,
            v: (value & 0x10) != 0,
            z: (value & 0x08) != 0,
            o: (value & 0x04) != 0,
            u: (value & 0x02) != 0,
            i: (value & 0x01) != 0,
        }
    }
}

impl Into<u32> for Flags {
    fn into(self) -> u32 {
        let mut value = 0;
        value |= if self.e { 0x20 } else { 0 };
        value |= if self.v { 0x10 } else { 0 };
        value |= if self.z { 0x08 } else { 0 };
        value |= if self.o { 0x04 } else { 0 };
        value |= if self.u { 0x02 } else { 0 };
        value |= if self.i { 0x01 } else { 0 };
        value
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}{}{}",
            if self.e { 'E' } else { '-' },
            if self.v { 'V' } else { '-' },
            if self.z { 'Z' } else { '-' },
            if self.o { 'O' } else { '-' },
            if self.u { 'U' } else { '-' },
            if self.i { 'I' } else { '-' },
        )
    }
}

pub fn lwc1(core: &mut Core<impl Bus>, base: usize, ft: usize, value: u32) {
    debug!(
        "{:08X} LWC1 $F{}, {}({})",
        core.pc, ft, value as i16, REGS[base]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(base).wrapping_add(ivalue);
    let result = core.read_word(address);
    core.cp1.set_w(ft, result as i32);
}

pub fn cop1(core: &mut Core<impl Bus>, word: u32) {
    match (word >> 21) & 31 {
        0b00010 => type_r(core, cfc1, word),
        0b00100 => type_r(core, mtc1, word),
        0b00110 => type_r(core, ctc1, word),
        0b01000 => format_b(core, word),
        0b10000 => format_s(core, word),
        0b10001 => format_d(core, word),
        0b10100 => format_w(core, word),
        rs => unimplemented!("CP1 RS={:05b} ({:08X}: {:08X})", rs, core.pc, word),
    }
}

fn format_b(core: &mut Core<impl Bus>, word: u32) {
    match (word >> 16) & 31 {
        0b00000 => branch::<false, false>(core, word),
        0b00001 => branch::<true, false>(core, word),
        0b00010 => branch::<false, true>(core, word),
        0b00011 => branch::<true, true>(core, word),
        cond => unimplemented!("CP1.B COND={:05b} ({:08X}: {:08X})", cond, core.pc, word),
    }
}

fn format_s(core: &mut Core<impl Bus>, word: u32) {
    match word & 0o77 {
        0o00 => type_f(core, add_s, word),
        0o01 => type_f(core, sub_s, word),
        0o02 => type_f(core, mul_s, word),
        0o03 => type_f(core, div_s, word),
        0o15 => type_f(core, trunc_w_s, word),
        0o76 => type_f(core, c_le_s, word),
        func => unimplemented!("CP1.W FN={:02o} ({:08X}: {:08X})", func, core.pc, word),
    }
}

fn format_d(core: &mut Core<impl Bus>, word: u32) {
    match word & 0o77 {
        0o40 => type_f(core, cvt_s_d, word),
        func => unimplemented!("CP1.W FN={:02o} ({:08X}: {:08X})", func, core.pc, word),
    }
}

fn format_w(core: &mut Core<impl Bus>, word: u32) {
    match word & 0o77 {
        0o40 => type_f(core, cvt_s_w, word),
        0o41 => type_f(core, cvt_d_w, word),
        func => unimplemented!("CP1.W FN={:02o} ({:08X}: {:08X})", func, core.pc, word),
    }
}

fn type_r<T: Bus>(core: &mut Core<T>, instr: impl Fn(&mut Core<T>, usize, usize), word: u32) {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    instr(core, rt, rd);
}

fn type_f<T: Bus>(
    core: &mut Core<T>,
    instr: impl Fn(&mut Core<T>, usize, usize, usize),
    word: u32,
) {
    let ft = ((word >> 16) & 31) as usize;
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    instr(core, ft, fs, fd);
}
