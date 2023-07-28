use crate::util::Primitive;

mod instruction;

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
            0b001111 => self.type_i(instr::lui, word),
            0b010000 => self.cop::<0>(word),
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
        let value = word & 0xff;
        instr(self, rs, rt, value);
    }
}
