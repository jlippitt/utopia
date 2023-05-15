use std::ops::{Index, IndexMut};

pub struct MirrorVec<T: Clone + Default> {
    vec: Vec<T>,
    mask: usize,
}

impl<T: Clone + Default> MirrorVec<T> {
    pub fn new(size: usize) -> Self {
        if !size.is_power_of_two() {
            panic!("MirrorVec size must be a power of two");
        }

        Self {
            vec: vec![Default::default(); size],
            mask: size - 1,
        }
    }
}

impl<T: Clone + Default> From<Vec<T>> for MirrorVec<T> {
    fn from(mut vec: Vec<T>) -> Self {
        let size = vec.len().next_power_of_two();

        vec.resize(size, Default::default());

        Self {
            vec,
            mask: size - 1,
        }
    }
}

impl<T: Clone + Default> Index<usize> for MirrorVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        unsafe { self.vec.get_unchecked(index & self.mask) }
    }
}

impl<T: Clone + Default> IndexMut<usize> for MirrorVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        unsafe { self.vec.get_unchecked_mut(index & self.mask) }
    }
}
