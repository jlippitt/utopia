use super::{Bus, Core};
use num_derive::{FromPrimitive, ToPrimitive};
use std::fmt;
use tracing::debug;
use transfer::*;

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
    f: f32,
    d: f64,
    w: u32,
    l: u64,
}

pub struct Cp1 {
    ctrl: Control,
    regs: [Register; 32],
}

impl Cp1 {
    pub fn new() -> Self {
        Self {
            ctrl: Control::default(),
            regs: [Register { l: 0 }; 32],
        }
    }

    pub fn setw(&mut self, reg: usize, value: u32) {
        self.regs[reg].w = value;
        debug!("  $F{}.W = {:08X}", reg, value);
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

pub fn cop1(core: &mut Core<impl Bus>, word: u32) {
    match (word >> 21) & 31 {
        0b00010 => type_r(core, cfc1, word),
        0b00100 => type_r(core, mtc1, word),
        0b00110 => type_r(core, ctc1, word),
        rs => unimplemented!("CP1 RS={:05b} ({:08X}: {:08X})", rs, core.pc, word),
    }
}

fn type_r<T: Bus>(core: &mut Core<T>, instr: impl Fn(&mut Core<T>, usize, usize), word: u32) {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    instr(core, rt, rd);
}
