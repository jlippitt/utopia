use super::{Bus, Core};

pub trait ReadAddress {
    const NAME: &'static str;
    fn read(core: &mut Core<impl Bus>) -> u8;
}

pub trait WriteAddress: ReadAddress {
    fn modify<T: Bus>(core: &mut Core<T>, callback: impl FnOnce(&mut Core<T>, u8) -> u8);
}

macro_rules! register {
    ($name:ident, $field:ident) => {
        pub struct $name;

        impl ReadAddress for $name {
            const NAME: &'static str = stringify!($name);

            fn read(core: &mut Core<impl Bus>) -> u8 {
                core.read(core.pc);
                core.$field
            }
        }

        impl WriteAddress for $name {
            fn modify<T: Bus>(core: &mut Core<T>, callback: impl FnOnce(&mut Core<T>, u8) -> u8) {
                let value = core.$field;
                let result = callback(core, value);
                core.$field = result;
            }
        }
    };
}

register!(A, a);
register!(X, x);
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
    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u16;
}

impl<T: Resolver> ReadAddress for T {
    const NAME: &'static str = T::NAME;

    fn read(core: &mut Core<impl Bus>) -> u8 {
        let address = T::resolve(core, false);
        core.read(address)
    }
}

impl<T: Resolver> WriteAddress for T {
    fn modify<U: Bus>(core: &mut Core<U>, callback: impl FnOnce(&mut Core<U>, u8) -> u8) {
        let address = T::resolve(core, true);
        let value = core.read(address);
        let result = callback(core, value);
        core.write(address, result);
    }
}

pub struct XIndirect;

impl Resolver for XIndirect {
    const NAME: &'static str = "(X)";

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u16 {
        if !write {
            core.read(core.pc);
        }

        core.flags.p | (core.x as u16)
    }
}
