use super::{Bus, Core, REGS};
use tracing::debug;

#[derive(Default, Debug)]
pub struct TlbEntry {
    lo0: u32,
    lo1: u32,
    hi: u32,
}

#[derive(Default)]
pub struct Status {
    ie: bool,
    exl: bool,
    im: u8,
    cu: [bool; 4],
}

#[derive(Default)]
pub struct Cop0 {
    index: u32,
    lo0: u32,
    lo1: u32,
    hi: u32,
    status: Status,
    epc: u32,
    tlb_entries: [TlbEntry; 32],
}

pub fn dispatch(core: &mut Core<impl Bus>, word: u32) {
    match (word >> 21) & 31 {
        0b00000 => type_r(core, mfc0, word),
        0b00100 => type_r(core, mtc0, word),
        0b10000..=0b11111 => match word & 63 {
            0b000010 => tlbwi(core),
            func => unimplemented!(
                "COP0 RS=10000 FN={:06b} ({:08X}: {:08X})",
                func,
                core.pc,
                word
            ),
        },
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
        0 => core.cop0.index,
        2 => core.cop0.lo0,
        3 => core.cop0.lo1,
        10 => core.cop0.hi,
        12 => {
            let status = &mut core.cop0.status;
            let mut value = 0x3000_0000;
            value |= if status.ie { 0x0000_0001 } else { 0 };
            value |= if status.exl { 0x0000_0002 } else { 0 };
            value |= (status.im as u32) << 8;
            value |= if status.cu[0] { 0x1000_0000 } else { 0 };
            value |= if status.cu[1] { 0x2000_0000 } else { 0 };
            value |= if status.cu[2] { 0x4000_0000 } else { 0 };
            value |= if status.cu[3] { 0x8000_0000 } else { 0 };
            value
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
            core.cop0.index = value;
            debug!("  COP0 Index: {}", core.cop0.index);
        }
        2 => {
            core.cop0.lo0 = value;
            debug!("  COP0 LO0: {:08X}", core.cop0.lo0);
        }
        3 => {
            core.cop0.lo1 = value;
            debug!("  COP0 LO1: {:08X}", core.cop0.lo1);
        }
        10 => {
            core.cop0.hi = value;
            debug!("  COP0 HI: {:08X}", core.cop0.hi);
        }
        12 => {
            if (value & 0x0fff_00fc) != 0 {
                todo!("COP0 Status Register: {:08X}", value);
            }

            let status = &mut core.cop0.status;
            status.ie = (value & 0x0000_0001) != 0;
            status.exl = (value & 0x0000_0002) != 0;
            status.im = (value >> 8) as u8;
            status.cu[0] = (value & 0x1000_0000) != 0;
            status.cu[1] = (value & 0x2000_0000) != 0;
            status.cu[2] = (value & 0x4000_0000) != 0;
            status.cu[3] = (value & 0x6000_0000) != 0;
            debug!("  COP0 IE: {}", status.ie);
            debug!("  COP0 EXL: {}", status.exl);
            debug!("  COP0 IM: {:08b}", status.im);
            debug!("  COP0 CU: {:?}", status.cu);
        }
        14 => {
            core.cop0.epc = value;
            debug!("  COP0 EPC: {:08X}", core.cop0.epc);
        }
        _ => {
            if value != 0 {
                todo!("COP0 Register Write: ${} <= {:08X}", rd, value);
            }
        }
    }
}

fn tlbwi(core: &mut Core<impl Bus>) {
    debug!("{:08X} TLBWI", core.pc);

    let tlb_entry = &mut core.cop0.tlb_entries[core.cop0.index as usize & 31];
    tlb_entry.lo0 = core.cop0.lo0;
    tlb_entry.lo1 = core.cop0.lo1;
    tlb_entry.hi = core.cop0.hi;

    debug!("TLB Entry {}: {:X?}", core.cop0.index, tlb_entry);
}
