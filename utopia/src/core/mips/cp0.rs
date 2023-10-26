use super::{Bus, Coprocessor0, Core, Interrupt, REGS};
use bitfield_struct::bitfield;
use tracing::trace;

const INT_TIMER: Interrupt = 0x80;

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
    compare: u32,
    status: Status,
    cause: Cause,
    epc: u32,
    error_epc: u32,
    int_active: u8,
    tlb_entries: [TlbEntry; 32],
}

impl Cp0 {
    const REGS: [&'static str; 32] = [
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

    pub fn new() -> Self {
        Self::default()
    }
}

impl Coprocessor0 for Cp0 {
    fn mfc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;
        let cp0 = &core.cp0;

        trace!("{:08X} MFC0 {}, {}", core.pc, REGS[rt], Self::REGS[rd]);

        let result = match rd {
            0 => cp0.index,
            2 => cp0.lo0,
            3 => cp0.lo1,
            5 => cp0.page_mask,
            6 => cp0.wired,
            9 => cp0.count,
            10 => cp0.hi,
            11 => cp0.compare,
            12 => u32::from(cp0.status) & !0x0088_0000,
            13 => cp0.cause.into(),
            14 => cp0.epc,
            _ => todo!("CP0 Register Read: {}", Self::REGS[rd]),
        };

        core.set(rt, result);
    }

    fn mtc0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;
        let value = core.get(rt);
        let cp0 = &mut core.cp0;

        trace!("{:08X} MTC0 {}, {}", core.pc, REGS[rt], Self::REGS[rd]);

        match rd {
            0 => {
                cp0.index = value & 0x8000_003f;
                trace!("  CP0 Index: {}", cp0.index);
            }
            2 => {
                cp0.lo0 = value;
                trace!("  CP0 LO0: {:08X}", cp0.lo0);
            }
            3 => {
                cp0.lo1 = value;
                trace!("  CP0 LO1: {:08X}", cp0.lo1);
            }
            5 => {
                cp0.page_mask = value & 0x01ff_e000;
                trace!("  CP0 Page Mask: {:08X}", cp0.page_mask);
            }
            6 => {
                cp0.wired = value & 0x0000_003f;
                trace!("  CP0 Wired: {:08X}", cp0.wired);
            }
            10 => {
                cp0.hi = value;
                trace!("  CP0 HI: {:08X}", cp0.hi);
            }
            11 => {
                cp0.compare = value;
                trace!("  CP0 Compare: {:08X}", cp0.compare);
                cp0.int_active &= !INT_TIMER;
            }
            12 => {
                cp0.status = value.into();
                trace!("  CP0 Status: {:?}", cp0.status);
                core.cp1.set_reg_size(core.cp0.status.fr());
            }
            14 => {
                cp0.epc = value;
                trace!("  CP0 EPC: {:08X}", cp0.epc);
            }
            30 => {
                cp0.error_epc = value;
                trace!("  CP0 Error EPC: {:08X}", cp0.error_epc);
            }
            _ => {
                if value != 0 {
                    todo!("CP0 Register Write: {} <= {:08X}", Self::REGS[rd], value);
                }
            }
        }
    }

    fn cop0(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        match word & 63 {
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
        }
    }

    fn break_(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("BREAK");
    }

    fn step(core: &mut Core<impl Bus<Cp0 = Self>>) {
        core.cp0.count = core.cp0.count.wrapping_add(1);

        if core.cp0.count == core.cp0.compare {
            core.cp0.int_active |= INT_TIMER;
        }

        // Test for interrupts
        let cp0 = &mut core.cp0;

        // If IE=0, EXL=1 or ERL=1, no interrupt for you
        if (u32::from(cp0.status) ^ 0x01) & 0x07 != 0 {
            return;
        }

        let int_active = (cp0.int_active | core.bus.poll()) & cp0.status.im();

        if int_active == 0 {
            return;
        }

        // Handle interrupt exception
        trace!("-- Exception: {:08b} --", int_active);

        let int_pending = (cp0.cause.ip() & 0x03) | (int_active & 0xfc);
        cp0.cause.set_ip(int_pending);

        cp0.cause.set_exc_code(0);
        cp0.cause.set_bd(core.delay);

        core.cp0.epc = if core.delay {
            core.next[0].wrapping_sub(4)
        } else {
            core.next[0]
        };

        core.cp0.status.set_exl(true);

        core.jump_now(0x8000_0180);
    }
}

fn tlbr(core: &mut Core<impl Bus<Cp0 = Cp0>>) {
    trace!("{:08X} TLBR", core.pc);

    let tlb_entry = &core.cp0.tlb_entries[core.cp0.index as usize];

    let global = tlb_entry.lo0 & tlb_entry.lo1 & 1;
    core.cp0.lo0 = (tlb_entry.lo0 & 0xffff_fffe) | global;
    core.cp0.lo1 = (tlb_entry.lo1 & 0xffff_fffe) | global;
    core.cp0.page_mask = tlb_entry.page_mask;
    core.cp0.hi = tlb_entry.hi;

    trace!("  CP0 LO0: {:08X}", core.cp0.lo0);
    trace!("  CP0 LO1: {:08X}", core.cp0.lo1);
    trace!("  CP0 Page Mask: {:08X}", core.cp0.page_mask);
    trace!("  CP0 HI: {:08X}", core.cp0.hi);
}

fn tlbwi(core: &mut Core<impl Bus<Cp0 = Cp0>>) {
    trace!("{:08X} TLBWI", core.pc);

    let tlb_entry = &mut core.cp0.tlb_entries[core.cp0.index as usize];
    tlb_entry.lo0 = core.cp0.lo0;
    tlb_entry.lo1 = core.cp0.lo1;
    tlb_entry.page_mask = core.cp0.page_mask;
    tlb_entry.hi = core.cp0.hi & !core.cp0.page_mask;

    trace!("TLB Entry {}: {:X?}", core.cp0.index, tlb_entry);
}

fn tlbp(core: &mut Core<impl Bus<Cp0 = Cp0>>) {
    trace!("{:08X} TLBP", core.pc);

    let index = core.cp0.tlb_entries.iter().position(|entry| {
        let mask = if ((entry.lo0 & entry.lo1) & 1) != 0 {
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

    trace!("  CP0 Index: {}", core.cp0.index);
}

fn eret(core: &mut Core<impl Bus<Cp0 = Cp0>>) {
    trace!("{:08X} ERET", core.pc);

    if core.cp0.status.erl() {
        core.jump_now(core.cp0.error_epc);
        core.cp0.status.set_erl(false);
    } else {
        core.jump_now(core.cp0.epc);
        core.cp0.status.set_exl(false);
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::new().with_fr(true).with_cu0(true).with_cu1(true)
    }
}
