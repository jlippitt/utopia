use crate::util::facade::Value;
use tracing::debug;

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
    regs: [u32; 32],
    hi_lo: u64,
    bus: T,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct State {
    pub pc: u32,
    pub regs: [u32; 32],
    pub hi_lo: u64,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T, initial_state: State) -> Self {
        let pc = initial_state.pc;

        Self {
            pc: 0,
            next: [pc, pc.wrapping_add(4)],
            regs: initial_state.regs,
            hi_lo: 0,
            bus,
        }
    }

    pub fn step(&mut self) {
        use instruction as instr;
        use operator as op;

        self.pc = self.next[0];
        self.next[0] = self.next[1];
        self.next[1] = self.next[1].wrapping_add(4);

        assert!((self.pc & 3) == 0);

        let word = self.bus.read::<u32>(self.pc);

        match word >> 26 {
            0o00 => self.special(word),
            0o01 => self.regimm(word),
            0o03 => self.type_j(instr::jal, word),
            0o04 => self.type_i(instr::branch::<op::Beq, false>, word),
            0o05 => self.type_i(instr::branch::<op::Bne, false>, word),
            0o06 => self.type_i(instr::branch::<op::Blez, false>, word),
            0o07 => self.type_i(instr::branch::<op::Bgtz, false>, word),
            0o10 => self.type_i(instr::addi, word),
            0o11 => self.type_i(instr::addiu, word),
            0o12 => self.type_i(instr::slti, word),
            0o13 => self.type_i(instr::sltiu, word),
            0o14 => self.type_i(instr::andi, word),
            0o15 => self.type_i(instr::ori, word),
            0o16 => self.type_i(instr::xori, word),
            0o17 => self.type_i(instr::lui, word),
            0o20 => self.cop::<0>(word),
            0o24 => self.type_i(instr::branch::<op::Beq, true>, word),
            0o25 => self.type_i(instr::branch::<op::Bne, true>, word),
            0o26 => self.type_i(instr::branch::<op::Blez, true>, word),
            0o27 => self.type_i(instr::branch::<op::Bgtz, true>, word),
            0o43 => self.type_i(instr::lw, word),
            0o44 => self.type_i(instr::lbu, word),
            0o50 => self.type_i(instr::sb, word),
            0o53 => self.type_i(instr::sw, word),
            0o57 => self.type_i(instr::cache, word),
            opcode => unimplemented!("Opcode {:02o} ({:08X})", opcode, self.pc),
        }
    }

    fn special(&mut self, word: u32) {
        use instruction as instr;

        match word & 0o77 {
            0o00 => self.type_r(instr::sll, word),
            0o02 => self.type_r(instr::srl, word),
            0o10 => self.type_r(instr::jr, word),
            0o20 => self.type_r(instr::mfhi, word),
            0o22 => self.type_r(instr::mflo, word),
            0o31 => self.type_r(instr::multu, word),
            0o40 => self.type_r(instr::add, word),
            0o41 => self.type_r(instr::addu, word),
            0o42 => self.type_r(instr::sub, word),
            0o43 => self.type_r(instr::subu, word),
            0o44 => self.type_r(instr::and, word),
            0o45 => self.type_r(instr::or, word),
            0o46 => self.type_r(instr::xor, word),
            0o52 => self.type_r(instr::slt, word),
            0o53 => self.type_r(instr::sltu, word),
            function => unimplemented!("SPECIAL FN={:02o} ({:08X})", function, self.pc),
        }
    }

    fn regimm(&mut self, word: u32) {
        use instruction as instr;
        use operator as op;

        match (word >> 16) & 31 {
            0b00000 => self.type_i(instr::branch::<op::Bltz, false>, word),
            0b00001 => self.type_i(instr::branch::<op::Bgez, false>, word),
            0b00010 => self.type_i(instr::branch::<op::Bltz, true>, word),
            0b00011 => self.type_i(instr::branch::<op::Bgez, true>, word),
            rt => unimplemented!("REGIMM RT={:05b} ({:08X})", rt, self.pc),
        }
    }
    fn cop<const COP: usize>(&mut self, word: u32) {
        use instruction as instr;

        match (word >> 21) & 31 {
            0b00100 => self.type_r(instr::mtc::<COP>, word),
            rs => unimplemented!("COP{} RS={:05b} ({:08X})", COP, rs, self.pc),
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

    fn type_j(&mut self, instr: impl Fn(&mut Core<T>, u32), word: u32) {
        let value = word & 0x03ff_ffff;
        instr(self, value);
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
