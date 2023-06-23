use super::{Bus, Core};

pub trait ReadAddress<T> {
    const NAME: &'static str;
    fn read(core: &mut Core<impl Bus>) -> T;
}

pub trait WriteAddress<T> : ReadAddress<T> {
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
    }
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
    }
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
    }
}

reg16!(BC, bc);
reg16!(DE, de);
reg16!(HL, hl);
reg16!(SP, sp);