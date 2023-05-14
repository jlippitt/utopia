pub trait Bus {
    //
}

pub struct Core<T: Bus> {
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self { bus }
    }

    pub fn step(&mut self) {
        println!("Step");
    }
}
