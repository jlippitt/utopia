use crate::util::Primitive;

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
        let opcode = self.bus.read::<u32>(self.pc);
        panic!("{:08X}: {:08X}", self.pc, opcode);
        //self.pc = self.pc.wrapping_add(1);
    }
}
