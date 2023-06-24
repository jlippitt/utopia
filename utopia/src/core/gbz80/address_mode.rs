use super::{Bus, Core};

pub trait ReadAddress<T> {
    const NAME: &'static str;
    fn read(core: &mut Core<impl Bus>) -> T;
}

pub trait WriteAddress<T>: ReadAddress<T> {
    fn write(core: &mut Core<impl Bus>, value: T);
}

pub struct A;

impl ReadAddress<u8> for A {
    const NAME: &'static str = "A";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        core.a
    }
}

impl WriteAddress<u8> for A {
    fn write(core: &mut Core<impl Bus>, value: u8) {
        core.a = value;
    }
}

macro_rules! reg8_high {
    ($name:ident, $field:ident) => {
        pub struct $name;

        impl ReadAddress<u8> for $name {
            const NAME: &'static str = stringify!($name);

            fn read(core: &mut Core<impl Bus>) -> u8 {
                (core.$field >> 8) as u8
            }
        }

        impl WriteAddress<u8> for $name {
            fn write(core: &mut Core<impl Bus>, value: u8) {
                core.$field = (core.$field & 0xff) | ((value as u16) << 8);
            }
        }
    };
}

reg8_high!(B, bc);
reg8_high!(D, de);
reg8_high!(H, hl);

macro_rules! reg8_low {
    ($name:ident, $field:ident) => {
        pub struct $name;

        impl ReadAddress<u8> for $name {
            const NAME: &'static str = stringify!($name);

            fn read(core: &mut Core<impl Bus>) -> u8 {
                core.$field as u8
            }
        }

        impl WriteAddress<u8> for $name {
            fn write(core: &mut Core<impl Bus>, value: u8) {
                core.$field = (core.$field & 0xff00) | value as u16;
            }
        }
    };
}

reg8_low!(C, bc);
reg8_low!(E, de);
reg8_low!(L, hl);

macro_rules! reg16 {
    ($name:ident, $field:ident) => {
        pub struct $name;

        impl ReadAddress<u16> for $name {
            const NAME: &'static str = stringify!($name);

            fn read(core: &mut Core<impl Bus>) -> u16 {
                core.$field
            }
        }

        impl WriteAddress<u16> for $name {
            fn write(core: &mut Core<impl Bus>, value: u16) {
                core.$field = value;
            }
        }
    };
}

reg16!(BC, bc);
reg16!(DE, de);
reg16!(HL, hl);
reg16!(SP, sp);

macro_rules! reg16_indirect {
    ($name:ident, $display_name:expr, $field:ident) => {
        pub struct $name;

        impl ReadAddress<u8> for $name {
            const NAME: &'static str = $display_name;

            fn read(core: &mut Core<impl Bus>) -> u8 {
                core.read(core.$field)
            }
        }

        impl WriteAddress<u8> for $name {
            fn write(core: &mut Core<impl Bus>, value: u8) {
                core.write(core.$field, value)
            }
        }
    };
}

reg16_indirect!(BCIndirect, "(BC)", bc);
reg16_indirect!(DEIndirect, "(DE)", de);
reg16_indirect!(HLIndirect, "(HL)", hl);

pub struct HLIncrement;

impl ReadAddress<u8> for HLIncrement {
    const NAME: &'static str = "(HL+)";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        let value = core.read(core.hl);
        core.hl = core.hl.wrapping_add(1);
        value
    }
}

impl WriteAddress<u8> for HLIncrement {
    fn write(core: &mut Core<impl Bus>, value: u8) {
        core.write(core.hl, value);
        core.hl = core.hl.wrapping_add(1);
    }
}

pub struct HLDecrement;

impl ReadAddress<u8> for HLDecrement {
    const NAME: &'static str = "(HL-)";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        let value = core.read(core.hl);
        core.hl = core.hl.wrapping_sub(1);
        value
    }
}

impl WriteAddress<u8> for HLDecrement {
    fn write(core: &mut Core<impl Bus>, value: u8) {
        core.write(core.hl, value);
        core.hl = core.hl.wrapping_sub(1);
    }
}

pub struct Immediate;

impl ReadAddress<u8> for Immediate {
    const NAME: &'static str = "u8";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        core.next_byte()
    }
}

pub struct High;

impl ReadAddress<u8> for High {
    const NAME: &'static str = "($FF00+u8)";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        let address = core.next_byte();
        core.read_high(address)
    }
}

impl WriteAddress<u8> for High {
    fn write(core: &mut Core<impl Bus>, value: u8) {
        let address = core.next_byte();
        core.write_high(address, value);
    }
}

pub struct CIndirect;

impl ReadAddress<u8> for CIndirect {
    const NAME: &'static str = "($FF00+C)";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        core.read_high(core.bc as u8)
    }
}

impl WriteAddress<u8> for CIndirect {
    fn write(core: &mut Core<impl Bus>, value: u8) {
        core.write_high(core.bc as u8, value);
    }
}
