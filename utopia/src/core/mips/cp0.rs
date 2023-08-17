use super::{Bus, Core, REGS};
use bitfield_struct::bitfield;
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
    page_mask: u32,
    hi: u32,
}

#[bitfield(u32)]
pub struct Cause {
    #[bits(2)]
    __: u8,
    #[bits(5)]
    exc_code: u8,
    __: bool,
    #[bits(8)]
    ip: u8,
    #[bits(12)]
    __: u16,
    #[bits(2)]
    ce: u8,
    __: bool,
    bd: bool,
}

#[bitfield(u32, default = false)]
pub struct Status {
    ie: bool,
    exl: bool,
    erl: bool,
    #[bits(2)]
    mode: u8,
    ux: bool,
    sx: bool,
    kx: bool,
    im: u8,
    #[bits(9)]
    ds: u16,
    re: bool,
    fr: bool,
    rp: bool,
    cu0: bool,
    cu1: bool,
    cu2: bool,
    cu3: bool,
}

#[derive(Default)]
pub struct Cp0 {
    index: u32,
    lo0: u32,
    lo1: u32,
    page_mask: u32,
    wired: u32,
    count: u32,
    hi: u32,
    status: Status,
    cause: Cause,
    epc: u32,
    error_epc: u32,
    tlb_entries: [TlbEntry; 32],
}

pub fn update(core: &mut Core<impl Bus>) {
    core.cp0.count = core.cp0.count.wrapping_add(1);

    // Test for interrupts
    let cp0 = &mut core.cp0;

    // If IE=0, EXL=1 or ERL=1, no interrupt for you
    if (u32::from(cp0.status) ^ 0x01) & 0x07 != 0 {
        return;
    }

    let int_active = core.bus.poll() & cp0.status.im();

    if int_active == 0 {
        return;
    }

    // Handle interrupt exception
    debug!("-- Exception: {:08b} --", int_active);

    let int_pending = (cp0.cause.ip() & 0x83) | (int_active & 0x7c);
    cp0.cause.set_ip(int_pending);

    cp0.cause.set_exc_code(0);
    cp0.cause.set_bd(core.delay > 0);

    core.cp0.epc = if core.delay > 0 {
        core.next[0].wrapping_sub(4)
    } else {
        core.next[0]
    };

    core.cp0.status.set_exl(true);

    core.next[0] = 0x8000_0180;
    core.next[1] = core.next[0].wrapping_add(4);
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
                "CP0 RS=10000 FN={:06b} ({:08X}: {:08X})",
                func,
                core.pc,
                word
            ),
        },
        rs => unimplemented!("CP0 RS={:05b} ({:08X}: {:08X})", rs, core.pc, word),
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
        0 => core.cp0.index,
        2 => core.cp0.lo0,
        3 => core.cp0.lo1,
        5 => core.cp0.page_mask,
        6 => core.cp0.wired,
        9 => core.cp0.count,
        10 => core.cp0.hi,
        12 => u32::from(core.cp0.status) & !0x0088_0000,
        13 => core.cp0.cause.into(),
        14 => core.cp0.epc,
        _ => todo!("CP0 Register Read: {}", CREGS[rd]),
    };

    core.set(rt, result);
}

fn mtc0(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} MTC0 {}, {}", core.pc, REGS[rt], CREGS[rd]);

    let value = core.get(rt);

    match rd {
        0 => {
            core.cp0.index = value & 0x8000_003f;
            debug!("  CP0 Index: {}", core.cp0.index);
        }
        2 => {
            core.cp0.lo0 = value;
            debug!("  CP0 LO0: {:08X}", core.cp0.lo0);
        }
        3 => {
            core.cp0.lo1 = value;
            debug!("  CP0 LO1: {:08X}", core.cp0.lo1);
        }
        5 => {
            core.cp0.page_mask = value & 0x01ff_e000;
            debug!("  CP0 Page Mask: {:08X}", core.cp0.page_mask);
        }
        6 => {
            core.cp0.wired = value & 0x0000_003f;
            debug!("  CP0 Wired: {:08X}", core.cp0.wired);
        }
        10 => {
            core.cp0.hi = value;
            debug!("  CP0 HI: {:08X}", core.cp0.hi);
        }
        12 => {
            core.cp0.status = value.into();
            debug!("  CP0 Status: {:?}", core.cp0.status);
        }
        14 => {
            core.cp0.epc = value;
            debug!("  CP0 EPC: {:08X}", core.cp0.epc);
        }
        30 => {
            core.cp0.error_epc = value;
            debug!("  CP0 Error EPC: {:08X}", core.cp0.error_epc);
        }
        _ => {
            if value != 0 {
                todo!("CP0 Register Write: {} <= {:08X}", CREGS[rd], value);
            }
        }
    }
}

fn tlbr(core: &mut Core<impl Bus>) {
    debug!("{:08X} TLBR", core.pc);

    let tlb_entry = &core.cp0.tlb_entries[core.cp0.index as usize];

    let global = tlb_entry.lo0 & tlb_entry.lo1 & 1;
    core.cp0.lo0 = (tlb_entry.lo0 & 0xffff_fffe) | global;
    core.cp0.lo1 = (tlb_entry.lo1 & 0xffff_fffe) | global;
    core.cp0.page_mask = tlb_entry.page_mask;
    core.cp0.hi = tlb_entry.hi;

    debug!("  CP0 LO0: {:08X}", core.cp0.lo0);
    debug!("  CP0 LO1: {:08X}", core.cp0.lo1);
    debug!("  CP0 Page Mask: {:08X}", core.cp0.page_mask);
    debug!("  CP0 HI: {:08X}", core.cp0.hi);
}

fn tlbwi(core: &mut Core<impl Bus>) {
    debug!("{:08X} TLBWI", core.pc);

    let tlb_entry = &mut core.cp0.tlb_entries[core.cp0.index as usize];
    tlb_entry.lo0 = core.cp0.lo0;
    tlb_entry.lo1 = core.cp0.lo1;
    tlb_entry.page_mask = core.cp0.page_mask;
    tlb_entry.hi = core.cp0.hi & !core.cp0.page_mask;

    debug!("TLB Entry {}: {:X?}", core.cp0.index, tlb_entry);
}

fn tlbp(core: &mut Core<impl Bus>) {
    debug!("{:08X} TLBP", core.pc);

    let index = core.cp0.tlb_entries.iter().position(|entry| {
        let mask = if ((entry.lo0 & entry.lo0) & 1) != 0 {
            // Global flag is set
            0xffff_e000
        } else {
            0xffff_e0ff
        };

        (entry.hi & mask) == (core.cp0.hi & mask)
    });

    if let Some(index) = index {
        core.cp0.index = index as u32;
    } else {
        core.cp0.index |= 0x8000_0000;
    }

    debug!("  CP0 Index: {}", core.cp0.index);
}

fn eret(core: &mut Core<impl Bus>) {
    debug!("{:08X} ERET", core.pc);

    if core.cp0.status.erl() {
        core.next[0] = core.cp0.error_epc;
        core.next[1] = core.next[0].wrapping_add(4);
        core.cp0.status.set_erl(false);
    } else {
        core.next[0] = core.cp0.epc;
        core.next[1] = core.next[0].wrapping_add(4);
        core.cp0.status.set_exl(false);
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::new().with_fr(true).with_cu0(true).with_cu1(true)
    }
}
