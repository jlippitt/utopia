use super::{Bus, Core, STACK_PAGE};

pub trait ReadAddress {
    const NAME: &'static str;
    fn init(_core: &mut Core<impl Bus>) {}
    fn read(core: &mut Core<impl Bus>) -> u8;
}

pub trait WriteAddress: ReadAddress {
    fn modify<T: Bus>(core: &mut Core<T>, callback: impl FnOnce(&mut Core<T>, u8) -> u8);
    fn finalize(_core: &mut Core<impl Bus>) {}
}

macro_rules! register {
    ($name:ident, $field:ident) => {
        pub struct $name;

        impl ReadAddress for $name {
            const NAME: &'static str = stringify!($name);

            fn init(core: &mut Core<impl Bus>) {
                core.read(core.pc);
            }

            fn read(core: &mut Core<impl Bus>) -> u8 {
                core.$field
            }
        }

        impl WriteAddress for $name {
            fn modify<T: Bus>(core: &mut Core<T>, callback: impl FnOnce(&mut Core<T>, u8) -> u8) {
                let value = Self::read(core);
                let result = callback(core, value);
                core.$field = result;
            }
        }
    };
}

register!(A, a);
register!(X, x);
register!(Y, y);
register!(SP, sp);

pub struct Psw;

impl ReadAddress for Psw {
    const NAME: &'static str = "PSW";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        let mut value = 0;
        value |= if (core.flags.n & 0x80) != 0 { 0x80 } else { 0 };
        value |= if core.flags.v { 0x40 } else { 0 };
        value |= if core.flags.p != 0 { 0x20 } else { 0 };
        value |= if core.flags.b { 0x10 } else { 0 };
        value |= if core.flags.h { 0x08 } else { 0 };
        value |= if core.flags.i { 0x04 } else { 0 };
        value |= if core.flags.z == 0 { 0x02 } else { 0 };
        value |= if core.flags.c { 0x01 } else { 0 };
        value
    }
}

impl WriteAddress for Psw {
    fn modify<T: Bus>(core: &mut Core<T>, callback: impl FnOnce(&mut Core<T>, u8) -> u8) {
        let value = Self::read(core);
        let result = callback(core, value);
        core.flags.n = result;
        core.flags.v = (result & 0x40) != 0;
        core.flags.p = if result & 0x20 != 0 { STACK_PAGE } else { 0 };
        core.flags.b = (result & 0x10) != 0;
        core.flags.h = (result & 0x08) != 0;
        core.flags.i = (result & 0x04) != 0;
        core.flags.z = !result & 0x02;
        core.flags.c = (result & 0x01) != 0;
    }
}

pub struct Immediate;

impl ReadAddress for Immediate {
    const NAME: &'static str = "#i";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        core.next_byte()
    }
}

pub trait Resolver {
    const NAME: &'static str;
    fn init(_core: &mut Core<impl Bus>) {}
    fn resolve(core: &mut Core<impl Bus>) -> u16;
}

impl<T: Resolver> ReadAddress for T {
    const NAME: &'static str = T::NAME;

    fn init(core: &mut Core<impl Bus>) {
        T::init(core);
    }

    fn read(core: &mut Core<impl Bus>) -> u8 {
        let address = T::resolve(core);
        core.read(address)
    }
}

impl<T: Resolver> WriteAddress for T {
    fn modify<U: Bus>(core: &mut Core<U>, callback: impl FnOnce(&mut Core<U>, u8) -> u8) {
        let address = T::resolve(core);
        let value = core.read(address);
        let result = callback(core, value);
        core.write(address, result);
    }

    fn finalize(core: &mut Core<impl Bus>) {
        core.idle();
    }
}

fn get_indirect(core: &mut Core<impl Bus>, low_address: u16) -> u16 {
    let low = core.read(low_address);
    let high_address = (low_address & 0xff00) | (low_address.wrapping_add(1) & 0xff);
    let high = core.read(high_address);
    u16::from_le_bytes([low, high])
}

pub struct Absolute;

impl Resolver for Absolute {
    const NAME: &'static str = "!a";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        core.next_word()
    }
}

pub struct AbsoluteX;

impl Resolver for AbsoluteX {
    const NAME: &'static str = "!a+X";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        core.next_word().wrapping_add(core.x as u16)
    }
}

pub struct AbsoluteY;

impl Resolver for AbsoluteY {
    const NAME: &'static str = "!a+Y";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        core.next_word().wrapping_add(core.y as u16)
    }
}

pub struct Direct;

impl Resolver for Direct {
    const NAME: &'static str = "d";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        core.flags.p | (core.next_byte() as u16)
    }
}

pub struct DirectX;

impl Resolver for DirectX {
    const NAME: &'static str = "d+X";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        let base = core.next_byte();
        core.idle();
        core.flags.p | base.wrapping_add(core.x) as u16
    }
}

pub struct DirectY;

impl Resolver for DirectY {
    const NAME: &'static str = "d+Y";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        let base = core.next_byte();
        core.idle();
        core.flags.p | base.wrapping_add(core.y) as u16
    }
}

pub struct DirectXIndirect;

impl Resolver for DirectXIndirect {
    const NAME: &'static str = "[d+X]";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        let direct = DirectX::resolve(core);
        get_indirect(core, direct)
    }
}

pub struct DirectIndirectY;

impl Resolver for DirectIndirectY {
    const NAME: &'static str = "[d]+Y";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        let direct = Direct::resolve(core);
        let base = get_indirect(core, direct);
        core.idle();
        base.wrapping_add(core.y as u16)
    }
}

pub struct XIndirect;

impl Resolver for XIndirect {
    const NAME: &'static str = "(X)";

    fn init(core: &mut Core<impl Bus>) {
        core.read(core.pc);
    }

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        core.flags.p | (core.x as u16)
    }
}
