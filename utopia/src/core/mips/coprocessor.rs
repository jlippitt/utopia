use super::{Bus, Core};

pub trait Cp0 {
    fn translate(&self, address: u32) -> u32;
    fn mfc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn mtc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn dmfc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn dmtc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn cop0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn syscall(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn break_(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn step(core: &mut Core<impl Bus<Cp0 = Self>>);
}

pub trait Cp1 {
    fn set_fr(&mut self, fr: bool);
    fn mfc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn mtc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn dmfc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn dmtc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn cfc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn ctc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn lwc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn ldc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn swc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn sdc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn cop1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
    fn bc1(core: &mut Core<impl Bus<Cp1 = Self>>, word: u32);
}

pub trait Cp2 {
    fn mfc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn mtc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn cfc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn ctc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn lwc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn swc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
    fn cop2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
}

#[derive(Default)]
pub struct NullCp1;

impl Cp1 for NullCp1 {
    fn set_fr(&mut self, _fr: bool) {
        unimplemented!("CP1")
    }

    fn mfc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn mtc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn dmfc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn dmtc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn cfc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn ctc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn lwc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn ldc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn swc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn sdc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn cop1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }

    fn bc1(_core: &mut Core<impl Bus<Cp1 = Self>>, _word: u32) {
        unimplemented!("CP1")
    }
}

#[derive(Default)]
pub struct NullCp2;

impl Cp2 for NullCp2 {
    fn mfc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("CP2")
    }

    fn mtc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("CP2")
    }

    fn cfc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("CP2")
    }

    fn ctc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("CP2")
    }

    fn lwc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("CP2")
    }

    fn swc2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("CP2")
    }

    fn cop2(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("CP2")
    }
}
