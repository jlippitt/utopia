pub use coprocessor::{Cp0, Cp1, Cp2, NullCp1, NullCp2};

use super::memory::Value;
use tracing::trace;

pub mod opcode;

mod coprocessor;
mod instruction;

pub const GPR: [&str; 32] = [
    "ZERO", "AT", "V0", "V1", "A0", "A1", "A2", "A3", "T0", "T1", "T2", "T3", "T4", "T5", "T6",
    "T7", "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "T8", "T9", "K0", "K1", "GP", "SP", "FP",
    "RA",
];

pub trait Bus {
    const NAME: &'static str;
    const ENABLE_64_BIT: bool;
    const ENABLE_MUL_DIV: bool;
    const ENABLE_LIKELY_BRANCH: bool;
    const FORCE_MEMORY_ALIGNMENT: bool;
    const PC_MASK: u32 = 0xffff_ffff;

    type Cp0: Cp0;
    type Cp1: Cp1;
    type Cp2: Cp2;

    fn read_data<T: Value>(&self, address: u32) -> T;
    fn write_data<T: Value>(&mut self, address: u32, value: T);
    fn step(&mut self);
    fn poll(&self) -> u8;

    fn read_opcode(&self, address: u32) -> u32 {
        self.read_data(address)
    }
}

pub trait UnalignedBus {
    fn read_data_unaligned<T: Value>(&self, address: u32) -> T;
    fn write_data_unaligned<T: Value>(&mut self, address: u32, value: T);
}

#[derive(Default)]
pub struct InitialState {
    pub pc: u32,
    pub regs: [u64; 32],
}

pub struct Core<T: Bus> {
    pc: u32,
    next: [u32; 2],
    delay: bool,
    regs: [u64; 32],
    hi: u64,
    lo: u64,
    cp0: T::Cp0,
    cp1: T::Cp1,
    cp2: T::Cp2,
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T, cp0: T::Cp0, cp1: T::Cp1, cp2: T::Cp2, initial_state: InitialState) -> Self {
        let InitialState { pc, regs } = initial_state;

        Self {
            pc: pc & T::PC_MASK,
            next: [
                pc.wrapping_add(4) & T::PC_MASK,
                pc.wrapping_add(8) & T::PC_MASK,
            ],
            delay: false,
            regs,
            hi: 0,
            lo: 0,
            cp0,
            cp1,
            cp2,
            bus,
        }
    }

    pub fn bus(&self) -> &T {
        &self.bus
    }

    pub fn bus_mut(&mut self) -> &mut T {
        &mut self.bus
    }

    pub fn pc(&self) -> u32 {
        self.pc
    }

    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc & T::PC_MASK;
        self.next[0] = pc.wrapping_add(4) & T::PC_MASK;
        self.next[1] = pc.wrapping_add(8) & T::PC_MASK;
    }

    pub fn cp0(&self) -> &T::Cp0 {
        &self.cp0
    }

    pub fn cp0_mut(&mut self) -> &mut T::Cp0 {
        &mut self.cp0
    }

    pub fn cp1(&self) -> &T::Cp1 {
        &self.cp1
    }

    pub fn cp1_mut(&mut self) -> &mut T::Cp1 {
        &mut self.cp1
    }

    pub fn cp2(&self) -> &T::Cp2 {
        &self.cp2
    }

    pub fn cp2_mut(&mut self) -> &mut T::Cp2 {
        &mut self.cp2
    }

    pub fn getw(&self, reg: usize) -> u32 {
        self.regs[reg] as u32
    }

    pub fn setw(&mut self, reg: usize, value: u32) {
        if reg == 0 {
            return;
        }

        self.regs[reg] = value as i32 as u64;
        trace!("  {}: {:08X}", GPR[reg], value);
    }

    pub fn getd(&self, reg: usize) -> u64 {
        self.regs[reg]
    }

    pub fn setd(&mut self, reg: usize, value: u64) {
        if !T::ENABLE_64_BIT {
            self.setw(reg, value as u32);
            return;
        }

        if reg == 0 {
            return;
        }

        self.regs[reg] = value;
        trace!("  {}: {:016X}", GPR[reg], value);
    }

    pub fn hi(&self) -> u64 {
        debug_assert!(T::ENABLE_MUL_DIV);
        self.hi
    }

    pub fn lo(&self) -> u64 {
        debug_assert!(T::ENABLE_MUL_DIV);
        self.lo
    }

    pub fn setw_hi(&mut self, value: u32) {
        debug_assert!(T::ENABLE_MUL_DIV);
        self.hi = value as i32 as u64;
        trace!("  HI: {:08X}", value);
    }

    pub fn setw_lo(&mut self, value: u32) {
        debug_assert!(T::ENABLE_MUL_DIV);
        self.lo = value as i32 as u64;
        trace!("  LO: {:08X}", value);
    }

    pub fn setd_hi(&mut self, value: u64) {
        debug_assert!(T::ENABLE_64_BIT && T::ENABLE_MUL_DIV);
        self.hi = value;
        trace!("  HI: {:016X}", value);
    }

    pub fn setd_lo(&mut self, value: u64) {
        debug_assert!(T::ENABLE_64_BIT && T::ENABLE_MUL_DIV);
        self.lo = value;
        trace!("  LO: {:016X}", value);
    }

    pub fn read_u8(&self, address: u32) -> u8 {
        let address = self.cp0.translate(address);
        let value = self.bus.read_data(address);
        trace!("  {:08X} => {:02X}", address, value);
        value
    }

    pub fn read_u16(&self, address: u32) -> u16 {
        let address = self.cp0.translate(address);
        debug_assert!(!T::FORCE_MEMORY_ALIGNMENT || (address & 1) == 0);
        let value = self.bus.read_data(address);
        trace!("  {:08X} => {:04X}", address, value);
        value
    }

    pub fn read_u32(&self, address: u32) -> u32 {
        let address = self.cp0.translate(address);
        debug_assert!(!T::FORCE_MEMORY_ALIGNMENT || (address & 3) == 0);
        let value = self.bus.read_data(address);
        trace!("  {:08X} => {:08X}", address, value);
        value
    }

    pub fn read_u64(&self, address: u32) -> u64 {
        let high = self.read_u32(address);
        let low = self.read_u32(address.wrapping_add(4));
        ((high as u64) << 32) | (low as u64)
    }

    pub fn write_u8(&mut self, address: u32, value: u8) {
        let address = self.cp0.translate(address);
        trace!("  {:08X} <= {:02X}", address, value);
        self.bus.write_data(address, value);
    }

    pub fn write_u16(&mut self, address: u32, value: u16) {
        let address = self.cp0.translate(address);
        debug_assert!(!T::FORCE_MEMORY_ALIGNMENT || (address & 1) == 0);
        trace!("  {:08X} <= {:04X}", address, value);
        self.bus.write_data(address, value);
    }

    pub fn write_u32(&mut self, address: u32, value: u32) {
        let address = self.cp0.translate(address);
        debug_assert!(!T::FORCE_MEMORY_ALIGNMENT || (address & 3) == 0);
        trace!("  {:08X} <= {:08X}", address, value);
        self.bus.write_data(address, value);
    }

    pub fn write_u64(&mut self, address: u32, value: u64) {
        self.write_u32(address, (value >> 32) as u32);
        self.write_u32(address.wrapping_add(4), value as u32);
    }

    pub fn branch_if<const LIKELY: bool>(&mut self, condition: bool, offset: i32) {
        if condition {
            trace!("  Branch taken");
            self.jump_delayed(self.next[0].wrapping_add(offset as u32))
        } else {
            trace!("  Branch not taken");
            if LIKELY {
                debug_assert!(T::ENABLE_LIKELY_BRANCH);
                // Skip delay slot
                self.next[0] = self.next[1];
                self.next[1] = self.next[1].wrapping_add(4) & T::PC_MASK;
            }
        }
    }

    pub fn trap_if(&mut self, condition: bool) {
        if condition {
            unimplemented!("Trap Exception");
        }
    }

    pub fn is_delay(&self) -> bool {
        self.delay
    }

    pub fn restart_location(&self) -> u32 {
        if self.delay {
            self.next[0].wrapping_sub(4) & T::PC_MASK
        } else {
            self.next[0]
        }
    }

    pub fn jump_delayed(&mut self, target: u32) {
        self.next[1] = target & T::PC_MASK;
        self.delay = true;
    }

    pub fn jump_now(&mut self, target: u32) {
        self.next[0] = target & T::PC_MASK;
        self.next[1] = target.wrapping_add(4) & T::PC_MASK;
    }

    pub fn step(&mut self) {
        debug_assert!((self.pc & 3) == 0);
        let word = self.bus.read_opcode(self.cp0.translate(self.pc));

        instruction::dispatch(self, word);

        self.bus.step();

        T::Cp0::step(self);

        self.pc = self.next[0];
        self.next[0] = self.next[1];
        self.next[1] = self.next[1].wrapping_add(4) & T::PC_MASK;
        self.delay = false;
    }
}
