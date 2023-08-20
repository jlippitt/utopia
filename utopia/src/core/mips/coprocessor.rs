use super::{Bus, Core};

pub trait Coprocessor0 {
    const REGS: [&'static str; 32];
    fn get(core: &Core<impl Bus<Cp0 = Self>>, index: usize) -> u32;
    fn set(core: &mut Core<impl Bus<Cp0 = Self>>, index: usize, value: u32);
    fn dispatch(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn update(_core: &mut Core<impl Bus<Cp0 = Self>>) {}
}

impl Coprocessor0 for () {
    const REGS: [&'static str; 32] = [""; 32];

    fn get(_core: &Core<impl Bus<Cp0 = Self>>, _index: usize) -> u32 {
        unimplemented!("CP0");
    }

    fn set(_core: &mut Core<impl Bus<Cp0 = Self>>, _index: usize, _value: u32) {
        unimplemented!("CP0");
    }

    fn dispatch(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("CP0");
    }
}

pub trait Coprocessor2 {
    const REGS: [&'static str; 32];
    fn get(core: &Core<impl Bus<Cp2 = Self>>, index: usize, elem: usize) -> u32;
    fn set(core: &mut Core<impl Bus<Cp2 = Self>>, index: usize, elem: usize, value: u32);
    fn dispatch(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
}

impl Coprocessor2 for () {
    const REGS: [&'static str; 32] = [""; 32];

    fn get(_core: &Core<impl Bus<Cp2 = Self>>, _index: usize, _elem: usize) -> u32 {
        unimplemented!("CP2");
    }

    fn set(_core: &mut Core<impl Bus<Cp2 = Self>>, _index: usize, _elem: usize, _value: u32) {
        unimplemented!("CP2");
    }

    fn dispatch(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("CP2");
    }
}
