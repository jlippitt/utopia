use super::super::{Bus, Core};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpType {
    Binary,
    Move,
    Compare,
}

pub trait AluOperator {
    const NAME: &'static str;
    const LOGICAL: bool;
    const TYPE: OpType;
    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32);
}

pub struct Mov;

impl AluOperator for Mov {
    const NAME: &'static str = "MOV";
    const LOGICAL: bool = true;
    const TYPE: OpType = OpType::Move;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, _lhs: u32, rhs: u32) {
        if SET_FLAGS {
            core.set_nz(rhs);
        }

        core.set(rd, rhs);
    }
}

// LOGIC OPERATORS

pub struct Mvn;

impl AluOperator for Mvn {
    const NAME: &'static str = "MVN";
    const LOGICAL: bool = true;
    const TYPE: OpType = OpType::Move;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, _lhs: u32, rhs: u32) {
        let result = !rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        core.set(rd, result);
    }
}

pub struct And;

impl AluOperator for And {
    const NAME: &'static str = "AND";
    const LOGICAL: bool = true;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = lhs & rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        core.set(rd, result)
    }
}

pub struct Eor;

impl AluOperator for Eor {
    const NAME: &'static str = "EOR";
    const LOGICAL: bool = true;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = lhs ^ rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        core.set(rd, result)
    }
}

pub struct Orr;

impl AluOperator for Orr {
    const NAME: &'static str = "ORR";
    const LOGICAL: bool = true;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = lhs | rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        core.set(rd, result)
    }
}

pub struct Bic;

impl AluOperator for Bic {
    const NAME: &'static str = "BIC";
    const LOGICAL: bool = true;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = lhs & !rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        core.set(rd, result)
    }
}
pub struct Tst;

impl AluOperator for Tst {
    const NAME: &'static str = "TST";
    const LOGICAL: bool = true;
    const TYPE: OpType = OpType::Compare;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, _rd: usize, lhs: u32, rhs: u32) {
        core.set_nz(lhs & rhs);
    }
}

pub struct Teq;

impl AluOperator for Teq {
    const NAME: &'static str = "TEQ";
    const LOGICAL: bool = true;
    const TYPE: OpType = OpType::Compare;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, _rd: usize, lhs: u32, rhs: u32) {
        core.set_nz(lhs ^ rhs);
    }
}

// ARITHMETIC OPERATORS

pub struct Add;

impl AluOperator for Add {
    const NAME: &'static str = "ADD";
    const LOGICAL: bool = false;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = core.add_with_carry::<SET_FLAGS>(lhs, rhs, false);
        core.set(rd, result);
    }
}

pub struct Adc;

impl AluOperator for Adc {
    const NAME: &'static str = "ADC";
    const LOGICAL: bool = false;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = core.add_with_carry::<SET_FLAGS>(lhs, rhs, core.cpsr.c);
        core.set(rd, result);
    }
}

pub struct Sub;

impl AluOperator for Sub {
    const NAME: &'static str = "SUB";
    const LOGICAL: bool = false;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = core.add_with_carry::<SET_FLAGS>(lhs, !rhs, true);
        core.set(rd, result);
    }
}

pub struct Sbc;

impl AluOperator for Sbc {
    const NAME: &'static str = "SBC";
    const LOGICAL: bool = false;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = core.add_with_carry::<SET_FLAGS>(lhs, !rhs, core.cpsr.c);
        core.set(rd, result);
    }
}

pub struct Rsb;

impl AluOperator for Rsb {
    const NAME: &'static str = "RSB";
    const LOGICAL: bool = false;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = core.add_with_carry::<SET_FLAGS>(rhs, !lhs, true);
        core.set(rd, result);
    }
}

pub struct Rsc;

impl AluOperator for Rsc {
    const NAME: &'static str = "RSC";
    const LOGICAL: bool = false;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = core.add_with_carry::<SET_FLAGS>(rhs, !lhs, core.cpsr.c);
        core.set(rd, result);
    }
}

pub struct Cmp;

impl AluOperator for Cmp {
    const NAME: &'static str = "CMP";
    const LOGICAL: bool = false;
    const TYPE: OpType = OpType::Compare;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, _rd: usize, lhs: u32, rhs: u32) {
        core.add_with_carry::<true>(lhs, !rhs, true);
    }
}

pub struct Cmn;

impl AluOperator for Cmn {
    const NAME: &'static str = "CMN";
    const LOGICAL: bool = false;
    const TYPE: OpType = OpType::Compare;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, _rd: usize, lhs: u32, rhs: u32) {
        core.add_with_carry::<true>(lhs, rhs, false);
    }
}

pub struct Mul;

impl AluOperator for Mul {
    const NAME: &'static str = "MUL";
    const LOGICAL: bool = false;
    const TYPE: OpType = OpType::Binary;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, rd: usize, lhs: u32, rhs: u32) {
        let result = lhs * rhs;

        if SET_FLAGS {
            core.set_nz(result);
            // TODO: Carry is set to meaningless value?
        }

        core.set(rd, result);
    }
}
