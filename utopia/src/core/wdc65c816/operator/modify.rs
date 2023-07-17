use super::super::{Bus, Core};

pub trait ModifyOperator {
    const NAME: &'static str;
    fn apply8(core: &mut Core<impl Bus>, value: u8) -> u8;
    fn apply16(core: &mut Core<impl Bus>, value: u16) -> u16;
}

pub struct Asl;

impl ModifyOperator for Asl {
    const NAME: &'static str = "ASL";

    fn apply8(core: &mut Core<impl Bus>, value: u8) -> u8 {
        core.flags.c = (value & 0x80) != 0;
        let result = value << 1;
        core.set_nz8(result);
        result
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) -> u16 {
        core.flags.c = (value & 0x8000) != 0;
        let result = value << 1;
        core.set_nz16(result);
        result
    }
}

pub struct Rol;

impl ModifyOperator for Rol {
    const NAME: &'static str = "ROL";

    fn apply8(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let carry = core.flags.c as u8;
        core.flags.c = (value & 0x80) != 0;
        let result = (value << 1) | carry;
        core.set_nz8(result);
        result
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) -> u16 {
        let carry = core.flags.c as u16;
        core.flags.c = (value & 0x8000) != 0;
        let result = (value << 1) | carry;
        core.set_nz16(result);
        result
    }
}

pub struct Lsr;

impl ModifyOperator for Lsr {
    const NAME: &'static str = "LSR";

    fn apply8(core: &mut Core<impl Bus>, value: u8) -> u8 {
        core.flags.c = (value & 0x01) != 0;
        let result = value >> 1;
        core.set_nz8(result);
        result
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) -> u16 {
        core.flags.c = (value & 0x0001) != 0;
        let result = value >> 1;
        core.set_nz16(result);
        result
    }
}

pub struct Ror;

impl ModifyOperator for Ror {
    const NAME: &'static str = "ROR";

    fn apply8(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let carry = core.flags.c as u8;
        core.flags.c = (value & 0x01) != 0;
        let result = (value >> 1) | (carry << 7);
        core.set_nz8(result);
        result
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) -> u16 {
        let carry = core.flags.c as u16;
        core.flags.c = (value & 0x0001) != 0;
        let result = (value >> 1) | (carry << 15);
        core.set_nz16(result);
        result
    }
}
