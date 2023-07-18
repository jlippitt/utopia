use super::{Bus, Core};

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

pub struct Absolute;

impl Resolver for Absolute {
    const NAME: &'static str = "!a";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        core.next_word()
    }
}

pub struct Direct;

impl Resolver for Direct {
    const NAME: &'static str = "d";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        core.flags.p | (core.next_byte() as u16)
    }
}

pub struct DirectIndirectY;

impl Resolver for DirectIndirectY {
    const NAME: &'static str = "[d]+Y";

    fn resolve(core: &mut Core<impl Bus>) -> u16 {
        let low_address = Direct::resolve(core);
        let low = core.read(low_address);
        let high_address = (low_address & 0xff00) | (low_address.wrapping_add(1) & 0xff);
        let high = core.read(high_address);
        let base = u16::from_le_bytes([low, high]);
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
