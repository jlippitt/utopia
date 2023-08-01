use super::{Bus, Core};
use num_derive::FromPrimitive;
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum Condition {
    EQ = 0,
    NE = 1,
    CS = 2,
    CC = 3,
    MI = 4,
    PL = 5,
    VS = 6,
    VC = 7,
    HI = 8,
    LS = 9,
    GE = 10,
    LT = 11,
    GT = 12,
    LE = 13,
    AL = 14,
}

impl Condition {
    pub fn apply(&self, core: &Core<impl Bus>) -> bool {
        let cpsr = &core.cpsr;

        match self {
            Self::EQ => cpsr.z,
            Self::NE => !cpsr.z,
            Self::CS => cpsr.c,
            Self::CC => !cpsr.c,
            Self::MI => cpsr.n,
            Self::PL => !cpsr.n,
            Self::VS => cpsr.v,
            Self::VC => !cpsr.v,
            Self::HI => !cpsr.z && cpsr.c,
            Self::LS => cpsr.z || !cpsr.c,
            Self::GE => cpsr.n == cpsr.v,
            Self::LT => cpsr.n != cpsr.v,
            Self::GT => !cpsr.z && cpsr.n == cpsr.v,
            Self::LE => cpsr.z || cpsr.n != cpsr.z,
            Self::AL => true,
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::EQ => "EQ",
                Self::NE => "NE",
                Self::CS => "CS",
                Self::CC => "CC",
                Self::MI => "MI",
                Self::PL => "PL",
                Self::VS => "VS",
                Self::VC => "VC",
                Self::HI => "HI",
                Self::LS => "LS",
                Self::GE => "GE",
                Self::LT => "LT",
                Self::GT => "GT",
                Self::LE => "LE",
                Self::AL => "",
            }
        )
    }
}
