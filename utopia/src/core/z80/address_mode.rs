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

pub struct AF;

impl ReadAddress<u16> for AF {
    const NAME: &'static str = "AF";

    fn read(core: &mut Core<impl Bus>) -> u16 {
        let mut value = (core.a as u16) << 8;
        value |= core.flags.s as u16 & 0x80;
        value |= if core.flags.z == 0 { 0x40 } else { 0 };
        value |= core.flags.y as u16 & 0x20;
        value |= if core.flags.h { 0x10 } else { 0 };
        value |= core.flags.x as u16 & 0x08;
        value |= if core.flags.pv { 0x04 } else { 0 };
        value |= if core.flags.n { 0x02 } else { 0 };
        value |= if core.flags.c { 0x01 } else { 0 };
        value
    }
}

impl WriteAddress<u16> for AF {
    fn write(core: &mut Core<impl Bus>, value: u16) {
        core.a = (value >> 8) as u8;
        core.flags.s = value as u8;
        core.flags.z = !(value as u8) & 0x40;
        core.flags.y = value as u8;
        core.flags.h = (value & 0x10) != 0;
        core.flags.x = value as u8;
        core.flags.pv = (value & 0x04) != 0;
        core.flags.n = (value & 0x02) != 0;
        core.flags.c = (value & 0x01) != 0;
    }
}

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

pub struct Immediate;

impl ReadAddress<u8> for Immediate {
    const NAME: &'static str = "u8";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        core.next_byte()
    }
}

impl ReadAddress<u16> for Immediate {
    const NAME: &'static str = "u16";

    fn read(core: &mut Core<impl Bus>) -> u16 {
        core.next_word()
    }
}

pub struct Absolute;

impl ReadAddress<u8> for Absolute {
    const NAME: &'static str = "(u16)";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        let address = core.next_word();
        core.read(address)
    }
}

impl WriteAddress<u8> for Absolute {
    fn write(core: &mut Core<impl Bus>, value: u8) {
        let address = core.next_word();
        core.write(address, value);
    }
}

impl ReadAddress<u16> for Absolute {
    const NAME: &'static str = "(u16)";

    fn read(core: &mut Core<impl Bus>) -> u16 {
        let address = core.next_word();
        let low = core.read(address);
        let high = core.read(address.wrapping_add(1));
        u16::from_le_bytes([low, high])
    }
}

impl WriteAddress<u16> for Absolute {
    fn write(core: &mut Core<impl Bus>, value: u16) {
        let address = core.next_word();
        core.write(address, value as u8);
        core.write(address.wrapping_add(1), (value >> 8) as u8);
    }
}
