use crate::util::Primitive;
use tracing::debug;

mod instruction;

const REGS: [&str; 32] = [
    "$ZERO", "$AT", "$V0", "$V1", "$A0", "$A1", "$A2", "$A3", "$T0", "$T1", "$T2", "$T3", "$T4",
    "$T5", "$T6", "$T7", "$S0", "$S1", "$S2", "$S3", "$S4", "$S5", "$S6", "$S7", "$T8", "$T9",
    "$K0", "$K1", "$GP", "$SP", "$FP", "$RA",
];

pub trait Bus {
    fn read<T: Primitive>(&mut self, address: u32) -> T;
}

pub struct Core<T: Bus> {
    pc: u32,
    regs: [u32; 32],
    bus: T,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct State {
    pub pc: u32,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T, initial_state: State) -> Self {
        Self {
            pc: initial_state.pc,
            regs: [0; 32],
            bus,
        }
    }

    pub fn step(&mut self) {
        use instruction as instr;

        let word = self.bus.read::<u32>(self.pc);

        match word >> 26 {
            0b001001 => self.type_i(instr::addiu, word),
            0b001111 => self.type_i(instr::lui, word),
            0b010000 => self.cop::<0>(word),
            0b100011 => self.type_i(instr::lw, word),
            opcode => unimplemented!("Opcode {:06b}", opcode),
        }

        self.pc = self.pc.wrapping_add(4);
    }

    fn cop<const COP: usize>(&mut self, word: u32) {
        use instruction as instr;

        match (word >> 21) & 31 {
            0b00100 => self.type_r(instr::mtc::<COP>, word),
            rs => unimplemented!("COP{} RS: {:06b}", COP, rs),
        }
    }

    fn type_r(&mut self, instr: impl Fn(&mut Core<T>, usize, usize, usize), word: u32) {
        let rs = ((word >> 21) & 31) as usize;
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;
        instr(self, rs, rt, rd);
    }

    fn type_i(&mut self, instr: impl Fn(&mut Core<T>, usize, usize, u32), word: u32) {
        let rs = ((word >> 21) & 31) as usize;
        let rt = ((word >> 16) & 31) as usize;
        let value = word & 0xffff;
        instr(self, rs, rt, value);
    }

    fn get(&self, reg: usize) -> u32 {
        self.regs[reg]
    }

    fn set(&mut self, reg: usize, value: u32) {
        self.regs[reg] = value;
        debug!("  {}: {:08X}", REGS[reg], value);
    }

    fn read_word(&mut self, address: u32) -> u32 {
        assert!((address & 3) == 0);
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:08X}", address, value);
        value
    }
}
