pub struct Sequencer<const SIZE: usize> {
    sequence: &'static [u8; SIZE],
    index: usize,
}

impl<const SIZE: usize> Sequencer<SIZE> {
    pub fn new(sequence: &'static [u8; SIZE]) -> Self {
        Self { sequence, index: 0 }
    }

    pub fn sample(&self) -> u8 {
        self.sequence[self.index]
    }

    pub fn set_sequence(&mut self, sequence: &'static [u8; SIZE]) {
        self.sequence = sequence;
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn step(&mut self) {
        self.index += 1;

        if self.index == SIZE {
            self.index = 0;
        }
    }
}
