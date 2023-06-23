use super::{Bus, Core};

pub trait ReadAddress<T> {
    const NAME: &'static str;
    fn read(core: &mut Core<impl Bus>) -> T;
}

pub trait WriteAddress<T> : ReadAddress<T> {
    fn write(core: &mut Core<impl Bus>, value: T);
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
    }
}

reg16!(BC, bc);
reg16!(DE, de);
reg16!(HL, hl);
reg16!(SP, sp);