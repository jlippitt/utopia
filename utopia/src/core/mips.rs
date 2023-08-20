// Default CP0 implementation is exported
pub use cp0::Cp0;

use crate::util::facade::Value;
use cp1::Cp1;
use tracing::debug;

mod cp0;
mod cp1;
mod instruction;
mod operator;

pub const REGS: [&str; 32] = [
    "$ZERO", "$AT", "$V0", "$V1", "$A0", "$A1", "$A2", "$A3", "$T0", "$T1", "$T2", "$T3", "$T4",
    "$T5", "$T6", "$T7", "$S0", "$S1", "$S2", "$S3", "$S4", "$S5", "$S6", "$S7", "$T8", "$T9",
    "$K0", "$K1", "$GP", "$SP", "$FP", "$RA",
];

pub type Interrupt = u8;

pub trait Coprocessor0 {
    fn new() -> Self;
    fn dispatch(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32);
    fn update(_core: &mut Core<impl Bus<Cp0 = Self>>) {}
}

impl Coprocessor0 for () {
    fn new() -> Self {
        ()
    }

    fn dispatch(_core: &mut Core<impl Bus<Cp0 = Self>>, _word: u32) {
        unimplemented!("CP0");
    }
}

pub trait Coprocessor2 {
    fn new() -> Self;
    fn dispatch(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32);
}

impl Coprocessor2 for () {
    fn new() -> Self {
        ()
    }

    fn dispatch(_core: &mut Core<impl Bus<Cp2 = Self>>, _word: u32) {
        unimplemented!("CP2");
    }
}

pub trait Bus {
    type Cp0: Coprocessor0;
    type Cp2: Coprocessor2;

    const CP1: bool;
    const MUL_DIV: bool;
    const INSTR_64: bool;
    const PC_MASK: u32 = 0xffff_ffff;

    fn read<T: Value>(&mut self, address: u32) -> T;
    fn write<T: Value>(&mut self, address: u32, value: T);
    fn step(&mut self);
    fn poll(&self) -> Interrupt;

    fn read_opcode<T: Value>(&mut self, address: u32) -> T {
        self.read(address)
    }
}

pub struct Core<T: Bus> {
    pc: u32,
    next: [u32; 2],
    regs: [u64; 32],
    delay: bool,
    hi: u64,
    lo: u64,
    cp0: T::Cp0,
    cp1: Cp1,
    _cp2: T::Cp2,
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
            pc,
            next: [pc.wrapping_add(4), pc.wrapping_add(8)],
            regs: initial_state.regs,
            delay: false,
            hi: 0,
            lo: 0,
            cp0: T::Cp0::new(),
            cp1: Cp1::new(),
            _cp2: T::Cp2::new(),
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
        self.pc = pc;
        self.next[0] = pc.wrapping_add(4) & T::PC_MASK;
        self.next[1] = pc.wrapping_add(8) & T::PC_MASK;
    }

    pub fn step(&mut self) {
        debug_assert!((self.pc & 3) == 0);

        let word = self.bus.read_opcode::<u32>(self.pc);

        instruction::dispatch(self, word);

        self.bus.step();

        T::Cp0::update(self);

        self.pc = self.next[0];
        self.next[0] = self.next[1];
        self.next[1] = self.next[1].wrapping_add(4) & T::PC_MASK;
        self.delay = false;
    }

    // TODO: Review need for 'pub'
    pub fn get(&self, reg: usize) -> u32 {
        self.regs[reg] as u32
    }

    fn getd(&self, reg: usize) -> u64 {
        self.regs[reg]
    }

    // TODO: Review need for 'pub'
    pub fn set(&mut self, reg: usize, value: u32) {
        if reg == 0 {
            return;
        }

        self.regs[reg] = value as i32 as i64 as u64;
        debug!("  {}: {:08X}", REGS[reg], value);
    }

    fn setd(&mut self, reg: usize, value: u64) {
        if !T::INSTR_64 {
            self.set(reg, value as u32);
            return;
        }

        if reg == 0 {
            return;
        }

        self.regs[reg] = value;
        debug!("  {}: {:016X}", REGS[reg], value);
    }

    fn set_lo(&mut self, value: u32) {
        debug_assert!(T::MUL_DIV);
        self.lo = value as i32 as i64 as u64;
        debug!("  LO: {:08X}", value);
    }

    fn setd_lo(&mut self, value: u64) {
        debug_assert!(T::MUL_DIV);
        debug_assert!(T::INSTR_64);
        self.lo = value;
        debug!("  LO: {:016X}", value);
    }

    fn set_hi(&mut self, value: u32) {
        debug_assert!(T::MUL_DIV);
        self.hi = value as i32 as i64 as u64;
        debug!("  HI: {:08X}", value);
    }

    fn setd_hi(&mut self, value: u64) {
        debug_assert!(T::MUL_DIV);
        debug_assert!(T::INSTR_64);
        self.hi = value;
        debug!("  HI: {:016X}", value);
    }

    fn read_byte(&mut self, address: u32) -> u8 {
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:02X}", address, value);
        value
    }

    fn read_halfword(&mut self, address: u32) -> u16 {
        debug_assert!((address & 1) == 0);
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:04X}", address, value);
        value
    }

    fn read_word(&mut self, address: u32) -> u32 {
        debug_assert!((address & 3) == 0);
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:08X}", address, value);
        value
    }

    fn read_doubleword(&mut self, address: u32) -> u64 {
        debug_assert!(T::INSTR_64);
        debug_assert!((address & 3) == 0);
        let high = self.read_word(address);
        let low = self.read_word(address.wrapping_add(4));
        ((high as u64) << 32) | (low as u64)
    }

    fn write_byte(&mut self, address: u32, value: u8) {
        debug!("  [{:08X}] <= {:02X}", address, value);
        self.bus.write(address, value);
    }

    fn write_halfword(&mut self, address: u32, value: u16) {
        debug_assert!((address & 1) == 0);
        debug!("  [{:08X}] <= {:04X}", address, value);
        self.bus.write(address, value);
    }

    fn write_word(&mut self, address: u32, value: u32) {
        debug_assert!((address & 3) == 0);
        debug!("  [{:08X}] <= {:08X}", address, value);
        self.bus.write(address, value);
    }

    fn write_doubleword(&mut self, address: u32, value: u64) {
        debug_assert!(T::INSTR_64);
        debug_assert!((address & 3) == 0);
        self.write_word(address, (value >> 32) as u32);
        self.write_word(address.wrapping_add(4), value as u32);
    }

    fn jump_now(&mut self, address: u32) {
        self.next[0] = address & T::PC_MASK;
        self.next[1] = address.wrapping_add(4) & T::PC_MASK;
    }

    fn jump_delayed(&mut self, address: u32) {
        self.next[1] = address & T::PC_MASK;
        self.delay = true;
    }
}
