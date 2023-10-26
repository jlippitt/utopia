use super::registers::Index;
use super::{Bus, Core, Cp0, Cpr};
use tracing::trace;

#[allow(unused)]
#[derive(Copy, Clone, Debug)]
pub struct TlbEntry {
    lo0: u32,
    lo1: u32,
    hi: u32,
    page_mask: u32,
}

pub fn tlbwi(core: &mut Core<impl Bus<Cp0 = Cp0>>, _word: u32) {
    trace!("{:08X} TLBWI", core.pc());

    let cp0 = core.cp0_mut();
    let index = Index::from(cp0.getw(Cpr::Index)).index() as usize;
    let page_mask = cp0.regs[Cpr::PageMask as usize];

    let entry = TlbEntry {
        lo0: cp0.getw(Cpr::EntryLo0),
        lo1: cp0.getw(Cpr::EntryLo1),
        hi: cp0.getw(Cpr::EntryHi) & !page_mask,
        page_mask,
    };

    cp0.tlb[index] = Some(entry);
    trace!("  TLB {}: {:?}", index, entry);
}

pub fn tlbp(core: &mut Core<impl Bus<Cp0 = Cp0>>, _word: u32) {
    trace!("{:08X} TLBP", core.pc());

    let pos = core.cp0().tlb.iter().position(|entry| {
        entry.is_some_and(|entry| {
            let mask = if ((entry.lo0 & entry.lo1) & 1) != 0 {
                // Global flag is set
                0xffff_e000
            } else {
                0xffff_e0ff
            };

            (entry.hi & mask) == (core.cp0().getw(Cpr::EntryHi) & mask)
        })
    });

    let mut index = Index::new();

    if let Some(pos) = pos {
        index.set_index(pos as u32)
    } else {
        index.set_probe_failed(true)
    };

    core.cp0_mut().setw(Cpr::Index, index.into());
}
