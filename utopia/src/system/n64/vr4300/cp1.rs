use super::super::mips::opcode::{IType, RType};
use super::super::mips::{self, Bus, Core, GPR};
use bitfield_struct::bitfield;
use tracing::trace;

mod arithmetic;
mod branch;
mod condition;
mod convert;
mod round;

const FCR: [&str; 32] = [
    "Revision", "FCR1", "FCR2", "FCR3", "FCR4", "FCR5", "FCR6", "FCR7", "FCR8", "FCR9", "FCR10",
    "FCR11", "FCR12", "FCR13", "FCR14", "FCR15", "FCR16", "FCR17", "FCR18", "FCR19", "FCR20",
    "FCR21", "FCR22", "FCR23", "FCR24", "FCR25", "FCR26", "FCR27", "FCR28", "FCR29", "FCR30",
    "Status",
];

#[bitfield(u32)]
pub struct Opcode {
    #[bits(6)]
    pub func: u32,
    #[bits(5)]
    pub fd: usize,
    #[bits(5)]
    pub fs: usize,
    #[bits(5)]
    pub ft: usize,
    #[bits(5)]
    pub fmt: u32,
    #[bits(6)]
    pub opcode: u32,
}

pub struct Cp1 {
    fr: bool,
    status: Status,
    regs: [u64; 32],
}

impl Cp1 {
    pub fn new() -> Self {
        Self {
            fr: true, // Set by IPL1
            status: Status::new(),
            regs: [0; 32],
        }
    }

    fn fcr(&self, reg: usize) -> u32 {
        match reg {
            31 => self.status.into(),
            _ => unimplemented!("VR4300 CP1 FCR Read: {} ({:?})", reg, FCR[reg]),
        }
    }

    fn set_fcr(&mut self, reg: usize, value: u32) {
        match reg {
            31 => {
                self.status = value.into();
                trace!("  Control/Status: {:?}", self.status);
            }
            _ => unimplemented!(
                "VR4300 CP1 FCR Write: {} ({:?}) <= {:08X}",
                reg,
                FCR[reg],
                value
            ),
        }
    }

    fn set_c(&mut self, value: bool) {
        self.status.set_c(value);
        trace!("  C: {}", value);
    }

    fn low(&self, reg: usize) -> u32 {
        self.regs[reg] as u32
    }

    fn set_low(&mut self, reg: usize, value: u32) {
        self.regs[reg] = (self.regs[reg] & 0xffff_ffff_0000_0000) | (value as u64);
    }

    fn high(&self, reg: usize) -> u32 {
        (self.regs[reg] >> 32) as u32
    }

    fn set_high(&mut self, reg: usize, value: u32) {
        self.regs[reg] = (self.regs[reg] & 0xffff_ffff) | ((value as u64) << 32);
    }

    fn gets(&self, reg: usize) -> f32 {
        f32::from_bits(if self.fr || (reg & 1) == 0 {
            self.low(reg)
        } else {
            self.high(reg & !1)
        })
    }

    fn getd(&self, reg: usize) -> f64 {
        f64::from_bits(if self.fr {
            self.regs[reg]
        } else {
            self.regs[reg & !1]
        })
    }

    fn getw(&self, reg: usize) -> i32 {
        if self.fr || (reg & 1) == 0 {
            self.low(reg) as i32
        } else {
            self.high(reg & !1) as i32
        }
    }

    fn getl(&self, reg: usize) -> i64 {
        if self.fr {
            self.regs[reg] as i64
        } else {
            self.regs[reg & !1] as i64
        }
    }

    fn sets(&mut self, reg: usize, value: f32) {
        let bits = value.to_bits();
        if self.fr || (reg & 1) == 0 {
            self.set_low(reg, bits);
        } else {
            self.set_high(reg & !1, bits);
        }
        trace!("  F{}.S: {}", reg, value);
    }

    fn setd(&mut self, reg: usize, value: f64) {
        let bits = value.to_bits();
        if self.fr {
            self.regs[reg] = bits;
        } else {
            self.regs[reg & !1] = bits;
        }
        trace!("  F{}.D: {}", reg, value);
    }

    fn setw(&mut self, reg: usize, value: i32) {
        if self.fr || (reg & 1) == 0 {
            self.set_low(reg, value as u32);
        } else {
            self.set_high(reg & !1, value as u32);
        }
        trace!("  F{}.W: {:08X}", reg, value);
    }

    fn setl(&mut self, reg: usize, value: i64) {
        if self.fr {
            self.regs[reg] = value as u64;
        } else {
            self.regs[reg & !1] = value as u64;
        }
        trace!("  F{}.L: {:016X}", reg, value);
    }
}

impl mips::Cp1 for Cp1 {
    fn set_fr(&mut self, fr: bool) {
        self.fr = fr;
        trace!("  Float Mode: {}", if self.fr { "Full" } else { "Half" });
    }

    fn mfc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = RType::from(word);
        trace!("{:08X} MFC1 {}, F{}", core.pc(), GPR[op.rt()], op.rd());
        let value = core.cp1().getw(op.rd());
        core.setw(op.rt(), value as u32);
    }

    fn mtc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = RType::from(word);
        trace!("{:08X} MTC1 {}, F{}", core.pc(), GPR[op.rt()], op.rd());
        let value = core.getw(op.rt());
        core.cp1_mut().setw(op.rd(), value as i32);
    }

    fn dmfc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = RType::from(word);
        trace!("{:08X} DMFC1 {}, F{}", core.pc(), GPR[op.rt()], op.rd());
        let value = core.cp1().getl(op.rd());
        core.setd(op.rt(), value as u64);
    }

    fn dmtc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = RType::from(word);
        trace!("{:08X} DMTC1 {}, F{}", core.pc(), GPR[op.rt()], op.rd());
        let value = core.getd(op.rt());
        core.cp1_mut().setl(op.rd(), value as i64);
    }

    fn cfc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = RType::from(word);
        trace!("{:08X} CFC1 {}, {}", core.pc(), GPR[op.rt()], FCR[op.rd()]);
        let value = core.cp1().fcr(op.rd());
        core.setw(op.rt(), value);
    }

    fn ctc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = RType::from(word);
        trace!("{:08X} CTC1 {}, {}", core.pc(), GPR[op.rt()], FCR[op.rd()]);
        let value = core.getw(op.rt());
        core.cp1_mut().set_fcr(op.rd(), value);
    }

    fn lwc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = IType::from(word);

        trace!(
            "{:08X} LWC1 F{}, {}({})",
            core.pc(),
            op.rt(),
            op.imm() as i16,
            GPR[op.rs()]
        );

        let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
        let value = core.read_u32(address);
        core.cp1_mut().setw(op.rt(), value as i32);
    }

    fn ldc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = IType::from(word);

        trace!(
            "{:08X} LDC1 F{}, {}({})",
            core.pc(),
            op.rt(),
            op.imm() as i16,
            GPR[op.rs()]
        );

        let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
        let value = core.read_u64(address);
        core.cp1_mut().setl(op.rt(), value as i64);
    }

    fn swc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = IType::from(word);

        trace!(
            "{:08X} SWC1 F{}, {}({})",
            core.pc(),
            op.rt(),
            op.imm() as i16,
            GPR[op.rs()]
        );

        let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
        core.write_u32(address, core.cp1().getw(op.rt()) as u32);
    }

    fn sdc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        let op = IType::from(word);

        trace!(
            "{:08X} SDC1 F{}, {}({})",
            core.pc(),
            op.rt(),
            op.imm() as i16,
            GPR[op.rs()]
        );

        let address = core.getw(op.rs()).wrapping_add(op.imm() as i16 as u32);
        core.write_u64(address, core.cp1().getl(op.rt()) as u64);
    }

    fn cop1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        match (word >> 21) & 0o37 {
            0o20 => fmt_s(core, word),
            0o21 => fmt_d(core, word),
            0o24 => fmt_w(core, word),
            0o25 => fmt_l(core, word),
            opcode => unimplemented!("R4300 COP1 Opcode {:02o} [PC:{:08X}]", opcode, core.pc()),
        }
    }

    fn bc1(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
        match (word >> 16) & 0o37 {
            0o00 => branch::bc1f::<false>(core, word),
            0o01 => branch::bc1t::<false>(core, word),
            0o02 => branch::bc1f::<true>(core, word),
            0o03 => branch::bc1t::<true>(core, word),
            opcode => unimplemented!("R4300 COP1 Opcode {:02o} [PC:{:08X}]", opcode, core.pc()),
        }
    }
}

fn fmt_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    match word & 0o77 {
        0o00 => arithmetic::add_s(core, word),
        0o01 => arithmetic::sub_s(core, word),
        0o02 => arithmetic::mul_s(core, word),
        0o03 => arithmetic::div_s(core, word),
        0o04 => arithmetic::sqrt_s(core, word),
        0o05 => arithmetic::abs_s(core, word),
        0o06 => arithmetic::mov_s(core, word),
        0o07 => arithmetic::neg_s(core, word),
        0o10 => round::round_l_s(core, word),
        0o11 => round::trunc_l_s(core, word),
        0o12 => round::ceil_l_s(core, word),
        0o13 => round::floor_l_s(core, word),
        0o14 => round::round_w_s(core, word),
        0o15 => round::trunc_w_s(core, word),
        0o16 => round::ceil_w_s(core, word),
        0o17 => round::floor_w_s(core, word),
        0o41 => convert::cvt_d_s(core, word),
        0o44 => convert::cvt_w_s(core, word),
        0o45 => convert::cvt_l_s(core, word),
        0o60 => condition::c_s::<0>(core, word),
        0o61 => condition::c_s::<1>(core, word),
        0o62 => condition::c_s::<2>(core, word),
        0o63 => condition::c_s::<3>(core, word),
        0o64 => condition::c_s::<4>(core, word),
        0o65 => condition::c_s::<5>(core, word),
        0o66 => condition::c_s::<6>(core, word),
        0o67 => condition::c_s::<7>(core, word),
        0o70 => condition::c_s::<8>(core, word),
        0o71 => condition::c_s::<9>(core, word),
        0o72 => condition::c_s::<10>(core, word),
        0o73 => condition::c_s::<11>(core, word),
        0o74 => condition::c_s::<12>(core, word),
        0o75 => condition::c_s::<13>(core, word),
        0o76 => condition::c_s::<14>(core, word),
        0o77 => condition::c_s::<15>(core, word),
        func => unimplemented!(
            "R4300 COP1 Function (FMT=S) {:02o} [PC:{:08X}]",
            func,
            core.pc()
        ),
    }
}

fn fmt_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    match word & 0o77 {
        0o00 => arithmetic::add_d(core, word),
        0o01 => arithmetic::sub_d(core, word),
        0o02 => arithmetic::mul_d(core, word),
        0o03 => arithmetic::div_d(core, word),
        0o04 => arithmetic::sqrt_d(core, word),
        0o05 => arithmetic::abs_d(core, word),
        0o06 => arithmetic::mov_d(core, word),
        0o07 => arithmetic::neg_d(core, word),
        0o10 => round::round_l_d(core, word),
        0o11 => round::trunc_l_d(core, word),
        0o12 => round::ceil_l_d(core, word),
        0o13 => round::floor_l_d(core, word),
        0o14 => round::round_w_d(core, word),
        0o15 => round::trunc_w_d(core, word),
        0o16 => round::ceil_w_d(core, word),
        0o17 => round::floor_w_d(core, word),
        0o40 => convert::cvt_s_d(core, word),
        0o44 => convert::cvt_w_d(core, word),
        0o45 => convert::cvt_l_d(core, word),
        0o60 => condition::c_d::<0>(core, word),
        0o61 => condition::c_d::<1>(core, word),
        0o62 => condition::c_d::<2>(core, word),
        0o63 => condition::c_d::<3>(core, word),
        0o64 => condition::c_d::<4>(core, word),
        0o65 => condition::c_d::<5>(core, word),
        0o66 => condition::c_d::<6>(core, word),
        0o67 => condition::c_d::<7>(core, word),
        0o70 => condition::c_d::<8>(core, word),
        0o71 => condition::c_d::<9>(core, word),
        0o72 => condition::c_d::<10>(core, word),
        0o73 => condition::c_d::<11>(core, word),
        0o74 => condition::c_d::<12>(core, word),
        0o75 => condition::c_d::<13>(core, word),
        0o76 => condition::c_d::<14>(core, word),
        0o77 => condition::c_d::<15>(core, word),
        func => unimplemented!(
            "R4300 COP1 Function (FMT=D) {:02o} [PC:{:08X}]",
            func,
            core.pc()
        ),
    }
}

fn fmt_w(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    match word & 0o77 {
        0o40 => convert::cvt_s_w(core, word),
        0o41 => convert::cvt_d_w(core, word),
        func => unimplemented!(
            "R4300 COP1 Function (FMT=W) {:02o} [PC:{:08X}]",
            func,
            core.pc()
        ),
    }
}

fn fmt_l(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    match word & 0o77 {
        0o40 => convert::cvt_s_l(core, word),
        0o41 => convert::cvt_d_l(core, word),
        func => unimplemented!(
            "R4300 COP1 Function (FMT=L) {:02o} [PC:{:08X}]",
            func,
            core.pc()
        ),
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RoundingMode {
    Round = 0,
    Trunc = 1,
    Ceil = 2,
    Floor = 3,
}

impl RoundingMode {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 3 {
            0 => Self::Round,
            1 => Self::Trunc,
            2 => Self::Ceil,
            3 => Self::Floor,
            _ => unreachable!(),
        }
    }
}

#[bitfield(u32)]
struct Status {
    #[bits(2)]
    rm: RoundingMode,
    #[bits(5)]
    flags: u32,
    #[bits(5)]
    enables: u32,
    #[bits(6)]
    cause: u32,
    #[bits(5)]
    __: u32,
    c: bool,
    fs: bool,
    #[bits(7)]
    __: u32,
}
