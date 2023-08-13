use super::{Bus, Core, REGS};
use tracing::debug;

#[derive(Default)]
pub struct TlbEntry {
    lo0: u32,
    lo1: u32,
    hi: u32,
}

#[derive(Default)]
pub struct Cop0 {
    index: usize,
    tlb_entries: [TlbEntry; 32],
}

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
        2 => core.cop0.tlb_entries[core.cop0.index].lo0,
        3 => core.cop0.tlb_entries[core.cop0.index].lo1,
        10 => core.cop0.tlb_entries[core.cop0.index].hi,
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
        0 => {
            core.cop0.index = value as usize & 31;
            debug!("COP0 Index: {}", core.cop0.index);
        }
        2 => {
            core.cop0.tlb_entries[core.cop0.index].lo0 = value;
            debug!("COP0 TLB[{}] LO0: {:08X}", core.cop0.index, value);
        }
        3 => {
            core.cop0.tlb_entries[core.cop0.index].lo1 = value;
            debug!("COP0 TLB[{}] LO1: {:08X}", core.cop0.index, value);
        }
        10 => {
            core.cop0.tlb_entries[core.cop0.index].hi = value;
            debug!("COP0 TLB[{}] HI: {:08X}", core.cop0.index, value);
        }
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
