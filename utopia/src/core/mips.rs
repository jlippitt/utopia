use crate::util::facade::Value;
use cop0::Cop0;
use cop1::Cop1;
use tracing::debug;

mod cop0;
mod cop1;
mod instruction;
mod operator;

const REGS: [&str; 32] = [
    "$ZERO", "$AT", "$V0", "$V1", "$A0", "$A1", "$A2", "$A3", "$T0", "$T1", "$T2", "$T3", "$T4",
    "$T5", "$T6", "$T7", "$S0", "$S1", "$S2", "$S3", "$S4", "$S5", "$S6", "$S7", "$T8", "$T9",
    "$K0", "$K1", "$GP", "$SP", "$FP", "$RA",
];

pub trait Bus {
    fn read<T: Value>(&mut self, address: u32) -> T;
    fn write<T: Value>(&mut self, address: u32, value: T);
}

pub struct Core<T: Bus> {
    pc: u32,
    next: [u32; 2],
    regs: [u64; 32],
    hi: u64,
    lo: u64,
    cop0: Cop0,
    cop1: Cop1,
    bus: T,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct State {
    pub pc: u32,
    pub regs: [u64; 32],
    pub hi_lo: u64,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T, initial_state: State) -> Self {
        let pc = initial_state.pc;

        Self {
            pc: 0,
            next: [pc, pc.wrapping_add(4)],
            regs: initial_state.regs,
            hi: 0,
            lo: 0,
            cop0: Cop0::default(),
            cop1: Cop1::default(),
            bus,
        }
    }

    pub fn step(&mut self) {
        self.pc = self.next[0];
        self.next[0] = self.next[1];
        self.next[1] = self.next[1].wrapping_add(4);

        assert!((self.pc & 3) == 0);

        let word = self.bus.read::<u32>(self.pc);
        instruction::dispatch(self, word);
    }

    fn get(&self, reg: usize) -> u32 {
        self.regs[reg] as u32
    }

    fn set(&mut self, reg: usize, value: u32) {
        if reg == 0 {
            return;
        }

        self.regs[reg] = value as i32 as i64 as u64;
        debug!("  {}: {:08X}", REGS[reg], value);
    }

    fn setd(&mut self, reg: usize, value: u64) {
        if reg == 0 {
            return;
        }

        self.regs[reg] = value;
        debug!("  {}: {:016X}", REGS[reg], value);
    }

    fn lo(&self) -> u32 {
        self.lo as u32
    }

    fn set_lo(&mut self, value: u32) {
        self.lo = value as i32 as i64 as u64;
        debug!("  LO: {:08X}", value);
    }

    fn hi(&self) -> u32 {
        self.hi as u32
    }

    fn set_hi(&mut self, value: u32) {
        self.hi = value as i32 as i64 as u64;
        debug!("  HI: {:08X}", value);
    }

    fn read_byte(&mut self, address: u32) -> u8 {
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:02X}", address, value);
        value
    }

    fn read_word(&mut self, address: u32) -> u32 {
        assert!((address & 3) == 0);
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:08X}", address, value);
        value
    }

    fn read_doubleword(&mut self, address: u32) -> u64 {
        assert!((address & 3) == 0);
        let high = self.read_word(address);
        let low = self.read_word(address.wrapping_add(4));
        ((high as u64) << 32) | (low as u64)
    }

    fn write_byte(&mut self, address: u32, value: u8) {
        debug!("  [{:08X}] <= {:02X}", address, value);
        self.bus.write(address, value);
    }

    fn write_word(&mut self, address: u32, value: u32) {
        assert!((address & 3) == 0);
        debug!("  [{:08X}] <= {:08X}", address, value);
        self.bus.write(address, value);
    }
}
