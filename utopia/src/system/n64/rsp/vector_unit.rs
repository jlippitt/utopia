use crate::core::mips::{Bus, Coprocessor2, Core, REGS};
use tracing::debug;

pub struct VectorUnit {
    regs: [[u16; 8]; 32],
}

impl VectorUnit {
    pub fn new() -> Self {
        Self { regs: [[0; 8]; 32] }
    }
}

impl Coprocessor2 for VectorUnit {
    fn mfc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;
        let elem = ((word >> 7) & 15) as usize;

        debug!("{:08X} MFC2 {}, $V{:02}", core.pc(), REGS[rt], rd);

        core.set(rt, core.cp2().regs[rd][elem >> 1] as u32);
    }

    fn mtc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;
        let elem = ((word >> 7) & 15) as usize;

        debug!("{:08X} MTC2 {}, $V{:02}", core.pc(), REGS[rt], rd);

        core.cp2_mut().regs[rd][elem >> 1] = core.get(rt) as u16;
    }

    fn cop2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
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
