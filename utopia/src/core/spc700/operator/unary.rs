use super::super::{Bus, Core};

pub trait UnaryOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8;
}

pub struct Asl;

impl UnaryOperator for Asl {
    const NAME: &'static str = "ASL";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        core.flags.c = (value & 0x80) != 0;
        let result = value << 1;
        core.set_nz(result);
        result
    }
}

pub struct Rol;

impl UnaryOperator for Rol {
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

impl UnaryOperator for Lsr {
    const NAME: &'static str = "LSR";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        core.flags.c = (value & 0x01) != 0;
        let result = value >> 1;
        core.set_nz(result);
        result
    }
}

pub struct Ror;

impl UnaryOperator for Ror {
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

impl UnaryOperator for Dec {
    const NAME: &'static str = "DEC";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        core.set_nz(result);
        result
    }
}

pub struct Inc;

impl UnaryOperator for Inc {
    const NAME: &'static str = "INC";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        core.set_nz(result);
        result
    }
}

pub struct Set1<const BIT: u8>;

impl<const BIT: u8> UnaryOperator for Set1<BIT> {
    // There must be a better way of doing this... :(
    const NAME: &'static str = unsafe {
        std::str::from_utf8_unchecked(&['S' as u8, 'E' as u8, 'T' as u8, ('0' as u8) + BIT])
    };

    fn apply(_core: &mut Core<impl Bus>, value: u8) -> u8 {
        value | (1 << BIT)
    }
}

pub struct Clr1<const BIT: u8>;

impl<const BIT: u8> UnaryOperator for Clr1<BIT> {
    // There must be a better way of doing this... :(
    const NAME: &'static str = unsafe {
        std::str::from_utf8_unchecked(&['C' as u8, 'L' as u8, 'R' as u8, ('0' as u8) + BIT])
    };

    fn apply(_core: &mut Core<impl Bus>, value: u8) -> u8 {
        value & !(1 << BIT)
    }
}
