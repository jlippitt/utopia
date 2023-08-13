use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn cop0(core: &mut Core<impl Bus>, word: u32) {
    match (word >> 21) & 31 {
        0b00000 => super::type_r(core, mfc0, word),
        0b00100 => super::type_r(core, mtc0, word),
        rs => unimplemented!("COP0 RS={:05b} ({:08X}: {:08X})", rs, core.pc, word),
    }
}

fn mfc0(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFC0 {}, ${}", core.pc, REGS[rt], rd);

    let result = match rd {
        12 => {
            // STATUS
            // TODO: Interrupts/Exceptions
            0x3000_0000
        }
        _ => todo!("COP0 register read: ${}", rd),
    };

    core.set(rt, result);
}

fn mtc0(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MTC0 {}, ${}", core.pc, REGS[rt], rd);

    let value = core.get(rt);

    match rd {
        12 => {
            // STATUS
            // TODO: Interrupts/Exceptions
            if value != 0x3000_0000 {
                todo!("COP0 status register")
            }
        }
        _ => {
            if value != 0 {
                todo!("COP0 register write: ${} <= {:08X}", rd, value);
            }
        }
    }
}
