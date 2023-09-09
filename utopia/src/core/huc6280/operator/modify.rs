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

pub struct Tsb;

impl ModifyOperator for Tsb {
    const NAME: &'static str = "TSB";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        core.flags.n = value;
        core.flags.v = value << 1;
        core.flags.z = value & core.a;
        value | core.a
    }
}

pub struct Trb;

impl ModifyOperator for Trb {
    const NAME: &'static str = "TRB";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        core.flags.n = value;
        core.flags.v = value << 1;
        core.flags.z = value & core.a;
        value & !core.a
    }
}

pub struct Smb<const BIT: u8>;

impl<const BIT: u8> ModifyOperator for Smb<BIT> {
    const NAME: &'static str =
        unsafe { std::str::from_utf8_unchecked(&[b'S', b'M', b'B', b'0' + BIT]) };

    fn apply(_core: &mut Core<impl Bus>, value: u8) -> u8 {
        value | (1 << BIT)
    }
}

pub struct Rmb<const BIT: u8>;

impl<const BIT: u8> ModifyOperator for Rmb<BIT> {
    const NAME: &'static str =
        unsafe { std::str::from_utf8_unchecked(&[b'R', b'M', b'B', b'0' + BIT]) };

    fn apply(_core: &mut Core<impl Bus>, value: u8) -> u8 {
        value & !(1 << BIT)
    }
}
