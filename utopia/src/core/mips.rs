use crate::util::Primitive;

mod instruction;

pub trait Bus {
    fn read<T: Primitive>(&mut self, address: u32) -> T;
}

pub struct Core<T: Bus> {
    pc: u32,
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
            bus,
        }
    }

    pub fn step(&mut self) {
        let word = self.bus.read::<u32>(self.pc);

        match word >> 26 {
            0b010000 => self.cop::<0>(word),
            opcode => unimplemented!("Opcode {:06b}", opcode),
        }

        self.pc = self.pc.wrapping_add(1);
    }

    fn cop<const COP: usize>(&mut self, word: u32) {
        use instruction as instr;

        match (word >> 21) & 31 {
            0b00100 => instr::mtc::<COP>(self, word),
            rs => unimplemented!("COP{} RS: {:06b}", COP, rs),
        }
    }
}
