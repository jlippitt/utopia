use crate::util::facade::Value;
use tracing::debug;

mod instruction;

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
    regs: [u32; 32],
    bus: T,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct State {
    pub pc: u32,
    pub regs: [u32; 32],
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T, initial_state: State) -> Self {
        let pc = initial_state.pc;

        Self {
            pc: 0,
            next: [pc, pc.wrapping_add(4)],
            regs: initial_state.regs,
            bus,
        }
    }

    pub fn step(&mut self) {
        use instruction as instr;

        self.pc = self.next[0];
        self.next[0] = self.next[1];
        self.next[1] = self.next[1].wrapping_add(4);

        assert!((self.pc & 3) == 0);

        let word = self.bus.read::<u32>(self.pc);

        match word >> 26 {
            0o00 => self.special(word),
            0o05 => self.type_i(instr::bne, word),
            0o10 => self.type_i(instr::addi, word),
            0o11 => self.type_i(instr::addiu, word),
            0o15 => self.type_i(instr::ori, word),
            0o17 => self.type_i(instr::lui, word),
            0o20 => self.cop::<0>(word),
            0o43 => self.type_i(instr::lw, word),
            0o53 => self.type_i(instr::sw, word),
            opcode => unimplemented!("Opcode {:02o} ({:08X})", opcode, self.pc),
        }
    }

    fn special(&mut self, word: u32) {
        use instruction as instr;

        match word & 0o77 {
            0o00 => self.type_r(instr::sll, word),
            0o45 => self.type_r(instr::or, word),
            function => unimplemented!("SPECIAL FN={:02o} ({:08X})", function, self.pc),
        }
    }

    fn cop<const COP: usize>(&mut self, word: u32) {
        use instruction as instr;

        match (word >> 21) & 31 {
            0b00100 => self.type_r(instr::mtc::<COP>, word),
            rs => unimplemented!("COP{} RS={:06b} ({:08X})", COP, rs, self.pc),
        }
    }

    fn type_r(&mut self, instr: impl Fn(&mut Core<T>, usize, usize, usize, u32), word: u32) {
        let rs = ((word >> 21) & 31) as usize;
        let rt = ((word >> 16) & 31) as usize;
        let rd = ((word >> 11) & 31) as usize;
        let sa = (word >> 6) & 31;
        instr(self, rs, rt, rd, sa);
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
        if reg == 0 {
            return;
        }

        self.regs[reg] = value;
        debug!("  {}: {:08X}", REGS[reg], value);
    }

    fn read_word(&mut self, address: u32) -> u32 {
        assert!((address & 3) == 0);
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:08X}", address, value);
        value
    }

    fn write_word(&mut self, address: u32, value: u32) {
        assert!((address & 3) == 0);
        debug!("  [{:08X}] <= {:08X}", address, value);
        self.bus.write(address, value);
    }
}
