const SIZE: usize = 8;

pub struct Fifo<T: Default + Clone + Copy> {
    values: [T; SIZE],
    index: usize,
}

impl<T: Default + Clone + Copy> Fifo<T> {
    pub fn new() -> Self {
        Self {
            values: [Default::default(); SIZE],
            index: SIZE,
        }
    }

    pub fn try_push(&mut self, values: [T; SIZE]) -> bool {
        if self.index == SIZE {
            self.values = values;
            self.index = 0;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.index < SIZE {
            let value = self.values[self.index];
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}
