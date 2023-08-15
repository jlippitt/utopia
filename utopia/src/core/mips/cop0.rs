use super::{Bus, Core, REGS};
use tracing::debug;

#[rustfmt::skip]
const CREGS: [&str; 32] = [
    "Index",
    "Random",
    "EntryLo0",
    "EntryLo1",
    "Context",
    "PageMask",
    "Wired",
    "$7",
    "BadVAddr",
    "Count",
    "EntryHi",
    "Compare",
    "Status",
    "Cause",
    "EPC",
    "PRId",
    "Config",
    "LLAddr",
    "WatchLo",
    "WatchHi",
    "XContext",
    "$21",
    "$22",
    "$23",
    "$24",
    "$25",
    "Parity Error",
    "Cache Error",
    "TagLo",
    "TagHi",
    "ErrorEPC",
    "$31",
];

#[derive(Default, Debug)]
pub struct TlbEntry {
    lo0: u32,
    lo1: u32,
    hi: u32,
    page_mask: u32,
}

#[derive(Default)]
pub struct Status {
    ie: bool,
    exl: bool,
    erl: bool,
    mode: u32,
    ux: bool,
    sx: bool,
    kx: bool,
    im: u8,
    ds: u32,
    re: bool,
    fr: bool,
    rp: bool,
    cu: [bool; 4],
}

#[derive(Default)]
pub struct Cop0 {
    index: u32,
    lo0: u32,
    lo1: u32,
    page_mask: u32,
    wired: u32,
    hi: u32,
    status: Status,
    epc: u32,
    error_epc: u32,
    tlb_entries: [TlbEntry; 32],
}

pub fn dispatch(core: &mut Core<impl Bus>, word: u32) {
    match (word >> 21) & 31 {
        0b00000 => type_r(core, mfc0, word),
        0b00100 => type_r(core, mtc0, word),
        0b10000..=0b11111 => match word & 63 {
            0o01 => tlbr(core),
            0o02 => tlbwi(core),
            0o10 => tlbp(core),
            0o30 => eret(core),
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
    debug!("{:08X} MFC0 {}, {}", core.pc, REGS[rt], CREGS[rd]);

    let result = match rd {
        0 => core.cop0.index,
        2 => core.cop0.lo0,
        3 => core.cop0.lo1,
        5 => core.cop0.page_mask,
        6 => core.cop0.wired,
        10 => core.cop0.hi,
        12 => {
            let status = &mut core.cop0.status;
            let mut value = 0;
            value |= if status.ie { 0x0000_0001 } else { 0 };
            value |= if status.exl { 0x0000_0002 } else { 0 };
            value |= if status.erl { 0x0000_0004 } else { 0 };
            value |= status.mode << 3;
            value |= if status.ux { 0x0000_0020 } else { 0 };
            value |= if status.sx { 0x0000_0040 } else { 0 };
            value |= if status.kx { 0x0000_0080 } else { 0 };
            value |= (status.im as u32) << 8;
            value |= status.ds << 16;
            value |= if status.re { 0x0200_0000 } else { 0 };
            value |= if status.fr { 0x0400_0000 } else { 0 };
            value |= if status.rp { 0x0800_0000 } else { 0 };
            value |= if status.cu[0] { 0x1000_0000 } else { 0 };
            value |= if status.cu[1] { 0x2000_0000 } else { 0 };
            value |= if status.cu[2] { 0x4000_0000 } else { 0 };
            value |= if status.cu[3] { 0x8000_0000 } else { 0 };
            value
        }
        _ => todo!("COP0 Register Read: {}", CREGS[rd]),
    };

    core.set(rt, result);
}

fn mtc0(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} MTC0 {}, {}", core.pc, REGS[rt], CREGS[rd]);

    let value = core.get(rt);

    match rd {
        0 => {
            core.cop0.index = value & 0x8000_003f;
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
        5 => {
            core.cop0.page_mask = value & 0x01ff_e000;
            debug!("  COP0 Page Mask: {:08X}", core.cop0.page_mask);
        }
        6 => {
            core.cop0.wired = value & 0x0000_003f;
            debug!("  COP0 Wired: {:08X}", core.cop0.wired);
        }
        10 => {
            core.cop0.hi = value;
            debug!("  COP0 HI: {:08X}", core.cop0.hi);
        }
        12 => {
            // if (value & 0x0fff_00f8) != 0 {
            //     unimplemented!("COP0 Feature: {:08X}", value);
            // }

            let status = &mut core.cop0.status;
            status.ie = (value & 0x0000_0001) != 0;
            status.exl = (value & 0x0000_0002) != 0;
            status.erl = (value & 0x0000_0004) != 0;
            status.mode = (value >> 3) & 3;
            status.ux = (value & 0x0000_0020) != 0;
            status.sx = (value & 0x0000_0020) != 0;
            status.kx = (value & 0x0000_0020) != 0;
            status.im = (value >> 8) as u8;
            status.ds = (value >> 16) & 511;
            status.re = (value & 0x0200_0000) != 0;
            status.fr = (value & 0x0400_0000) != 0;
            status.rp = (value & 0x0800_0000) != 0;
            status.cu[0] = (value & 0x1000_0000) != 0;
            status.cu[1] = (value & 0x2000_0000) != 0;
            status.cu[2] = (value & 0x4000_0000) != 0;
            status.cu[3] = (value & 0x6000_0000) != 0;
            debug!("  COP0 IE: {}", status.ie);
            debug!("  COP0 EXL: {}", status.exl);
            debug!("  COP0 ERL: {}", status.erl);
            debug!("  COP0 Mode: {:02b}", status.mode);
            debug!("  COP0 UX: {}", status.ux);
            debug!("  COP0 SX: {}", status.sx);
            debug!("  COP0 KX: {}", status.kx);
            debug!("  COP0 IM: {:08b}", status.im);
            debug!("  COP0 DS: {}", status.ds);
            debug!("  COP0 RE: {}", status.re);
            debug!("  COP0 FR: {}", status.fr);
            debug!("  COP0 RP: {}", status.rp);
            debug!("  COP0 CU: {:?}", status.cu);
        }
        14 => {
            core.cop0.epc = value;
            debug!("  COP0 EPC: {:08X}", core.cop0.epc);
        }
        30 => {
            core.cop0.error_epc = value;
            debug!("  COP0 Error EPC: {:08X}", core.cop0.error_epc);
        }
        _ => {
            if value != 0 {
                todo!("COP0 Register Write: {} <= {:08X}", CREGS[rd], value);
            }
        }
    }
}

fn tlbr(core: &mut Core<impl Bus>) {
    debug!("{:08X} TLBR", core.pc);

    let tlb_entry = &core.cop0.tlb_entries[core.cop0.index as usize];

    let global = tlb_entry.lo0 & tlb_entry.lo1 & 1;
    core.cop0.lo0 = (tlb_entry.lo0 & 0xffff_fffe) | global;
    core.cop0.lo1 = (tlb_entry.lo1 & 0xffff_fffe) | global;
    core.cop0.hi = tlb_entry.hi;
    core.cop0.page_mask = tlb_entry.page_mask;

    debug!("  COP0 LO0: {:08X}", core.cop0.lo0);
    debug!("  COP0 LO1: {:08X}", core.cop0.lo1);
    debug!("  COP0 HI: {:08X}", core.cop0.hi);
    debug!("  COP0 Page Mask: {:08X}", core.cop0.page_mask);
}

fn tlbwi(core: &mut Core<impl Bus>) {
    debug!("{:08X} TLBWI", core.pc);

    let tlb_entry = &mut core.cop0.tlb_entries[core.cop0.index as usize];
    tlb_entry.lo0 = core.cop0.lo0;
    tlb_entry.lo1 = core.cop0.lo1;
    tlb_entry.hi = core.cop0.hi;
    tlb_entry.page_mask = core.cop0.page_mask;

    debug!("TLB Entry {}: {:X?}", core.cop0.index, tlb_entry);
}

fn tlbp(core: &mut Core<impl Bus>) {
    debug!("{:08X} TLBP", core.pc);

    let index = core.cop0.tlb_entries.iter().position(|entry| {
        let mask = if ((entry.lo0 & entry.lo0) & 1) != 0 {
            // Global flag is set
            0xffff_e000
        } else {
            0xffff_e0ff
        };

        (entry.hi & mask) == (core.cop0.hi & mask)
    });

    if let Some(index) = index {
        core.cop0.index = index as u32;
    } else {
        core.cop0.index |= 0x8000_0000;
    }

    debug!("  COP0 Index: {}", core.cop0.index);
}

fn eret(core: &mut Core<impl Bus>) {
    debug!("{:08X} ERET", core.pc);

    if core.cop0.status.erl {
        core.next[0] = core.cop0.error_epc;
        core.cop0.status.erl = false;
        debug!("  COP0 ERL: {}", core.cop0.status.erl);
    } else {
        core.next[0] = core.cop0.epc;
        core.cop0.status.exl = false;
        debug!("  COP0 EXL: {}", core.cop0.status.exl);
    }
}
