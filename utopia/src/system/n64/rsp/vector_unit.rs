use crate::core::mips::{Bus, Coprocessor2, Core};

pub struct VectorUnit {
    regs: [[u16; 8]; 32],
}

impl VectorUnit {
    pub fn new() -> Self {
        Self { regs: [[0; 8]; 32] }
    }
}

impl Coprocessor2 for VectorUnit {
    #[rustfmt::skip]
    const REGS: [&'static str; 32] = [
        "$V00", "$V01", "$V02", "$V03", "$V04", "$V05", "$V06", "$V07",
        "$V08", "$V09", "$V10", "$V11", "$V12", "$V13", "$V14", "$V15",
        "$V16", "$V17", "$V18", "$V19", "$V20", "$V21", "$V22", "$V23",
        "$V24", "$V25", "$V26", "$V27", "$V28", "$V29", "$V30", "$V31",
    ];

    fn get(core: &Core<impl Bus<Cp2 = Self>>, index: usize, elem: usize) -> u32 {
        debug_assert!((elem & 1) == 0);
        core.cp2().regs[index][elem >> 1] as u32
    }

    fn set(core: &mut Core<impl Bus<Cp2 = Self>>, index: usize, elem: usize, value: u32) {
        debug_assert!((elem & 1) == 0);
        core.cp2_mut().regs[index][elem >> 1] = value as u16;
    }

    fn dispatch(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        match word & 63 {
            func => unimplemented!(
                "RSP CP2 RS=10000 FN={:06b} ({:08X}: {:08X})",
                func,
                core.pc(),
                word
            ),
        }
    }
}
