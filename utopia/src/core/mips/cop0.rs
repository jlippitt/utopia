use super::{Bus, Core, REGS};
use tracing::debug;

pub fn dispatch(core: &mut Core<impl Bus>, word: u32) {
    match (word >> 21) & 31 {
        0b00000 => type_r(core, mfc0, word),
        0b00100 => type_r(core, mtc0, word),
        rs => unimplemented!("COP0 RS={:05b} ({:08X}: {:08X})", rs, core.pc, word),
    }
}

fn type_r<T: Bus>(core: &mut Core<T>, instr: impl Fn(&mut Core<T>, usize, usize), word: u32) {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    instr(core, rt, rd);
}

fn mfc0(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} MFC0 {}, ${}", core.pc, REGS[rt], rd);

    let result = match rd {
        12 => {
            // STATUS
            // TODO: Interrupts/Exceptions
            0x3000_0000
        }
        _ => todo!("COP0 Register Read: ${}", rd),
    };

    core.set(rt, result);
}

fn mtc0(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} MTC0 {}, ${}", core.pc, REGS[rt], rd);

    let value = core.get(rt);

    match rd {
        12 => {
            // STATUS
            // TODO: Interrupts/Exceptions
            if value != 0x3000_0000 {
                todo!("COP0 Status Register")
            }
        }
        _ => {
            if value != 0 {
                todo!("COP0 Register Write: ${} <= {:08X}", rd, value);
            }
        }
    }
}
