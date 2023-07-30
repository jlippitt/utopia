pub trait Bus {}

pub struct Core<T: Bus> {
    _bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self { _bus: bus }
    }

    pub fn step(&mut self) {
        todo!("ARM7 instructions");
    }
}
