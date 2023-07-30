use crate::util::facade::Value;
use tracing::debug;

pub trait Bus {
    fn read<T: Value>(&mut self, address: u32) -> T;
}

pub struct Core<T: Bus> {
    bus: T,
    pc: u32,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self { bus, pc: 0 }
    }

    pub fn step(&mut self) {
        let word: u32 = self.bus.read(self.pc);
        debug!("{:08X}", word);

        todo!("ARM7 instructions");
    }
}
