use std::ops::{Index, IndexMut};

pub struct MirrorVec<T: Clone + Default> {
    vec: Vec<T>,
    mask: usize,
}

impl<T: Clone + Default> MirrorVec<T> {
    pub fn new(size: usize) -> Self {
        Self {
            vec: vec![Default::default(); size],
            mask: Self::mask_for(size),
        }
    }

    pub fn resize(mut vec: Vec<T>) -> Self {
        let new_size = vec.len().next_power_of_two();

        vec.resize(new_size, T::default());

        Self {
            vec,
            mask: Self::mask_for(new_size),
        }
    }

    fn mask_for(size: usize) -> usize {
        if size.is_power_of_two() {
            size - 1
        } else if size == 0 {
            0
        } else {
            panic!("MirrorVec size must be a power of two");
        }
    }
}

impl<T: Clone + Default> From<Vec<T>> for MirrorVec<T> {
    fn from(vec: Vec<T>) -> Self {
        let size = vec.len();

        Self {
            vec,
            mask: Self::mask_for(size),
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
