use super::{Bus, Core};

pub trait Coprocessor0 {
    fn mfc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn mtc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn cop0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn break_(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn step(_core: &mut Core<impl Bus<Cp0 = Self>>) {}
}

impl Coprocessor0 for () {
    fn mfc0(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("MFC0");
    }

    fn mtc0(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("MTC0");
    }

    fn cop0(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("COP0");
    }

    fn break_(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("COP0");
    }
}

pub trait Coprocessor2 {
    fn mfc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn mtc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn lwc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn swc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn cop2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
}

impl Coprocessor2 for () {
    fn mfc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("MFC2");
    }

    fn mtc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("MTC2");
    }

    fn lwc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("MFC2");
    }

    fn swc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("MTC2");
    }

    fn cop2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("COP2");
    }
}
