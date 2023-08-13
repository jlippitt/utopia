use super::{Bus, Core, REGS};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::fmt;
use tracing::debug;

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

#[derive(Default)]
pub struct Cop1 {
    ctrl: Control,
}

pub fn dispatch(core: &mut Core<impl Bus>, word: u32) {
    match (word >> 21) & 31 {
        0b00010 => type_r(core, cfc1, word),
        0b00110 => type_r(core, ctc1, word),
        rs => unimplemented!("COP1 RS={:05b} ({:08X}: {:08X})", rs, core.pc, word),
    }
}

fn type_r<T: Bus>(core: &mut Core<T>, instr: impl Fn(&mut Core<T>, usize, usize), word: u32) {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    instr(core, rt, rd);
}

fn cfc1(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} CFC1 {}, ${}", core.pc, REGS[rt], rd);

    let result = match rd {
        31 => {
            // CONTROL/STATUS
            let ctrl = &core.cop1.ctrl;
            let mut value = 0;
            value |= ctrl.rm.to_u32().unwrap();
            value |= Into::<u32>::into(ctrl.flags) << 2;
            value |= Into::<u32>::into(ctrl.enable) << 7;
            value |= Into::<u32>::into(ctrl.cause) << 12;
            value |= (ctrl.c as u32) << 23;
            value |= (ctrl.fs as u32) << 24;
            value
        }
        _ => todo!("COP1 Register Read: ${}", rd),
    };

    core.set(rt, result);
}

fn ctc1(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} CTC1 {}, ${}", core.pc, REGS[rt], rd);

    let value = core.get(rt);

    match rd {
        31 => {
            // CONTROL/STATUS
            let ctrl = &mut core.cop1.ctrl;
            ctrl.rm = RoundingMode::from_u32(value & 3).unwrap();
            ctrl.flags = Flags::from((value >> 2) & 31);
            ctrl.enable = Flags::from((value >> 7) & 31);
            ctrl.cause = Flags::from((value >> 12) & 63);
            ctrl.c = (value & 0x0080_0000) != 0;
            ctrl.fs = (value & 0x0100_0000) != 0;
            debug!("  COP1 Rounding Mode: {:?}", ctrl.rm);
            debug!("  COP1 Flags: {}", ctrl.flags);
            debug!("  COP1 Enable: {}", ctrl.enable);
            debug!("  COP1 Cause: {}", ctrl.cause);
            debug!("  COP1 Compare: {}", ctrl.c);
            debug!("  COP1 Flash: {}", ctrl.fs);
        }
        _ => todo!("COP1 Register Write: ${} <= {:08X}", rd, value),
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
