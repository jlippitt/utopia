use super::super::{Bus, Core};

pub trait ModifyOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8;
}

pub struct Asl;

impl ModifyOperator for Asl {
    const NAME: &'static str = "ASL";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        core.flags.c = (value & 0x80) != 0;
        let result = value << 1;
        core.set_nz(result);
        result
    }
}

pub struct Rol;

impl ModifyOperator for Rol {
    const NAME: &'static str = "ROL";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let carry = core.flags.c as u8;
        core.flags.c = (value & 0x80) != 0;
        let result = (value << 1) | carry;
        core.set_nz(result);
        result
    }
}

pub struct Lsr;

impl ModifyOperator for Lsr {
    const NAME: &'static str = "LSR";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        core.flags.c = (value & 0x01) != 0;
        let result = value >> 1;
        core.set_nz(result);
        result
    }
}

pub struct Ror;

impl ModifyOperator for Ror {
    const NAME: &'static str = "ROR";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let carry = core.flags.c as u8;
        core.flags.c = (value & 0x01) != 0;
        let result = (value >> 1) | (carry << 7);
        core.set_nz(result);
        result
    }
}

pub struct Dec;

impl ModifyOperator for Dec {
    const NAME: &'static str = "DEC";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        core.set_nz(result);
        result
    }
}

pub struct Inc;

impl ModifyOperator for Inc {
    const NAME: &'static str = "INC";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        core.set_nz(result);
        result
    }
}
