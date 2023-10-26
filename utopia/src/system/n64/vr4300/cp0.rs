use super::super::mips::opcode::RType;
use super::super::mips::{self, Bus, Core, Cp1, GPR};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use registers::{Cause, Index, Status};
use tlb::TlbEntry;
use tracing::trace;

mod registers;
mod tlb;

const EXCEPTION_HANDLER: u32 = 0x8000_0180;
const INT_TIMER: u8 = 0x80;

#[repr(usize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
enum Cpr {
    Index,
    Random,
    EntryLo0,
    EntryLo1,
    Context,
    PageMask,
    Wired,
    R7,
    BadVAddr,
    Count,
    EntryHi,
    Compare,
    Status,
    Cause,
    Epc,
    PRId,
    Config,
    LLAddr,
    WatchLo,
    WatchHi,
    XContext,
    R21,
    R22,
    R23,
    R24,
    R25,
    ParityError,
    CacheError,
    TagLo,
    TagHi,
    ErrorEpc,
    R31,
}

pub struct Cp0 {
    regs: [u32; 32],
    tlb: [Option<TlbEntry>; 32],
    pending: u8,
}

impl Cp0 {
    pub fn new() -> Self {
        let mut regs = [0; 32];

        // Set by IPL1
        regs[Cpr::Status as usize] = 0x3400_0000; // Status
        regs[Cpr::Config as usize] = 0x0006_e463; // Config

        Self {
            regs,
            tlb: [None; 32],
            pending: 0,
        }
    }

    fn getw(&self, cpr: Cpr) -> u32 {
        match cpr {
            Cpr::Index
            | Cpr::Count
            | Cpr::EntryLo0
            | Cpr::EntryLo1
            | Cpr::EntryHi
            | Cpr::PageMask
            | Cpr::Compare
            | Cpr::Status
            | Cpr::Cause
            | Cpr::Epc => self.regs[cpr as usize],
            _ => todo!("R4300 CPR Read: {} ({:?})", cpr as usize, cpr),
        }
    }

    fn setw(&mut self, cpr: Cpr, value: u32) {
        self.regs[cpr as usize] = value;

        match cpr {
            Cpr::Index => trace!("  Index: {:?}", Index::from(value)),
            Cpr::Status => trace!("  Status: {:?}", Status::from(value)),
            Cpr::Cause => {
                let cause = Cause::from(value);
                trace!("  Cause: {:?}", Cause::from(value));
                // Software interrupt bits
                self.pending = (self.pending & !3) | (cause.ip() & 3);
            }
            Cpr::Compare => {
                self.pending &= !INT_TIMER;
            }
            _ => trace!("  {:?}: {:08X}", cpr, value),
        }
    }

    fn update_counter(&mut self) {
        let count = self.getw(Cpr::Count).wrapping_add(1);

        // Don't use 'setw' as we don't want a log message every time the counter updates(!)
        self.regs[Cpr::Count as usize] = count;

        if count == self.getw(Cpr::Compare) {
            self.pending |= INT_TIMER;
            trace!("CP0 Timer Interrupt Raised");
        }
    }
}

impl mips::Cp0 for Cp0 {
    fn translate(&self, address: u32) -> u32 {
        if (address & 0xc000_0000) == 0x8000_0000 {
            address & 0x1fff_ffff
        } else {
            todo!("TLB");
        }
    }

    fn mfc0(core: &mut Core<impl Bus<Cp0 = Cp0>>, word: u32) {
        let op = RType::from(word);
        let cpr = Cpr::from_usize(op.rd()).unwrap();
        trace!("{:08X} MFC0 {}, {:?}", core.pc(), GPR[op.rt()], cpr);
        let value = core.cp0().getw(cpr);
        core.setw(op.rt(), value);
    }

    fn mtc0(core: &mut Core<impl Bus<Cp0 = Cp0>>, word: u32) {
        let op = RType::from(word);
        let cpr = Cpr::from_usize(op.rd()).unwrap();
        trace!("{:08X} MTC0 {}, {:?}", core.pc(), GPR[op.rt()], cpr);
        let value = core.getw(op.rt());
        core.cp0_mut().setw(cpr, value);

        if cpr == Cpr::Status {
            core.cp1_mut().set_fr(Status::from(value).fr());
        }
    }

    fn cop0(core: &mut Core<impl Bus<Cp0 = Cp0>>, word: u32) {
        match word & 0o77 {
            0o02 => tlb::tlbwi(core, word),
            0o10 => tlb::tlbp(core, word),
            0o30 => eret(core, word),
            func => unimplemented!("R4300 COP0 Function {:02o} [PC:{:08X}]", func, core.pc()),
        }
    }

    fn syscall(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("R4300 SYSCALL");
    }

    fn break_(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("R4300 BREAK");
    }

    fn step(core: &mut Core<impl Bus<Cp0 = Cp0>>) {
        core.cp0_mut().update_counter();

        let mut status = Status::from(core.cp0().getw(Cpr::Status));

        if !status.ie() || status.erl() || status.exl() {
            return;
        }

        let mut cause = Cause::from(core.cp0().getw(Cpr::Cause));

        let pending = core.cp0().pending | core.bus().poll();

        cause.set_ip(pending);

        let active = pending & status.im();

        if active != 0 {
            trace!("-- Exception: {:08b} --", active);

            status.set_exl(true);

            cause.set_exc_code(0); // 0 = Interrupt
            cause.set_bd(core.is_delay());

            let epc = core.restart_location();

            {
                let cp0 = core.cp0_mut();
                cp0.setw(Cpr::Status, status.into());
                cp0.setw(Cpr::Cause, cause.into());
                cp0.setw(Cpr::Epc, epc);
            }

            core.jump_now(EXCEPTION_HANDLER);
        } else {
            // Again we want to avoid logging here
            core.cp0_mut().regs[Cpr::Cause as usize] = cause.into();
        }
    }
}

pub fn eret(core: &mut Core<impl Bus<Cp0 = Cp0>>, _word: u32) {
    trace!("{:08X} ERET", core.pc());

    let mut status = Status::from(core.cp0().getw(Cpr::Status));

    if status.erl() {
        status.set_erl(false);
        core.jump_now(core.cp0().regs[Cpr::ErrorEpc as usize]);
    } else {
        status.set_exl(false);
        core.jump_now(core.cp0().regs[Cpr::Epc as usize]);
    }

    core.cp0_mut().setw(Cpr::Status, status.into());

    // TODO: LLBit
}
